use std::path::{Path, PathBuf};

use anyhow::Result;

use super::constants::COMMAND;
use super::types::{FileEdit, Outcome, Plan};
use crate::backup::{self, Original};
use crate::files::replace_contents;
use crate::history::{self, Event, Trigger};
use crate::mimeapps;
use crate::utils::{Xdg, now_millis};

pub fn apply(xdg: &Xdg, plan: &Plan, trigger: Trigger) -> Result<Outcome> {
    let mut outcome = Outcome::default();
    let mut ready: Vec<&FileEdit> = Vec::new();
    for edit in &plan.files {
        if mimeapps::read(&target(&edit.path))? != edit.before {
            outcome.conflicts.push(edit.path.clone());
            continue;
        }
        ready.push(edit);
    }
    if ready.is_empty() {
        return Ok(outcome);
    }

    let originals: Vec<Original> = ready
        .iter()
        .map(|edit| Original {
            name: name(&edit.path),
            contents: edit.before.as_deref(),
        })
        .collect();
    let saved = backup::save(&backup::root(xdg), &originals)?;

    for edit in &ready {
        replace_contents(COMMAND, &target(&edit.path), edit.after.as_bytes())?;
        outcome.written.push(edit.path.clone());
    }

    outcome.events = events(plan, &outcome.written, trigger, &saved.id);
    history::append(&history::path(xdg), &outcome.events)?;
    outcome.backup = Some(saved);
    Ok(outcome)
}

pub fn target(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
}

fn events(plan: &Plan, written: &[PathBuf], trigger: Trigger, backup: &str) -> Vec<Event> {
    let at = now_millis();
    plan.drifts
        .iter()
        .filter_map(|drift| {
            let files: Vec<String> = drift
                .files
                .iter()
                .filter(|path| written.contains(path))
                .map(|path| path.display().to_string())
                .collect();
            if files.is_empty() {
                return None;
            }
            Some(Event {
                at,
                trigger,
                mime: drift.mime.clone(),
                from: drift.previous().map(str::to_string),
                to: drift.wanted.clone(),
                overridden: drift.overridden(),
                files,
                backup: Some(backup.to_string()),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::Drift;

    fn xdg(root: &Path) -> Xdg {
        Xdg {
            config_home: root.join("config"),
            config_dirs: vec![],
            data_home: root.join("data"),
            data_dirs: vec![],
            state_home: root.join("state"),
            desktops: vec![],
        }
    }

    fn plan(path: PathBuf, before: Option<&str>) -> Plan {
        Plan {
            files: vec![FileEdit {
                path: path.clone(),
                before: before.map(str::to_string),
                after: "[Default Applications]\ntext/html=firefox.desktop;\n".into(),
            }],
            drifts: vec![Drift {
                mime: "text/html".into(),
                wanted: "firefox.desktop".into(),
                found: None,
                files: vec![path],
            }],
            skipped: vec![],
        }
    }

    #[test]
    fn a_plan_is_written_backed_up_and_logged() {
        let root = tempfile::tempdir().unwrap();
        let xdg = xdg(root.path());
        let path = xdg.config_home.join("mimeapps.list");
        std::fs::create_dir_all(&xdg.config_home).unwrap();
        std::fs::write(&path, "old\n").unwrap();

        let outcome = apply(&xdg, &plan(path.clone(), Some("old\n")), Trigger::Apply).unwrap();

        assert_eq!(outcome.written, std::slice::from_ref(&path));
        assert!(std::fs::read_to_string(&path).unwrap().contains("firefox"));
        let backup = outcome.backup.unwrap();
        assert_eq!(
            std::fs::read_to_string(backup.dir.join("mimeapps.list")).unwrap(),
            "old\n"
        );
        let events = history::read(&history::path(&xdg)).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].backup.as_deref(), Some(backup.id.as_str()));
    }

    #[test]
    fn a_file_changed_since_the_plan_is_left_alone() {
        let root = tempfile::tempdir().unwrap();
        let xdg = xdg(root.path());
        let path = xdg.config_home.join("mimeapps.list");
        std::fs::create_dir_all(&xdg.config_home).unwrap();
        std::fs::write(&path, "newer\n").unwrap();

        let outcome = apply(&xdg, &plan(path.clone(), Some("old\n")), Trigger::Watch).unwrap();

        assert_eq!(outcome.conflicts, std::slice::from_ref(&path));
        assert!(outcome.written.is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "newer\n");
    }

    #[test]
    fn a_linked_file_is_written_through_its_link() {
        let root = tempfile::tempdir().unwrap();
        let xdg = xdg(root.path());
        let dotfiles = root.path().join("dotfiles/mimeapps.list");
        std::fs::create_dir_all(dotfiles.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&xdg.config_home).unwrap();
        std::fs::write(&dotfiles, "old\n").unwrap();
        let link = xdg.config_home.join("mimeapps.list");
        std::os::unix::fs::symlink(&dotfiles, &link).unwrap();

        apply(&xdg, &plan(link.clone(), Some("old\n")), Trigger::Apply).unwrap();

        assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
        assert!(
            std::fs::read_to_string(&dotfiles)
                .unwrap()
                .contains("firefox")
        );
    }
}
