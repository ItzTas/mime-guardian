use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::constants::{COMMAND, CREATED_FILE, DIR, KEEP};
use super::types::Backup;
use crate::files::replace_contents;
use crate::utils::{Xdg, now_millis};

pub struct Original<'a> {
    pub name: &'a str,
    pub contents: Option<&'a str>,
}

pub fn root(xdg: &Xdg) -> PathBuf {
    xdg.state_home.join(DIR)
}

pub fn save(root: &Path, originals: &[Original]) -> Result<Backup> {
    let (id, dir) = reserve(root)?;
    let mut created = Vec::new();
    let mut saved = Vec::new();

    for original in originals {
        let Some(contents) = original.contents else {
            created.push(original.name.to_string());
            continue;
        };
        replace_contents(COMMAND, &dir.join(original.name), contents.as_bytes())?;
        saved.push(original.name.to_string());
    }
    if !created.is_empty() {
        let listed = format!("{}\n", created.join("\n"));
        replace_contents(COMMAND, &dir.join(CREATED_FILE), listed.as_bytes())?;
    }
    prune(root, KEEP)?;

    Ok(Backup {
        id,
        dir,
        saved,
        created,
    })
}

pub fn list(root: &Path) -> Result<Vec<Backup>> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("{COMMAND}: failed to list {}", root.display()));
        }
    };
    let mut backups = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        backups.push(open(dir)?);
    }
    backups.sort_by(|left, right| right.id.cmp(&left.id));
    Ok(backups)
}

pub fn find(root: &Path, id: Option<&str>) -> Result<Backup> {
    let backups = list(root)?;
    let found = match id {
        Some(id) => backups.into_iter().find(|backup| backup.id == id),
        None => backups.into_iter().next(),
    };
    let Some(backup) = found else {
        bail!(
            "{COMMAND}: no backup {}in {}",
            id.map(|id| format!("{id} ")).unwrap_or_default(),
            root.display()
        );
    };
    Ok(backup)
}

pub fn prune(root: &Path, keep: usize) -> Result<()> {
    for stale in list(root)?.into_iter().skip(keep) {
        std::fs::remove_dir_all(&stale.dir)
            .with_context(|| format!("{COMMAND}: failed to remove {}", stale.dir.display()))?;
    }
    Ok(())
}

fn open(dir: PathBuf) -> Result<Backup> {
    let id = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let mut saved = Vec::new();
    for entry in std::fs::read_dir(&dir)
        .with_context(|| format!("{COMMAND}: failed to list {}", dir.display()))?
        .flatten()
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name != CREATED_FILE {
            saved.push(name);
        }
    }
    saved.sort();
    let created = std::fs::read_to_string(dir.join(CREATED_FILE))
        .map(|text| text.lines().map(str::to_string).collect())
        .unwrap_or_default();

    Ok(Backup {
        id,
        dir,
        saved,
        created,
    })
}

fn reserve(root: &Path) -> Result<(String, PathBuf)> {
    std::fs::create_dir_all(root)
        .with_context(|| format!("{COMMAND}: failed to create {}", root.display()))?;
    let stamp = now_millis();
    for attempt in 0..u64::from(u16::MAX) {
        let id = format!("{:016}", stamp + attempt);
        let dir = root.join(&id);
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok((id, dir)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("{COMMAND}: failed to create {}", dir.display()));
            }
        }
    }
    bail!(
        "{COMMAND}: failed to reserve a backup in {}",
        root.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backup_keeps_contents_and_created_names() {
        let root = tempfile::tempdir().unwrap();

        let backup = save(
            root.path(),
            &[
                Original {
                    name: "mimeapps.list",
                    contents: Some("old"),
                },
                Original {
                    name: "kde-mimeapps.list",
                    contents: None,
                },
            ],
        )
        .unwrap();

        let found = find(root.path(), None).unwrap();
        assert_eq!(found, backup);
        assert_eq!(found.saved, ["mimeapps.list"]);
        assert_eq!(found.created, ["kde-mimeapps.list"]);
        assert_eq!(
            std::fs::read_to_string(found.dir.join("mimeapps.list")).unwrap(),
            "old"
        );
    }

    #[test]
    fn the_newest_backups_survive_pruning() {
        let root = tempfile::tempdir().unwrap();
        let ids: Vec<String> = (0..4)
            .map(|_| {
                save(
                    root.path(),
                    &[Original {
                        name: "mimeapps.list",
                        contents: Some("x"),
                    }],
                )
                .unwrap()
                .id
            })
            .collect();

        prune(root.path(), 2).unwrap();

        let kept: Vec<String> = list(root.path())
            .unwrap()
            .into_iter()
            .map(|backup| backup.id)
            .collect();
        assert_eq!(kept, [ids[3].clone(), ids[2].clone()]);
        assert!(find(root.path(), Some(&ids[0])).is_err());
    }
}
