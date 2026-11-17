use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(15);

struct Session {
    root: tempfile::TempDir,
}

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Session {
    fn new() -> Self {
        let session = Self {
            root: tempfile::tempdir().unwrap(),
        };
        for app in ["firefox.desktop", "chrome.desktop"] {
            write(
                &session.path("data/applications").join(app),
                "[Desktop Entry]\nType=Application\nMimeType=text/html;\n",
            );
        }
        write(
            &session.path("config/mime-guardian/config.toml"),
            "[defaults]\n\"text/html\" = \"firefox.desktop\"\n\n[guard]\nnotify = false\n",
        );
        session
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.path().join(relative)
    }

    fn watch(&self) -> Running {
        self.watch_with(&[])
    }

    fn watch_with(&self, vars: &[(&str, &str)]) -> Running {
        let child = Command::new(assert_cmd::cargo::cargo_bin("mmg"))
            .arg("watch")
            .env_clear()
            .envs(vars.iter().copied())
            .env("HOME", self.root.path())
            .env("XDG_CONFIG_HOME", self.path("config"))
            .env("XDG_CONFIG_DIRS", self.path("etc"))
            .env("XDG_DATA_HOME", self.path("data"))
            .env("XDG_DATA_DIRS", self.path("sys"))
            .env("XDG_STATE_HOME", self.path("state"))
            .env("XDG_RUNTIME_DIR", self.path("run"))
            .env("XDG_CURRENT_DESKTOP", "KDE")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Running(child)
    }
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn wait_until(path: &Path, wanted: &str) {
    let start = Instant::now();
    while start.elapsed() < TIMEOUT {
        if read(path).contains(wanted) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "{} never held {wanted:?}; it holds {:?}",
        path.display(),
        read(path)
    );
}

const INTRUDER: &str = "[Default Applications]\ntext/html=chrome.desktop;\n";

#[test]
fn an_intruder_is_put_back() {
    let session = Session::new();
    let _watch = session.watch();
    let generic = session.path("config/mimeapps.list");
    let desktop = session.path("config/kde-mimeapps.list");

    wait_until(&generic, "text/html=firefox.desktop;");

    write(&desktop, INTRUDER);
    wait_until(&desktop, "text/html=firefox.desktop;");

    let history = read(&session.path("state/mime-guardian/history.jsonl"));
    assert!(history.contains("\"trigger\":\"watch\""));
    assert!(history.contains("\"from\":\"chrome.desktop\""));
}

#[test]
fn a_pause_holds_the_watch_until_it_is_lifted() {
    let session = Session::new();
    let _watch = session.watch();
    let generic = session.path("config/mimeapps.list");
    wait_until(&generic, "text/html=firefox.desktop;");

    let pause = session
        .path("run/mime-guardian/pauses")
        .join(std::process::id().to_string());
    write(&pause, "held\n");
    write(&generic, INTRUDER);
    std::thread::sleep(Duration::from_millis(1_500));
    assert_eq!(read(&generic), INTRUDER);

    std::fs::remove_file(&pause).unwrap();
    wait_until(&generic, "text/html=firefox.desktop;");
}

#[test]
fn the_variable_turns_on_a_custom_notification_command() {
    let session = Session::new();
    let sent = session.path("sent");
    write(
        &session.path("config/mime-guardian/config.toml"),
        &format!(
            "[defaults]\n\"text/html\" = \"firefox.desktop\"\n\n[guard]\nnotify = false\n\n\
[notification]\nsummary = \"{{count}} put back\"\n\
command = 'printf \"%s|%s|%s\" \"$MMG_SUMMARY\" \"$MMG_BODY\" \"$MMG_TRANSIENT\" > \"{}\"'\n",
            sent.display()
        ),
    );
    let _watch = session.watch_with(&[("MMG_NOTIFY", "yes")]);
    let generic = session.path("config/mimeapps.list");
    let desktop = session.path("config/kde-mimeapps.list");
    wait_until(&generic, "text/html=firefox.desktop;");

    write(&desktop, INTRUDER);

    wait_until(
        &sent,
        "1 put back|text/html: chrome.desktop → firefox.desktop|false",
    );
}
