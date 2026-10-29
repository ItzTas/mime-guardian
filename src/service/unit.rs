use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::constants::{
    COMMAND, DESKTOP_VARIABLE, SYSTEMCTL, UNIT_DIR, UNIT_NAME, UNIT_TEMPLATE, USER_FLAG,
};
use crate::files::replace_contents;
use crate::utils::Xdg;

pub fn unit_path(xdg: &Xdg) -> PathBuf {
    xdg.config_home.join(UNIT_DIR).join(UNIT_NAME)
}

pub fn unit(program: &Path) -> String {
    UNIT_TEMPLATE
        .replace("{documentation}", env!("CARGO_PKG_REPOSITORY"))
        .replace("{program}", &program.display().to_string())
}

pub fn install(xdg: &Xdg, program: &Path, start: bool) -> Result<PathBuf> {
    let path = unit_path(xdg);
    replace_contents(COMMAND, &path, unit(program).as_bytes())?;
    systemctl(&["daemon-reload"])?;
    let mut enable = vec!["enable", UNIT_NAME];
    if start {
        enable.insert(1, "--now");
    }
    systemctl(&enable)?;
    Ok(path)
}

pub fn uninstall(xdg: &Xdg) -> Result<Option<PathBuf>> {
    let path = unit_path(xdg);
    if !path.exists() {
        return Ok(None);
    }
    systemctl(&["disable", "--now", UNIT_NAME])?;
    std::fs::remove_file(&path)
        .with_context(|| format!("{COMMAND}: failed to remove {}", path.display()))?;
    systemctl(&["daemon-reload"])?;
    Ok(Some(path))
}

pub fn state(verb: &str) -> String {
    Command::new(SYSTEMCTL)
        .args([USER_FLAG, verb, UNIT_NAME])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default()
}

pub fn session_desktop() -> Option<String> {
    let output = Command::new(SYSTEMCTL)
        .args([USER_FLAG, "show-environment"])
        .output()
        .ok()?;
    desktop_in(&String::from_utf8_lossy(&output.stdout))
}

fn desktop_in(environment: &str) -> Option<String> {
    environment
        .lines()
        .find_map(|line| line.strip_prefix(DESKTOP_VARIABLE))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn systemctl(args: &[&str]) -> Result<()> {
    let status = Command::new(SYSTEMCTL)
        .arg(USER_FLAG)
        .args(args)
        .status()
        .with_context(|| format!("{COMMAND}: failed to run {SYSTEMCTL}"))?;
    if !status.success() {
        bail!(
            "{COMMAND}: {SYSTEMCTL} {USER_FLAG} {} failed",
            args.join(" ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unit_runs_watch_with_the_session() {
        let unit = unit(Path::new("/usr/bin/mmg"));

        assert!(unit.contains("ExecStart=/usr/bin/mmg watch\n"));
        assert!(unit.contains("WantedBy=graphical-session.target\n"));
        assert!(unit.contains("After=graphical-session.target\n"));
        assert!(!unit.contains('{'));
    }

    #[test]
    fn the_session_desktop_is_read_from_the_environment() {
        assert_eq!(
            desktop_in("HOME=/h\nXDG_CURRENT_DESKTOP=KDE\n"),
            Some("KDE".to_string())
        );
        assert_eq!(desktop_in("HOME=/h\nXDG_CURRENT_DESKTOP=\n"), None);
        assert_eq!(desktop_in("HOME=/h\n"), None);
    }
}
