use anyhow::Result;
use clap::Args;

use super::super::constants::{
    STATUS_CLEAN, STATUS_OVERRIDDEN, STATUS_OVERRIDDEN_HINT, STATUS_SKIPPED, STATUS_SKIPPED_HINT,
    STATUS_UNPINNED, STATUS_UNPINNED_HINT, UNSET,
};
use super::context::Context;
use crate::history::Event;
use crate::output::{self, Tone};
use crate::reconcile::{Drift, Plan};

#[derive(Debug, Args)]
pub struct StatusArgs {}

pub fn status_cmd(_args: StatusArgs) -> Result<()> {
    let context = Context::load()?;
    let plan = context.plan();

    output::title(format!(
        "{} protected types in {}",
        context.wanted.len(),
        context.config_path.display()
    ));
    report(&plan);
    Ok(())
}

pub fn report(plan: &Plan) {
    let overridden: Vec<&Drift> = plan.overridden().collect();
    let unpinned: Vec<&Drift> = plan.unpinned().collect();

    drifts(
        STATUS_OVERRIDDEN,
        STATUS_OVERRIDDEN_HINT,
        Tone::Bad,
        &overridden,
    );
    drifts(
        STATUS_UNPINNED,
        STATUS_UNPINNED_HINT,
        Tone::Warning,
        &unpinned,
    );

    if !plan.skipped.is_empty() {
        output::section(STATUS_SKIPPED);
        for skipped in &plan.skipped {
            output::item(Tone::Muted, &skipped.mime, &skipped.app);
        }
        output::hint(STATUS_SKIPPED_HINT);
    }

    if plan.is_clean() {
        output::note(STATUS_CLEAN);
    }
}

fn drifts(head: &str, hint: &str, tone: Tone, drifts: &[&Drift]) {
    if drifts.is_empty() {
        return;
    }
    output::section(head);
    for drift in drifts {
        output::item(tone, &drift.mime, describe(drift));
    }
    output::hint(hint);
}

fn describe(drift: &Drift) -> String {
    change(drift.previous(), &drift.wanted)
}

pub fn describe_event(event: &Event) -> String {
    change(event.from.as_deref(), &event.to)
}

fn change(from: Option<&str>, to: &str) -> String {
    let from = from.unwrap_or(UNSET);
    if from == to {
        return to.to_string();
    }
    format!("{from} → {to}")
}
