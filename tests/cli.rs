use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn mmg(home: &Path) -> Command {
    let mut command = Command::cargo_bin("mmg").unwrap();
    command.env_clear().env("HOME", home);
    command
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
