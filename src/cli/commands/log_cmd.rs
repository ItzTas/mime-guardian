use anyhow::Result;
use clap::Args;

use super::super::constants::{LOG_DEFAULT_LIMIT, LOG_EMPTY};
use super::status_cmd::describe_event;
use crate::history::{self, Trigger};
use crate::output::{self, Tone};
use crate::utils::{Xdg, utc};

#[derive(Debug, Args)]
pub struct LogArgs {
    #[arg(short = 'n', long, default_value_t = LOG_DEFAULT_LIMIT, help = "How many changes to show")]
    pub limit: usize,
    #[arg(short, long, help = "Show the types mmg pinned for the first time too")]
    pub all: bool,
}

pub fn log_cmd(args: LogArgs) -> Result<()> {
    let xdg = Xdg::from_env()?;
    let events = history::read(&history::path(&xdg))?;

    let shown: Vec<_> = events
        .iter()
        .rev()
        .filter(|event| args.all || event.overridden)
        .take(args.limit)
        .collect();
    if shown.is_empty() {
        output::note(LOG_EMPTY);
        return Ok(());
    }

    for event in shown {
        let trigger = match event.trigger {
            Trigger::Apply => "apply",
            Trigger::Watch => "watch",
        };
        output::entry(
            Tone::Muted,
            utc(event.at),
            format!("{}  {}  ({trigger})", event.mime, describe_event(event)),
        );
    }
    Ok(())
}
