use anyhow::{Context as _, Result, bail};
use clap::Args;

use super::super::constants::{DESKTOP_SUFFIX, SET_COMMAND, UNSET_COMMAND};
use super::apply_cmd::pin;
use crate::config;
use crate::files::replace_contents;
use crate::output;
use crate::reconcile::target;
use crate::system::{Context, System};
use crate::utils::Xdg;

#[derive(Debug, Args)]
pub struct SetArgs {
    #[arg(
        value_name = "MIME",
        help = "The type to protect, such as text/html or image/*"
    )]
    pub mime: String,
    #[arg(
        value_name = "APP",
        help = "The desktop entry that opens it, such as firefox.desktop"
    )]
    pub app: String,
    #[arg(
        short,
        long,
        help = "Set it even when the application is not installed"
    )]
    pub force: bool,
    #[arg(long, help = "Only change the configuration, without pinning it now")]
    pub no_apply: bool,
}

#[derive(Debug, Args)]
pub struct UnsetArgs {
    #[arg(value_name = "MIME", help = "The type to stop protecting")]
    pub mime: String,
}

pub fn set_cmd(args: SetArgs) -> Result<()> {
    config::validate(&args.mime).with_context(|| format!("{SET_COMMAND}: refused"))?;
    let app = desktop_id(&args.app);
    let xdg = Xdg::from_env()?;
    if !args.force && !System::load(xdg.clone())?.installed(&app) {
        bail!("{SET_COMMAND}: {app} is not installed (use --force to set it anyway)");
    }

    let path = config::path(&xdg);
    let text = existing(SET_COMMAND, &path)?;
    let updated = config::set_default(&text, &args.mime, &app)?;
    config::parse(&updated)?;
    replace_contents(SET_COMMAND, &target(&path), updated.as_bytes())?;
    output::note(format!("{} now opens with {app}", args.mime));

    if args.no_apply {
        return Ok(());
    }
    let context = Context::load_with(xdg)?;
    let plan = context.plan();
    if plan.is_clean() {
        return Ok(());
    }
    pin(&context, &plan)
}

pub fn unset_cmd(args: UnsetArgs) -> Result<()> {
    let xdg = Xdg::from_env()?;
    let path = config::path(&xdg);
    let text = existing(UNSET_COMMAND, &path)?;
    let Some(updated) = config::unset_default(&text, &args.mime)? else {
        bail!(
            "{UNSET_COMMAND}: {} is not in {}",
            args.mime,
            path.display()
        );
    };
    replace_contents(UNSET_COMMAND, &target(&path), updated.as_bytes())?;
    output::note(format!(
        "mmg no longer protects {}; its current default stays as it is",
        args.mime
    ));
    Ok(())
}

fn desktop_id(app: &str) -> String {
    if app.ends_with(DESKTOP_SUFFIX) {
        return app.to_string();
    }
    format!("{app}{DESKTOP_SUFFIX}")
}

fn existing(command: &str, path: &std::path::Path) -> Result<String> {
    let Some(text) = config::read(path)? else {
        bail!(
            "{command}: {} does not exist (use \"mmg init\" to create it)",
            path.display()
        );
    };
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_desktop_suffix_is_added_once() {
        assert_eq!(desktop_id("firefox"), "firefox.desktop");
        assert_eq!(desktop_id("firefox.desktop"), "firefox.desktop");
    }
}
