use std::path::{Path, PathBuf};

use super::constants::{
    APPLICATIONS_DIR, ENTRY_GROUP, EXTENSION, FLATPAK_SYSTEM_EXPORTS, FLATPAK_USER_EXPORTS,
    HIDDEN_KEY, ID_SEPARATOR, MIME_TYPE_KEY, TRUE,
};
use super::types::{App, Installed};
use crate::utils::Xdg;

pub fn app_dirs(xdg: &Xdg) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::iter::once(xdg.data_home.clone())
        .chain(xdg.data_dirs.iter().cloned())
        .collect();
    for flatpak in [
        xdg.data_home.join(FLATPAK_USER_EXPORTS),
        PathBuf::from(FLATPAK_SYSTEM_EXPORTS),
    ] {
        if !roots.contains(&flatpak) {
            roots.push(flatpak);
        }
    }
    roots
        .into_iter()
        .map(|root| root.join(APPLICATIONS_DIR))
        .collect()
}

impl Installed {
    pub fn scan(dirs: &[PathBuf]) -> Self {
        let mut installed = Self::default();
        for dir in dirs {
            installed.walk(dir, dir);
        }
        installed
    }

    fn walk(&mut self, root: &Path, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
        paths.sort();

        for path in paths {
            if path.is_dir() {
                self.walk(root, &path);
                continue;
            }
            self.add(root, &path);
        }
    }

    fn add(&mut self, root: &Path, path: &Path) {
        if path.extension().and_then(|ext| ext.to_str()) != Some(EXTENSION) {
            return;
        }
        let Some(id) = id(root, path) else {
            return;
        };
        if self.apps.contains_key(&id) {
            return;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            tracing::debug!("desktop: skipping unreadable {}", path.display());
            return;
        };
        self.apps.insert(id, parse(path, &text));
    }
}

fn id(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let parts: Option<Vec<&str>> = relative.iter().map(|part| part.to_str()).collect();
    Some(parts?.join(ID_SEPARATOR))
}

fn parse(path: &Path, text: &str) -> App {
    let mut app = App {
        path: path.to_path_buf(),
        hidden: false,
        mime_types: Vec::new(),
    };
    let mut in_entry = false;

    for line in text.lines().map(str::trim) {
        if let Some(group) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            in_entry = group == ENTRY_GROUP;
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            HIDDEN_KEY => app.hidden = value.trim() == TRUE,
            MIME_TYPE_KEY => app.mime_types = split(value),
            _ => {}
        }
    }

    app
}

fn split(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|mime| !mime.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn ids_come_from_the_relative_path() {
        let root = tempfile::tempdir().unwrap();
        let apps = root.path().join("applications");
        write(
            &apps.join("firefox.desktop"),
            "[Desktop Entry]\nName=Firefox\nMimeType=text/html;x-scheme-handler/https;\n[Desktop Action new]\nMimeType=bogus/type;\n",
        );
        write(&apps.join("kde4/okular.desktop"), "[Desktop Entry]\n");
        write(&apps.join("notes.txt"), "");

        let installed = Installed::scan(&[apps]);

        assert!(installed.contains("firefox.desktop"));
        assert!(installed.contains("kde4-okular.desktop"));
        assert_eq!(installed.apps.len(), 2);
        assert_eq!(
            installed.mime_types().collect::<Vec<_>>(),
            ["text/html", "x-scheme-handler/https"]
        );
    }

    #[test]
    fn a_hidden_user_entry_masks_the_system_one() {
        let root = tempfile::tempdir().unwrap();
        let user = root.path().join("user/applications");
        let system = root.path().join("system/applications");
        write(
            &user.join("chrome.desktop"),
            "[Desktop Entry]\nHidden=true\n",
        );
        write(
            &system.join("chrome.desktop"),
            "[Desktop Entry]\nName=Chrome\n",
        );

        let installed = Installed::scan(&[user, system]);

        assert!(!installed.contains("chrome.desktop"));
    }

    #[test]
    fn flatpak_exports_are_scanned_once() {
        let xdg = Xdg {
            config_home: "/h/.config".into(),
            config_dirs: vec![],
            data_home: "/h/.local/share".into(),
            data_dirs: vec!["/var/lib/flatpak/exports/share".into(), "/usr/share".into()],
            state_home: "/h/.local/state".into(),
            desktops: vec![],
        };

        assert_eq!(
            app_dirs(&xdg),
            [
                PathBuf::from("/h/.local/share/applications"),
                PathBuf::from("/var/lib/flatpak/exports/share/applications"),
                PathBuf::from("/usr/share/applications"),
                PathBuf::from("/h/.local/share/flatpak/exports/share/applications"),
            ]
        );
    }
}
