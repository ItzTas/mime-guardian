use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::{COMMAND, PAUSES_DIR, PROC, RUNTIME_DIR};
use crate::utils::Xdg;

#[derive(Debug)]
pub struct Pause {
    path: PathBuf,
}

impl Pause {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Pause {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn pauses_dir(xdg: &Xdg) -> PathBuf {
    std::env::var_os(RUNTIME_DIR)
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .unwrap_or_else(|| xdg.state_home.clone())
        .join(PAUSES_DIR)
}

pub fn pause(xdg: &Xdg) -> Result<Pause> {
    hold(&pauses_dir(xdg), std::process::id())
}

pub fn hold(dir: &Path, pid: u32) -> Result<Pause> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("{COMMAND}: failed to create {}", dir.display()))?;
    let path = dir.join(pid.to_string());
    std::fs::write(&path, format!("{pid}\n"))
        .with_context(|| format!("{COMMAND}: failed to write {}", path.display()))?;
    Ok(Pause { path })
}

pub fn paused(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    let mut held = false;
    for entry in entries.flatten() {
        let path = entry.path();
        let alive = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.parse::<u32>().ok())
            .is_some_and(alive);
        if alive {
            held = true;
            continue;
        }
        let _ = std::fs::remove_file(&path);
    }
    held
}

fn alive(pid: u32) -> bool {
    Path::new(PROC).join(pid.to_string()).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_held_pause_is_seen_until_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let pauses = dir.path().join("pauses");

        let pause = hold(&pauses, std::process::id()).unwrap();
        assert!(paused(&pauses));

        drop(pause);
        assert!(!paused(&pauses));
    }

    #[test]
    fn a_dead_process_does_not_hold_the_watch() {
        let dir = tempfile::tempdir().unwrap();
        let stale = hold(dir.path(), u32::MAX).unwrap();
        let path = stale.path().to_path_buf();
        std::mem::forget(stale);

        assert!(!paused(dir.path()));
        assert!(!path.exists());
    }
}
