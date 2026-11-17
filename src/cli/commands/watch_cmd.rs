use anyhow::Result;
use clap::Args;

use crate::config;
use crate::utils::Xdg;
use crate::watch;

#[derive(Debug, Args)]
pub struct WatchArgs {
    #[arg(
        long,
        help = "Do not show a desktop notification when a default is put back"
    )]
    pub no_notify: bool,
}

pub fn watch_cmd(args: WatchArgs) -> Result<()> {
    let notify = if args.no_notify {
        Some(false)
    } else {
        config::notify_override()?
    };
    watch::run(Xdg::from_env()?, notify)
}
