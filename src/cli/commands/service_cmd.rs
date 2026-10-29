use anyhow::{Context as _, Result};
use clap::{Args, Subcommand};

use super::super::constants::{SERVICE_COMMAND, SERVICE_NO_DESKTOP, SERVICE_NOT_INSTALLED};
use crate::output;
use crate::service;
use crate::utils::Xdg;

#[derive(Debug, Args)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub action: ServiceAction,
}

#[derive(Debug, Subcommand)]
pub enum ServiceAction {
    #[command(about = "Write the systemd user unit, enable it and start it")]
    Install(InstallArgs),
    #[command(about = "Stop and disable the unit, then remove it")]
    Uninstall,
    #[command(about = "Show whether the unit is installed, enabled and running")]
    Status,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    #[arg(long, help = "Enable the unit without starting it now")]
    pub no_start: bool,
}

pub fn service_cmd(args: ServiceArgs) -> Result<()> {
    let xdg = Xdg::from_env()?;
    match args.action {
        ServiceAction::Install(install) => install_unit(&xdg, !install.no_start),
        ServiceAction::Uninstall => uninstall_unit(&xdg),
        ServiceAction::Status => status(&xdg),
    }
}

fn install_unit(xdg: &Xdg, start: bool) -> Result<()> {
    let program = std::env::current_exe()
        .with_context(|| format!("{SERVICE_COMMAND}: failed to find the mmg binary"))?;
    let path = service::install(xdg, &program, start)?;
    output::note(format!("installed {}", path.display()));
    warn_without_desktop();
    Ok(())
}

fn uninstall_unit(xdg: &Xdg) -> Result<()> {
    let Some(path) = service::uninstall(xdg)? else {
        output::note(SERVICE_NOT_INSTALLED);
        return Ok(());
    };
    output::note(format!("removed {}", path.display()));
    Ok(())
}

fn status(xdg: &Xdg) -> Result<()> {
    let path = service::unit_path(xdg);
    if !path.exists() {
        output::note(SERVICE_NOT_INSTALLED);
        return Ok(());
    }
    output::field("unit", path.display());
    output::field("enabled", service::state("is-enabled"));
    output::field("active", service::state("is-active"));
    output::field(
        "desktop",
        service::session_desktop().unwrap_or_else(|| "(not set)".to_string()),
    );
    warn_without_desktop();
    Ok(())
}

fn warn_without_desktop() {
    if service::session_desktop().is_none() {
        output::warn(SERVICE_NO_DESKTOP);
    }
}
