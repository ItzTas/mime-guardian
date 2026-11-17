use std::path::Path;

use anyhow::Result;
use clap::{Args, Subcommand};

use super::super::constants::{CONFIG_COMMAND, SWITCH_OFF, SWITCH_ON};
use super::set_cmd::existing;
use crate::config::{self, NOTIFY_ENV};
use crate::files::replace_contents;
use crate::output;
use crate::reconcile::target;
use crate::utils::Xdg;

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub setting: ConfigSetting,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSetting {
    #[command(
        about = "Show or switch the notification mmg watch shows when it puts a default back",
        long_about = "Show or switch the notification mmg watch shows when it puts a default back. \
The choice is saved as guard.notify in the configuration, and a running watcher picks it up \
without a restart. The MMG_NOTIFY environment variable takes the same words and overrides the \
configuration for a watcher started with it."
    )]
    Notify(NotifyArgs),
}

#[derive(Debug, Args)]
pub struct NotifyArgs {
    #[arg(
        value_name = "STATE",
        value_parser = config::parse_switch,
        help = "on, off, true, false, 1, 0, yes or no; leave it out to show the current state"
    )]
    pub state: Option<bool>,
}

pub fn config_cmd(args: ConfigArgs) -> Result<()> {
    let xdg = Xdg::from_env()?;
    match args.setting {
        ConfigSetting::Notify(notify) => notify_setting(&xdg, notify.state),
    }
}

fn notify_setting(xdg: &Xdg, state: Option<bool>) -> Result<()> {
    let path = config::path(xdg);
    let overridden = config::notify_override()?;
    let Some(on) = state else {
        return show_notify(&path, overridden);
    };

    let text = existing(CONFIG_COMMAND, &path)?;
    let updated = config::set_notify(&text, on)?;
    config::parse(&updated)?;
    replace_contents(CONFIG_COMMAND, &target(&path), updated.as_bytes())?;
    output::note(format!("notifications are {}", word(on)));

    if let Some(forced) = overridden.filter(|forced| *forced != on) {
        output::warn(format!(
            "{NOTIFY_ENV} is {} in this shell, so a watcher started from it keeps them {}",
            word(forced),
            word(forced)
        ));
    }
    Ok(())
}

fn show_notify(path: &Path, overridden: Option<bool>) -> Result<()> {
    let configured = config::load(path)?.guard.notify;
    let Some(forced) = overridden else {
        output::line(word(configured));
        return Ok(());
    };
    output::line(word(forced));
    output::hint(format!(
        "({NOTIFY_ENV} overrides the configuration, which says {})",
        word(configured)
    ));
    Ok(())
}

fn word(on: bool) -> &'static str {
    if on { SWITCH_ON } else { SWITCH_OFF }
}
