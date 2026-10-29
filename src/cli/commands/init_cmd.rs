use anyhow::{Result, bail};
use clap::Args;

use super::super::constants::{INIT_COMMAND, INIT_NEXT};
use crate::config;
use crate::files::replace_contents;
use crate::output;
use crate::reconcile::target;
use crate::system::System;
use crate::utils::Xdg;

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(short, long, help = "Overwrite a configuration that already exists")]
    pub force: bool,
}

pub fn init_cmd(args: InitArgs) -> Result<()> {
    let xdg = Xdg::from_env()?;
    let path = config::path(&xdg);
    if !args.force && path.exists() {
        bail!(
            "{INIT_COMMAND}: {} already exists (use --force to replace it)",
            path.display()
        );
    }

    let defaults = System::load(xdg)?.current_defaults();
    replace_contents(
        INIT_COMMAND,
        &target(&path),
        config::initial(&defaults).as_bytes(),
    )?;

    output::note(format!(
        "created {} with {} defaults this session already uses",
        path.display(),
        defaults.len()
    ));
    output::hint(INIT_NEXT);
    Ok(())
}
