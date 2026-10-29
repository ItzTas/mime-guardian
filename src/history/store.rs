use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::constants::{COMMAND, FILE};
use super::types::Event;
use crate::files::create_parent;
use crate::utils::Xdg;

pub fn path(xdg: &Xdg) -> PathBuf {
    xdg.state_home.join(FILE)
}

pub fn append(path: &Path, events: &[Event]) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    create_parent(COMMAND, path)?;
    let mut lines = String::new();
    for event in events {
        lines.push_str(&serde_json::to_string(event)?);
        lines.push('\n');
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("{COMMAND}: failed to open {}", path.display()))?;
    file.write_all(lines.as_bytes())
        .with_context(|| format!("{COMMAND}: failed to write {}", path.display()))
}

pub fn read(path: &Path) -> Result<Vec<Event>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("{COMMAND}: failed to read {}", path.display()));
        }
    };
    Ok(text
        .lines()
        .filter_map(|line| match serde_json::from_str(line) {
            Ok(event) => Some(event),
            Err(err) => {
                tracing::warn!("{COMMAND}: skipping a damaged line: {err}");
                None
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::Trigger;

    fn event(mime: &str) -> Event {
        Event {
            at: 1,
            trigger: Trigger::Watch,
            mime: mime.to_string(),
            from: Some("chrome.desktop".into()),
            to: "firefox.desktop".into(),
            overridden: true,
            files: vec!["/c/mimeapps.list".into()],
            backup: Some("0001".into()),
        }
    }

    #[test]
    fn events_are_appended_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state/history.jsonl");

        append(&path, &[event("text/html")]).unwrap();
        append(&path, &[event("image/png")]).unwrap();
        let mut text = std::fs::read_to_string(&path).unwrap();
        text.push_str("not json\n");
        std::fs::write(&path, text).unwrap();

        let events = read(&path).unwrap();

        assert_eq!(events, [event("text/html"), event("image/png")]);
    }

    #[test]
    fn a_missing_history_is_empty() {
        let dir = tempfile::tempdir().unwrap();

        assert!(read(&dir.path().join("none.jsonl")).unwrap().is_empty());
    }
}
