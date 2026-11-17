use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Args;

use super::super::constants::{
    ADOPT_COMMAND, ADOPT_KEPT, ADOPT_KEPT_HINT, ADOPT_NO_CONFIG, ADOPT_REFUSED, UNSET,
};
use crate::adopt::{self, Adoption, Change};
use crate::config;
use crate::desktop::{Installed, app_dirs};
use crate::files::replace_contents;
use crate::mimeapps::{self, MimeApps};
use crate::output::{self, Tone};
use crate::utils::Xdg;
use crate::watch;

#[derive(Debug, Args)]
pub struct AdoptArgs {
    #[arg(
        required = true,
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_name = "COMMAND",
        help = "The program to run, followed by its arguments"
    )]
    pub command: Vec<OsString>,
}

pub fn adopt_cmd(args: AdoptArgs) -> Result<()> {
    let Some((program, program_args)) = args.command.split_first() else {
        bail!("{ADOPT_COMMAND}: no command to run");
    };

    let xdg = Xdg::from_env()?;
    let config_path = config::path(&xdg);
    let config_text = config::read(&config_path)?;
    let desktops = match &config_text {
        Some(text) => config::parse(text)?.guard.desktops,
        None => Vec::new(),
    };
    let xdg = xdg.with_desktops(&desktops);
    let user_files = mimeapps::user_files(&xdg);
    let _pause = watch::pause(&xdg)?;

    let before = snapshot(&user_files)?;
    run(program, program_args)?;
    let after = snapshot(&user_files)?;

    let changes = adopt::changes(&before, &after);
    if changes.is_empty() {
        return Ok(());
    }
    let Some(text) = config_text else {
        output::hint(ADOPT_NO_CONFIG);
        return Ok(());
    };

    let adoption = adopt::adopt(&text, packaged(changes, &xdg))?;
    if adoption.text != text {
        replace_contents(ADOPT_COMMAND, &config_path, adoption.text.as_bytes())?;
    }
    report(&adoption, &config_path);
    Ok(())
}

fn packaged(changes: Vec<Change>, xdg: &Xdg) -> Vec<Change> {
    let installed = Installed::scan(&app_dirs(xdg));
    changes
        .into_iter()
        .map(|change| Change {
            after: change.after.map(|app| installed.canonical(&app)),
            ..change
        })
        .collect()
}

fn snapshot(paths: &[PathBuf]) -> Result<Vec<MimeApps>> {
    paths
        .iter()
        .map(|path| Ok(mimeapps::parse(&mimeapps::read(path)?.unwrap_or_default())))
        .collect()
}

fn run(program: &OsString, args: &[OsString]) -> Result<()> {
    let name = program.to_string_lossy();
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("{ADOPT_COMMAND}: failed to run {name}"))?;
    if !status.success() {
        bail!("{ADOPT_COMMAND}: {name} exited with {status}, nothing was kept");
    }
    Ok(())
}

fn report(adoption: &Adoption, config_path: &Path) {
    if !adoption.adopted.is_empty() {
        output::section(format!("{ADOPT_KEPT} {}", config_path.display()));
        for change in &adoption.adopted {
            output::item(Tone::Good, &change.mime, describe(change));
        }
        output::hint(ADOPT_KEPT_HINT);
    }

    if !adoption.refused.is_empty() {
        output::section(ADOPT_REFUSED);
        for (change, reason) in &adoption.refused {
            output::item(Tone::Warning, &change.mime, reason);
        }
    }
}

fn describe(change: &Change) -> String {
    let before = change.before.as_deref().unwrap_or(UNSET);
    let after = change.after.as_deref().unwrap_or(UNSET);
    format!("{before} → {after}")
}
