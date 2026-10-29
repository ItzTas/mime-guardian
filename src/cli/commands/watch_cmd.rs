use anyhow::Result;
use clap::Args;

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
    watch::run(Xdg::from_env()?, !args.no_notify)
}
