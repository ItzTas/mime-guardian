use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const TOOL: &str = "defaults-tool";

struct Home {
    dir: TempDir,
}

impl Home {
    fn new() -> Self {
        let home = Self {
            dir: tempfile::tempdir().unwrap(),
        };
        fs::create_dir_all(home.config_dir().join("mime-guardian")).unwrap();
        fs::create_dir_all(home.bin_dir()).unwrap();
        home
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn config_dir(&self) -> PathBuf {
        self.path().join(".config")
    }

    fn bin_dir(&self) -> PathBuf {
        self.path().join("bin")
    }

    fn mimeapps(&self) -> PathBuf {
        self.config_dir().join("mimeapps.list")
    }

    fn kde_mimeapps(&self) -> PathBuf {
        self.config_dir().join("kde-mimeapps.list")
    }

    fn config(&self) -> PathBuf {
        self.config_dir().join("mime-guardian").join("config.toml")
    }

    fn received(&self) -> PathBuf {
        self.path().join("received")
    }

    fn pauses(&self) -> PathBuf {
        self.path().join(".local/state/mime-guardian/pauses")
    }

    fn tool(&self, body: &str) {
        let script = self.bin_dir().join(TOOL);
        fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn rewriting_tool(&self, file: &Path, contents: &str) {
        self.tool(&format!(
            "echo \"$@\" > '{}'\nprintf '{contents}' > '{}'",
            self.received().display(),
            file.display()
        ));
    }

    fn mmg(&self) -> Command {
        let mut command = Command::cargo_bin("mmg").unwrap();
        command
            .env_clear()
            .env("HOME", self.path())
            .env("PATH", self.bin_dir());
        command
    }
}

#[test]
fn the_defaults_a_program_changes_are_kept_in_the_configuration() {
    let home = Home::new();
    fs::write(
        home.mimeapps(),
        "[Default Applications]\ntext/html=chrome.desktop;\ntext/plain=nvim.desktop;\nimage/png=loupe.desktop;\n",
    )
    .unwrap();
    fs::write(
        home.config(),
        "# mine\n[defaults]\n\"text/html\" = \"chrome.desktop\"\n\"text/plain\" = \"nvim.desktop\"\n\"image/png\" = \"loupe.desktop\"\n",
    )
    .unwrap();
    home.rewriting_tool(
        &home.mimeapps(),
        "[Default Applications]\\ntext/html=firefox.desktop;\\nimage/png=loupe.desktop;\\nimage/*=feh.desktop;\\n",
    );

    home.mmg()
        .args([
            "adopt",
            TOOL,
            "set",
            "--no-op",
            "text/html",
            "firefox.desktop",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("chrome.desktop → firefox.desktop"))
        .stdout(predicate::str::contains("image/*"));

    assert_eq!(
        fs::read_to_string(home.received()).unwrap(),
        "set --no-op text/html firefox.desktop\n"
    );
    assert_eq!(
        fs::read_to_string(home.config()).unwrap(),
        "# mine\n[defaults]\n\"text/html\" = \"firefox.desktop\"\n\"image/png\" = \"loupe.desktop\"\n\"image/*\" = \"feh.desktop\"\n"
    );
}

#[test]
fn the_desktop_file_decides_which_change_counts() {
    let home = Home::new();
    fs::write(
        home.kde_mimeapps(),
        "[Default Applications]\ntext/html=chrome.desktop;\n",
    )
    .unwrap();
    let config = "[defaults]\n\"text/html\" = \"chrome.desktop\"\n";
    fs::write(home.config(), config).unwrap();
    home.rewriting_tool(
        &home.mimeapps(),
        "[Default Applications]\\ntext/html=firefox.desktop;\\n",
    );

    home.mmg()
        .env("XDG_CURRENT_DESKTOP", "KDE")
        .args(["adopt", TOOL])
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read_to_string(home.config()).unwrap(), config);

    home.rewriting_tool(
        &home.kde_mimeapps(),
        "[Default Applications]\\ntext/html=firefox.desktop;\\n",
    );

    home.mmg()
        .env("XDG_CURRENT_DESKTOP", "KDE")
        .args(["adopt", TOOL])
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(home.config()).unwrap(),
        "[defaults]\n\"text/html\" = \"firefox.desktop\"\n"
    );
}

#[test]
fn a_run_that_changes_nothing_leaves_the_configuration_alone() {
    let home = Home::new();
    let config = "[defaults]\n\"text/html\" = \"chrome.desktop\"\n";
    fs::write(home.config(), config).unwrap();
    home.tool("echo opened");

    home.mmg()
        .args(["adopt", TOOL, "open", "https://example.org"])
        .assert()
        .success()
        .stdout("opened\n");

    assert_eq!(fs::read_to_string(home.config()).unwrap(), config);
}

#[test]
fn a_failing_program_fails_and_keeps_nothing() {
    let home = Home::new();
    let config = "[defaults]\n\"text/html\" = \"chrome.desktop\"\n";
    fs::write(home.config(), config).unwrap();
    home.tool(&format!(
        "printf '[Default Applications]\\ntext/html=firefox.desktop;\\n' > '{}'\nexit 3",
        home.mimeapps().display()
    ));

    home.mmg()
        .args(["adopt", TOOL])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nothing was kept"));

    assert_eq!(fs::read_to_string(home.config()).unwrap(), config);
}

#[test]
fn a_missing_program_is_named() {
    let home = Home::new();

    home.mmg()
        .args(["adopt", "no-such-tool"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to run no-such-tool"));
}

#[test]
fn without_a_configuration_the_change_is_only_reported() {
    let home = Home::new();
    home.rewriting_tool(
        &home.mimeapps(),
        "[Default Applications]\\ntext/html=firefox.desktop;\\n",
    );

    home.mmg()
        .args(["adopt", TOOL])
        .assert()
        .success()
        .stdout(predicate::str::contains("mmg init"));

    assert!(!home.config().exists());
}

#[test]
fn the_watch_stays_paused_while_the_program_runs() {
    let home = Home::new();
    home.tool(&format!(
        "for pause in '{}'/*; do echo \"$pause\"; done > '{}'",
        home.pauses().display(),
        home.received().display()
    ));

    home.mmg().args(["adopt", TOOL]).assert().success();

    let held = fs::read_to_string(home.received()).unwrap();
    assert!(
        held.trim_end()
            .starts_with(&home.pauses().display().to_string())
    );
    assert!(!held.contains('*'));
    assert_eq!(fs::read_dir(home.pauses()).unwrap().count(), 0);
}
