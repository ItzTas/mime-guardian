use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context as _, Result};
use notify_debouncer_full::notify::RecursiveMode;
use notify_debouncer_full::{DebouncedEvent, new_debouncer};

use super::constants::{COMMAND, DEBOUNCE_MILLIS, SETTLE_ATTEMPTS, SETTLE_MILLIS};
use super::pause::{paused, pauses_dir};
use crate::alert;
use crate::config;
use crate::history::{Event, Trigger};
use crate::mimeapps;
use crate::output::{self, Tone};
use crate::reconcile;
use crate::system::Context;
use crate::utils::{Xdg, utc};

struct Watched {
    xdg: Xdg,
    config: PathBuf,
    pauses: PathBuf,
    notify: Option<bool>,
}

pub fn run(xdg: Xdg, notify: Option<bool>) -> Result<()> {
    let watched = Watched {
        config: config::path(&xdg),
        pauses: pauses_dir(&xdg),
        xdg,
        notify,
    };

    let (sender, receiver) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(DEBOUNCE_MILLIS), None, sender)
        .with_context(|| format!("{COMMAND}: failed to start watching"))?;
    for dir in watched.dirs() {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("{COMMAND}: failed to create {}", dir.display()))?;
        debouncer
            .watch(&dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("{COMMAND}: failed to watch {}", dir.display()))?;
        tracing::debug!("{COMMAND}: watching {}", dir.display());
    }

    output::note(format!(
        "watching {} for changes",
        watched.xdg.config_home.display()
    ));
    watched.cycle();

    for result in receiver {
        match result {
            Ok(events) if events.iter().any(|event| watched.relevant(event)) => watched.cycle(),
            Ok(_) => {}
            Err(errors) => {
                for err in errors {
                    output::warn(format!("{COMMAND}: {err}"));
                }
            }
        }
    }
    Ok(())
}

impl Watched {
    fn dirs(&self) -> Vec<PathBuf> {
        let mut dirs = vec![self.xdg.config_home.clone(), self.pauses.clone()];
        if let Some(parent) = self.config.parent() {
            dirs.push(parent.to_path_buf());
        }
        dirs
    }

    fn relevant(&self, event: &DebouncedEvent) -> bool {
        event.paths.iter().any(|path| {
            path == &self.config
                || path.parent() == Some(self.pauses.as_path())
                || path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(mimeapps::is_mimeapps)
        })
    }

    fn cycle(&self) {
        if let Err(err) = self.reconcile() {
            output::warn(format!("{err:#}"));
        }
    }

    fn reconcile(&self) -> Result<()> {
        if paused(&self.pauses) {
            tracing::debug!("{COMMAND}: paused, leaving the files alone");
            return Ok(());
        }
        settle(&mimeapps::user_files(&self.xdg))?;

        let context = Context::load_with(self.xdg.clone())?;
        let plan = context.plan();
        if plan.is_clean() {
            return Ok(());
        }
        let outcome = reconcile::apply(&context.system.xdg, &plan, Trigger::Watch)?;
        for path in &outcome.conflicts {
            tracing::debug!(
                "{COMMAND}: {} changed again, waiting for it",
                path.display()
            );
        }

        report(&outcome.events);
        let overridden: Vec<Event> = outcome
            .events
            .into_iter()
            .filter(|event| event.overridden)
            .collect();
        if self.notify.unwrap_or(context.config.guard.notify) {
            alert::send(&overridden, &context.config.notification);
        }
        Ok(())
    }
}

fn settle(paths: &[PathBuf]) -> Result<()> {
    let mut last = read_all(paths)?;
    for _ in 0..SETTLE_ATTEMPTS {
        std::thread::sleep(Duration::from_millis(SETTLE_MILLIS));
        let now = read_all(paths)?;
        if now == last {
            return Ok(());
        }
        last = now;
    }
    Ok(())
}

fn read_all(paths: &[PathBuf]) -> Result<Vec<Option<String>>> {
    paths
        .iter()
        .map(|path| mimeapps::read(Path::new(path)))
        .collect()
}

fn report(events: &[Event]) {
    for event in events {
        let from = event.from.as_deref().unwrap_or("(none)");
        let tone = if event.overridden {
            Tone::Warning
        } else {
            Tone::Muted
        };
        output::entry(
            tone,
            utc(event.at),
            format!("{}: {from} → {}", event.mime, event.to),
        );
    }
}
