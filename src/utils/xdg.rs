use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use super::constants::{
    COMMAND, CONFIG_DIRS, CONFIG_DIRS_DEFAULT, CONFIG_HOME, CONFIG_HOME_DEFAULT, CURRENT_DESKTOP,
    DATA_DIRS, DATA_DIRS_DEFAULT, DATA_HOME, DATA_HOME_DEFAULT, HOME, LIST_SEPARATOR, STATE_HOME,
    STATE_HOME_DEFAULT,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xdg {
    pub config_home: PathBuf,
    pub config_dirs: Vec<PathBuf>,
    pub data_home: PathBuf,
    pub data_dirs: Vec<PathBuf>,
    pub state_home: PathBuf,
    pub desktops: Vec<String>,
}

impl Xdg {
    pub fn from_env() -> Result<Self> {
        Self::from_vars(|name| std::env::var_os(name))
    }

    pub fn from_vars(var: impl Fn(&str) -> Option<OsString>) -> Result<Self> {
        let home = var(HOME)
            .map(PathBuf::from)
            .filter(|home| home.is_absolute());
        let home_dir = |name: &str, fallback: &str| -> Result<PathBuf> {
            if let Some(dir) = absolute(var(name)) {
                return Ok(dir);
            }
            let Some(home) = &home else {
                bail!("{COMMAND}: neither {name} nor {HOME} is set to an absolute path");
            };
            Ok(home.join(fallback))
        };

        Ok(Self {
            config_home: home_dir(CONFIG_HOME, CONFIG_HOME_DEFAULT)?,
            config_dirs: dirs(var(CONFIG_DIRS), CONFIG_DIRS_DEFAULT),
            data_home: home_dir(DATA_HOME, DATA_HOME_DEFAULT)?,
            data_dirs: dirs(var(DATA_DIRS), DATA_DIRS_DEFAULT),
            state_home: home_dir(STATE_HOME, STATE_HOME_DEFAULT)?,
            desktops: desktops(var(CURRENT_DESKTOP)),
        })
    }

    pub fn with_desktops(mut self, fallback: &[String]) -> Self {
        if self.desktops.is_empty() {
            self.desktops = fallback.iter().map(|name| name.to_lowercase()).collect();
        }
        self
    }
}

fn absolute(value: Option<OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|path| path.is_absolute())
}

fn dirs(value: Option<OsString>, fallback: &[&str]) -> Vec<PathBuf> {
    let listed: Vec<PathBuf> = value
        .as_deref()
        .and_then(|value| value.to_str())
        .map(|value| {
            value
                .split(LIST_SEPARATOR)
                .map(Path::new)
                .filter(|path| path.is_absolute())
                .map(Path::to_path_buf)
                .collect()
        })
        .unwrap_or_default();

    if !listed.is_empty() {
        return listed;
    }

    fallback.iter().map(PathBuf::from).collect()
}

fn desktops(value: Option<OsString>) -> Vec<String> {
    value
        .as_deref()
        .and_then(|value| value.to_str())
        .map(|value| {
            value
                .split(LIST_SEPARATOR)
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_lowercase)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn xdg(vars: &[(&str, &str)]) -> Result<Xdg> {
        let vars: HashMap<String, OsString> = vars
            .iter()
            .map(|(name, value)| (name.to_string(), OsString::from(value)))
            .collect();

        Xdg::from_vars(|name| vars.get(name).cloned())
    }

    #[test]
    fn home_fills_every_unset_directory() {
        let xdg = xdg(&[("HOME", "/home/me")]).unwrap();

        assert_eq!(xdg.config_home, PathBuf::from("/home/me/.config"));
        assert_eq!(xdg.data_home, PathBuf::from("/home/me/.local/share"));
        assert_eq!(xdg.state_home, PathBuf::from("/home/me/.local/state"));
        assert_eq!(xdg.config_dirs, [PathBuf::from("/etc/xdg")]);
        assert_eq!(
            xdg.data_dirs,
            [
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share")
            ]
        );
        assert!(xdg.desktops.is_empty());
    }

    #[test]
    fn relative_entries_are_ignored() {
        let xdg = xdg(&[
            ("HOME", "/home/me"),
            ("XDG_CONFIG_HOME", "relative"),
            ("XDG_DATA_DIRS", "nope:/opt/share"),
        ])
        .unwrap();

        assert_eq!(xdg.config_home, PathBuf::from("/home/me/.config"));
        assert_eq!(xdg.data_dirs, [PathBuf::from("/opt/share")]);
    }

    #[test]
    fn desktops_are_split_and_lowercased() {
        let xdg = xdg(&[("HOME", "/h"), ("XDG_CURRENT_DESKTOP", "KDE:GNOME")]).unwrap();

        assert_eq!(xdg.desktops, ["kde", "gnome"]);
        assert_eq!(
            xdg.with_desktops(&["sway".into()]).desktops,
            ["kde", "gnome"]
        );
    }

    #[test]
    fn the_fallback_desktops_fill_an_empty_session() {
        let xdg = xdg(&[("HOME", "/h")])
            .unwrap()
            .with_desktops(&["KDE".into()]);

        assert_eq!(xdg.desktops, ["kde"]);
    }

    #[test]
    fn no_home_and_no_override_is_an_error() {
        assert!(xdg(&[]).is_err());
    }
}
