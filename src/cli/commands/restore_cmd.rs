use anyhow::{Context as _, Result};
use clap::Args;

use super::super::constants::{RESTORE_COMMAND, RESTORE_EMPTY, RESTORE_WATCH_HINT};
use crate::backup::{self, Backup, Original};
use crate::files::replace_contents;
use crate::mimeapps;
use crate::output::{self, Tone};
use crate::reconcile::target;
use crate::utils::{Xdg, utc};

#[derive(Debug, Args)]
pub struct RestoreArgs {
    #[arg(
        value_name = "ID",
        help = "The backup to restore; the newest one when left out"
    )]
    pub id: Option<String>,
    #[arg(short, long, help = "List the backups instead of restoring one")]
    pub list: bool,
}

pub fn restore_cmd(args: RestoreArgs) -> Result<()> {
    let xdg = Xdg::from_env()?;
    let root = backup::root(&xdg);
    if args.list {
        return list(&backup::list(&root)?);
    }

    let chosen = backup::find(&root, args.id.as_deref())?;
    let current = snapshot(&xdg, &chosen)?;
    let originals: Vec<Original> = current
        .iter()
        .map(|(name, contents)| Original {
            name,
            contents: contents.as_deref(),
        })
        .collect();
    let undo = backup::save(&root, &originals)?;

    for name in &chosen.saved {
        let contents = std::fs::read(chosen.dir.join(name)).with_context(|| {
            format!(
                "{RESTORE_COMMAND}: failed to read {name} from backup {}",
                chosen.id
            )
        })?;
        let path = xdg.config_home.join(name);
        replace_contents(RESTORE_COMMAND, &target(&path), &contents)?;
        output::item(Tone::Good, "restore", path.display());
    }
    for name in &chosen.created {
        let path = xdg.config_home.join(name);
        if !path.exists() {
            continue;
        }
        std::fs::remove_file(&path)
            .with_context(|| format!("{RESTORE_COMMAND}: failed to remove {}", path.display()))?;
        output::item(Tone::Warning, "remove", path.display());
    }

    output::note(format!(
        "restored backup {}; what it replaced is in backup {}",
        chosen.id, undo.id
    ));
    output::hint(RESTORE_WATCH_HINT);
    Ok(())
}

fn snapshot(xdg: &Xdg, chosen: &Backup) -> Result<Vec<(String, Option<String>)>> {
    chosen
        .saved
        .iter()
        .chain(&chosen.created)
        .map(|name| {
            let current = mimeapps::read(&xdg.config_home.join(name))?;
            Ok((name.clone(), current))
        })
        .collect()
}

fn list(backups: &[Backup]) -> Result<()> {
    if backups.is_empty() {
        output::note(RESTORE_EMPTY);
        return Ok(());
    }
    for backup in backups {
        let when = backup.id.parse::<u64>().map(utc).unwrap_or_default();
        let files: Vec<&str> = backup
            .saved
            .iter()
            .chain(&backup.created)
            .map(String::as_str)
            .collect();
        output::entry(
            Tone::Strong,
            &backup.id,
            format!("{when}  {}", files.join(", ")),
        );
    }
    Ok(())
}
