use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn mmg(home: &Path) -> Command {
    let mut command = Command::cargo_bin("mmg").unwrap();
    command.env_clear().env("HOME", home);
    command
}

fn session(home: &Path) -> Command {
    let mut command = mmg(home);
    for (name, relative) in [
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_CONFIG_DIRS", "etc"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_DATA_DIRS", "sys"),
        ("XDG_STATE_HOME", "state"),
    ] {
        command.env(name, home.join(relative));
    }
    command
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn a_type_is_set_pinned_and_unset() {
    let home = tempfile::tempdir().unwrap();
    let home = home.path();
    let apps = home.join("data/applications");
    std::fs::create_dir_all(&apps).unwrap();
    std::fs::write(apps.join("firefox.desktop"), "[Desktop Entry]\n").unwrap();
    let config = home.join("config/mime-guardian/config.toml");
    let mimeapps = home.join("config/mimeapps.list");

    session(home)
        .args(["set", "text/html", "firefox"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mmg init"));

    session(home).arg("init").assert().success();
    session(home)
        .args(["set", "text/html", "firefox"])
        .assert()
        .success();
    assert!(read(&config).contains("\"text/html\" = \"firefox.desktop\""));
    assert!(read(&mimeapps).contains("text/html=firefox.desktop;"));

    session(home)
        .args(["set", "image/png", "missing.desktop"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not installed"));

    session(home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to apply"));

    session(home)
        .args(["unset", "text/html"])
        .assert()
        .success();
    assert!(!read(&config).contains("text/html"));
    session(home)
        .args(["unset", "text/html"])
        .assert()
        .failure();
}

#[test]
fn no_command_prints_the_help() {
    let home = tempfile::tempdir().unwrap();

    mmg(home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: mmg"));
}

#[test]
fn every_shell_gets_a_completion_script() {
    let home = tempfile::tempdir().unwrap();

    for shell in ["bash", "zsh", "fish", "elvish", "powershell"] {
        mmg(home.path())
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(predicate::str::contains("mmg"));
    }
}

#[test]
fn the_manual_page_is_roff() {
    let home = tempfile::tempdir().unwrap();

    mmg(home.path())
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::starts_with(".ie \\n(.g"))
        .stdout(predicate::str::contains(".TH MMG"));
}
