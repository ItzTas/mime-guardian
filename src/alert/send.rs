use std::process::{Command, Stdio};

use super::constants::{APP_NAME, ICON, PROGRAM, SHOWN, SUMMARY};
use crate::history::Event;

pub fn send(events: &[Event]) {
    if events.is_empty() {
        return;
    }
    let status = Command::new(PROGRAM)
        .args([APP_NAME, ICON, SUMMARY, &body(events)])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if let Err(err) = status {
        tracing::debug!("alert: {PROGRAM} could not run: {err}");
    }
}

pub fn body(events: &[Event]) -> String {
    let mut lines: Vec<String> = events
        .iter()
        .take(SHOWN)
        .map(|event| {
            let from = event.from.as_deref().unwrap_or("(none)");
            format!("{}: {from} → {}", event.mime, event.to)
        })
        .collect();
    if events.len() > SHOWN {
        lines.push(format!("and {} more (mmg log)", events.len() - SHOWN));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::Trigger;

    fn event(mime: &str) -> Event {
        Event {
            at: 0,
            trigger: Trigger::Watch,
            mime: mime.to_string(),
            from: Some("chrome.desktop".into()),
            to: "firefox.desktop".into(),
            overridden: true,
            files: vec![],
            backup: None,
        }
    }

    #[test]
    fn a_long_list_is_cut_with_a_count() {
        let events: Vec<Event> = (0..7).map(|n| event(&format!("a/{n}"))).collect();

        let body = body(&events);

        assert!(body.starts_with("a/0: chrome.desktop → firefox.desktop\n"));
        assert!(body.ends_with("and 2 more (mmg log)"));
        assert_eq!(body.lines().count(), 6);
    }
}
