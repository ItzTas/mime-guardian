use anyhow::Result;
use clap::Args;

use super::super::constants::{APPLY_CONFLICTS, APPLY_DRY_RUN, APPLY_PINNED};
use super::status_cmd::{describe_event, report};
use crate::history::Trigger;
use crate::output::{self, Tone};
use crate::reconcile;
use crate::system::Context;

#[derive(Debug, Args)]
pub struct ApplyArgs {
    #[arg(
        short = 'n',
        long,
        help = "Show what would change without writing anything"
    )]
    pub dry_run: bool,
}

pub fn apply_cmd(args: ApplyArgs) -> Result<()> {
    let context = Context::load()?;
    let plan = context.plan();

    if args.dry_run || plan.is_clean() {
        report(&plan);
        if !plan.is_clean() {
            output::section(APPLY_DRY_RUN);
            for edit in &plan.files {
                output::item(Tone::Muted, "write", edit.path.display());
            }
        }
        return Ok(());
    }

    let outcome = reconcile::apply(&context.system.xdg, &plan, Trigger::Apply)?;

    if !outcome.events.is_empty() {
        output::section(APPLY_PINNED);
        for event in &outcome.events {
            let tone = if event.overridden {
                Tone::Good
            } else {
                Tone::Muted
            };
            output::item(tone, &event.mime, describe_event(event));
        }
    }
    if let Some(backup) = &outcome.backup {
        let count = outcome.written.len();
        let noun = if count == 1 { "file" } else { "files" };
        output::note(format!(
            "wrote {count} {noun}, the old contents are in backup {}",
            backup.id
        ));
    }
    if !outcome.conflicts.is_empty() {
        output::warn(APPLY_CONFLICTS);
        for path in &outcome.conflicts {
            output::detail(path.display());
        }
    }
    Ok(())
}
