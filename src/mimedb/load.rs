use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::{ALIASES_FILE, COMMAND, DIR, TYPES_FILE};
use super::types::MimeDb;
use crate::utils::Xdg;

impl MimeDb {
    pub fn load(xdg: &Xdg) -> Result<Self> {
        let mut db = Self::default();
        for dir in data_dirs(xdg) {
            let dir = dir.join(DIR);
            db.add(
                &read(&dir.join(TYPES_FILE))?.unwrap_or_default(),
                &read(&dir.join(ALIASES_FILE))?.unwrap_or_default(),
            );
        }
        Ok(db)
    }

    pub fn add(&mut self, types: &str, aliases: &str) {
        self.types.extend(words(types).map(str::to_string));
        for line in aliases.lines() {
            let mut parts = line.split_whitespace();
            let (Some(alias), Some(canonical)) = (parts.next(), parts.next()) else {
                continue;
            };
            self.aliases
                .entry(alias.to_string())
                .or_insert_with(|| canonical.to_string());
        }
    }

    pub fn aliases_of(&self, canonical: &str) -> Vec<String> {
        let mut aliases: Vec<String> = self
            .aliases
            .iter()
            .filter(|(_, target)| target.as_str() == canonical)
            .map(|(alias, _)| alias.clone())
            .collect();
        aliases.sort();
        aliases
    }

    pub fn canonical(&self, mime: &str) -> String {
        self.aliases
            .get(mime)
            .cloned()
            .unwrap_or_else(|| mime.to_string())
    }
}

fn data_dirs(xdg: &Xdg) -> Vec<PathBuf> {
    std::iter::once(xdg.data_home.clone())
        .chain(xdg.data_dirs.iter().cloned())
        .collect()
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(str::trim).filter(|line| !line.is_empty())
}

fn read(path: &Path) -> Result<Option<String>> {
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

    #[test]
    fn types_and_aliases_are_read() {
        let mut db = MimeDb::default();
        db.add(
            "image/png\nimage/jpeg\n\n",
            "application/x-pdf application/pdf\n",
        );

        assert!(db.types.contains("image/png"));
        assert_eq!(db.types.len(), 2);
        assert_eq!(db.canonical("application/x-pdf"), "application/pdf");
        assert_eq!(db.canonical("image/png"), "image/png");
        assert_eq!(db.aliases_of("application/pdf"), ["application/x-pdf"]);
    }

    #[test]
    fn the_user_database_comes_first() {
        let root = tempfile::tempdir().unwrap();
        let user = root.path().join("user");
        let system = root.path().join("system");
        for (dir, aliases) in [(&user, "a/x a/user\n"), (&system, "a/x a/system\n")] {
            std::fs::create_dir_all(dir.join("mime")).unwrap();
            std::fs::write(dir.join("mime/types"), "a/b\n").unwrap();
            std::fs::write(dir.join("mime/aliases"), aliases).unwrap();
        }
        let xdg = Xdg {
            config_home: root.path().join("config"),
            config_dirs: vec![],
            data_home: user,
            data_dirs: vec![system],
            state_home: root.path().join("state"),
            desktops: vec![],
        };

        let db = MimeDb::load(&xdg).unwrap();

        assert_eq!(db.canonical("a/x"), "a/user");
    }
}
