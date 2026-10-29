use std::fs::{File, Metadata, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use anyhow::{Context, Result};

pub fn metadata(command: &str, path: &Path) -> Result<Option<Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => Ok(Some(meta)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("{command}: failed to inspect {}", path.display()))
        }
    }
}

pub fn create_parent(command: &str, path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)
        .with_context(|| format!("{command}: failed to create {}", parent.display()))
}

pub fn write_contents(
    command: &str,
    dest: &Path,
    contents: &[u8],
    mode: Option<u32>,
) -> Result<()> {
    let failed = || format!("{command}: failed to write {}", dest.display());

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    if let Some(mode) = mode {
        options.mode(mode);
    }

    let file = options.open(dest).with_context(failed)?;
    if let Some(mode) = mode {
        file.set_permissions(Permissions::from_mode(mode))
            .with_context(|| format!("{command}: failed to set the mode of {}", dest.display()))?;
    }

    persist(file, contents).with_context(failed)
}

fn persist(mut file: File, contents: &[u8]) -> std::io::Result<()> {
    file.write_all(contents)?;

    file.sync_all()
}
