use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::{APPLICATIONS_DIR, COMMAND, DESKTOP_SUFFIX, FILE_NAME};
use super::parse::parse;
use super::types::{Layer, Source};
use crate::utils::Xdg;

pub fn sources(xdg: &Xdg) -> Vec<Source> {
    let config = std::iter::once((xdg.config_home.clone(), true))
        .chain(xdg.config_dirs.iter().map(|dir| (dir.clone(), false)));
    let data = std::iter::once(xdg.data_home.clone())
        .chain(xdg.data_dirs.iter().cloned())
        .map(|dir| (dir.join(APPLICATIONS_DIR), false));

    config
        .chain(data)
        .flat_map(|(dir, writable)| in_dir(&dir, &xdg.desktops, writable))
        .collect()
}

pub fn user_files(xdg: &Xdg) -> Vec<PathBuf> {
    sources(xdg)
        .into_iter()
        .filter(|source| source.writable)
        .map(|source| source.path)
        .collect()
}

pub fn generic_user_file(xdg: &Xdg) -> PathBuf {
    xdg.config_home.join(FILE_NAME)
}

pub fn is_mimeapps(name: &str) -> bool {
    name == FILE_NAME || name.ends_with(DESKTOP_SUFFIX)
}

fn in_dir(dir: &Path, desktops: &[String], writable: bool) -> Vec<Source> {
    desktops
        .iter()
        .map(|desktop| dir.join(format!("{desktop}{DESKTOP_SUFFIX}")))
        .chain(std::iter::once(dir.join(FILE_NAME)))
        .map(|path| Source { path, writable })
        .collect()
}

pub fn load(sources: Vec<Source>) -> Result<Vec<Layer>> {
    let mut layers = Vec::new();
    for source in sources {
        let Some(text) = read(&source.path)? else {
            continue;
        };
        layers.push(Layer {
            source,
            file: parse(&text),
        });
    }
    Ok(layers)
}

pub fn read(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("{COMMAND}: failed to read {}", path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xdg(desktops: &[&str]) -> Xdg {
        Xdg {
            config_home: "/home/me/.config".into(),
            config_dirs: vec!["/etc/xdg".into()],
            data_home: "/home/me/.local/share".into(),
            data_dirs: vec!["/usr/share".into()],
            state_home: "/home/me/.local/state".into(),
            desktops: desktops.iter().map(|name| name.to_string()).collect(),
        }
    }

    #[test]
    fn the_order_follows_the_specification() {
        let paths: Vec<(String, bool)> = sources(&xdg(&["kde"]))
            .into_iter()
            .map(|source| (source.path.display().to_string(), source.writable))
            .collect();

        assert_eq!(
            paths,
            [
                ("/home/me/.config/kde-mimeapps.list".into(), true),
                ("/home/me/.config/mimeapps.list".into(), true),
                ("/etc/xdg/kde-mimeapps.list".into(), false),
                ("/etc/xdg/mimeapps.list".into(), false),
                (
                    "/home/me/.local/share/applications/kde-mimeapps.list".into(),
                    false
                ),
                (
                    "/home/me/.local/share/applications/mimeapps.list".into(),
                    false
                ),
                ("/usr/share/applications/kde-mimeapps.list".into(), false),
                ("/usr/share/applications/mimeapps.list".into(), false),
            ]
        );
    }

    #[test]
    fn user_files_are_the_config_home_ones() {
        assert_eq!(
            user_files(&xdg(&["kde", "gnome"])),
            [
                PathBuf::from("/home/me/.config/kde-mimeapps.list"),
                PathBuf::from("/home/me/.config/gnome-mimeapps.list"),
                PathBuf::from("/home/me/.config/mimeapps.list"),
            ]
        );
    }

    #[test]
    fn only_existing_files_become_layers() {
        let dir = tempfile::tempdir().unwrap();
        let present = dir.path().join("mimeapps.list");
        std::fs::write(&present, "[Default Applications]\na/b=x;\n").unwrap();

        let layers = load(vec![
            Source {
                path: dir.path().join("kde-mimeapps.list"),
                writable: true,
            },
            Source {
                path: present.clone(),
                writable: true,
            },
        ])
        .unwrap();

        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].source.path, present);
        assert_eq!(layers[0].file.defaults("a/b"), ["x"]);
    }

    #[test]
    fn only_mimeapps_names_are_recognized() {
        assert!(is_mimeapps("mimeapps.list"));
        assert!(is_mimeapps("kde-mimeapps.list"));
        assert!(!is_mimeapps("mimeapps.list.tmp"));
        assert!(!is_mimeapps(".mimeapps.list.mime-guardian-1-0"));
    }
}
