use std::process::{Command, Stdio};

use super::constants::{
    APP_NAME, BODY_ENV, COUNT_ENV, COUNT_FIELD, DESKTOP_ENTRY_HINT, FROM_FIELD, ICON_ENV,
    ICON_FLAG, MIME_FIELD, NO_APPLICATION, PROGRAM, SHELL, SHELL_SCRIPT_FLAG, SHOWN, SUMMARY_ENV,
    TO_FIELD, TRANSIENT_ENV, TRANSIENT_FLAG,
};
use super::types::Alert;
use crate::config::Notification;
use crate::history::Event;
use crate::output;

pub fn send(events: &[Event], settings: &Notification) {
    if events.is_empty() {
        return;
    }
    let alert = compose(events, settings);
    let Some(script) = settings.command.as_deref() else {
        run(notify_send(&alert, settings), false);
        return;
    };
    run(shell(script, &alert, settings), true);
}

pub fn compose(events: &[Event], settings: &Notification) -> Alert {
    let count = events.len();
    let mut lines: Vec<String> = events
        .iter()
        .take(SHOWN)
        .map(|event| line(&settings.line, event))
        .collect();
    if count > SHOWN {
        lines.push(format!("and {} more (mmg log)", count - SHOWN));
    }
    Alert {
        summary: settings.summary.replace(COUNT_FIELD, &count.to_string()),
        body: lines.join("\n"),
        count,
    }
}

fn line(template: &str, event: &Event) -> String {
    let from = event.from.as_deref().unwrap_or(NO_APPLICATION);
    [
        (MIME_FIELD, event.mime.as_str()),
        (FROM_FIELD, from),
        (TO_FIELD, event.to.as_str()),
    ]
    .iter()
    .fold(template.to_string(), |text, (field, value)| {
        text.replace(field, value)
    })
}

fn notify_send(alert: &Alert, settings: &Notification) -> Command {
    let mut command = Command::new(PROGRAM);
    command.args([APP_NAME, DESKTOP_ENTRY_HINT]);
    if !settings.icon.is_empty() {
        command.arg(format!("{ICON_FLAG}={}", settings.icon));
    }
    if settings.transient {
        command.arg(TRANSIENT_FLAG);
    }
    command.args(["--", &alert.summary, &alert.body]);
    command
}

fn shell(script: &str, alert: &Alert, settings: &Notification) -> Command {
    let mut command = Command::new(SHELL);
    command
        .args([SHELL_SCRIPT_FLAG, script])
        .env(SUMMARY_ENV, &alert.summary)
        .env(BODY_ENV, &alert.body)
        .env(COUNT_ENV, alert.count.to_string())
        .env(ICON_ENV, &settings.icon)
        .env(TRANSIENT_ENV, settings.transient.to_string());
    command
}

fn run(mut command: Command, custom: bool) {
    command.stdin(Stdio::null()).stdout(Stdio::null());
    if !custom {
        command.stderr(Stdio::null());
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            let program = command.get_program().to_string_lossy().into_owned();
            report(custom, format!("alert: {program} could not run: {err}"));
            return;
        }
    };
    std::thread::spawn(move || {
        let Ok(status) = child.wait() else {
            return;
        };
        if custom && !status.success() {
            output::warn(format!("alert: the notification command failed ({status})"));
        }
    });
}

fn report(custom: bool, message: String) {
    if custom {
        output::warn(message);
        return;
    }
    tracing::debug!("{message}");
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

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

    fn args(command: &Command) -> Vec<&OsStr> {
        command.get_args().collect()
    }

    #[test]
    fn a_long_list_is_cut_with_a_count() {
        let events: Vec<Event> = (0..7).map(|n| event(&format!("a/{n}"))).collect();

        let alert = compose(&events, &Notification::default());

        assert_eq!(alert.summary, "Default applications put back");
        assert!(
            alert
                .body
                .starts_with("a/0: chrome.desktop → firefox.desktop\n")
        );
        assert!(alert.body.ends_with("and 2 more (mmg log)"));
        assert_eq!(alert.body.lines().count(), 6);
        assert_eq!(alert.count, 7);
    }

    #[test]
    fn the_message_follows_the_templates() {
        let settings = Notification {
            summary: "{count} types put back".into(),
            line: "{to} keeps {mime} (was {from})".into(),
            ..Notification::default()
        };
        let mut lost = event("text/html");
        lost.from = None;

        let alert = compose(&[lost, event("image/png")], &settings);

        assert_eq!(alert.summary, "2 types put back");
        assert_eq!(
            alert.body,
            "firefox.desktop keeps text/html (was (none))\nfirefox.desktop keeps image/png (was chrome.desktop)"
        );
    }

    #[test]
    fn notify_send_gets_the_icon_and_the_transient_hint() {
        let alert = compose(&[event("text/html")], &Notification::default());
        let settings = Notification {
            transient: true,
            ..Notification::default()
        };

        let command = notify_send(&alert, &settings);

        assert_eq!(command.get_program(), "notify-send");
        assert_eq!(
            args(&command),
            [
                "--app-name=mime-guardian",
                "--hint=string:desktop-entry:mime-guardian",
                "--icon=mime-guardian",
                "--transient",
                "--",
                "Default applications put back",
                "text/html: chrome.desktop → firefox.desktop",
            ]
        );
        let plain = Notification {
            icon: String::new(),
            ..Notification::default()
        };
        assert_eq!(args(&notify_send(&alert, &plain))[2], "--");
    }

    #[test]
    fn a_custom_command_reads_the_message_from_the_environment() {
        let alert = compose(&[event("text/html")], &Notification::default());
        let settings = Notification::default();

        let command = shell("notify-send \"$MMG_SUMMARY\"", &alert, &settings);

        assert_eq!(command.get_program(), "sh");
        assert_eq!(args(&command), ["-c", "notify-send \"$MMG_SUMMARY\""]);
        let envs: Vec<(&OsStr, Option<&OsStr>)> = command.get_envs().collect();
        for (name, value) in [
            ("MMG_SUMMARY", "Default applications put back"),
            ("MMG_BODY", "text/html: chrome.desktop → firefox.desktop"),
            ("MMG_COUNT", "1"),
            ("MMG_ICON", "mime-guardian"),
            ("MMG_TRANSIENT", "false"),
        ] {
            assert!(
                envs.contains(&(OsStr::new(name), Some(OsStr::new(value)))),
                "{name}"
            );
        }
    }
}
