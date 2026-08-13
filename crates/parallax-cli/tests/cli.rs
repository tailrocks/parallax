#![expect(
    clippy::expect_used,
    reason = "integration tests fail loud on fixture setup"
)]

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

fn bin() -> Command {
    Command::cargo_bin("parallax").expect("parallax bin")
}

#[test]
fn help_exits_0() {
    bin().arg("--help").assert().success();
}

#[test]
fn unknown_command_exits_2() {
    bin().arg("nope").assert().failure().code(2);
}

#[test]
fn metrics_run_alias_is_rejected() {
    bin()
        .args(["metrics", "--run", "x"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unexpected argument").or(predicate::str::contains("run")),
        );
}

#[test]
fn prune_dry_run_leaves_data_dir_bytes_unchanged() {
    let home = TempDir::new().expect("home");
    let data = home.path().join(".parallax");
    std::fs::create_dir_all(&data).expect("data");
    let marker = data.join("marker");
    std::fs::write(&marker, b"keep").expect("marker");
    let before = std::fs::read(&marker).expect("before");
    bin()
        .env("HOME", home.path())
        .args(["prune", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("plan_id").or(predicate::str::contains("items")));
    assert_eq!(std::fs::read(&marker).expect("after"), before);
}

#[test]
fn prune_execute_without_yes_does_not_delete() {
    let home = TempDir::new().expect("home");
    let data = home.path().join(".parallax");
    std::fs::create_dir_all(data.join("spool")).expect("spool");
    let marker = data.join("marker");
    std::fs::write(&marker, b"keep").expect("marker");
    bin()
        .env("HOME", home.path())
        .args(["prune", "--execute"])
        .assert()
        .success();
    assert_eq!(std::fs::read(&marker).expect("after"), b"keep");
}

#[test]
fn prune_execute_yes_emits_json_report() {
    let home = TempDir::new().expect("home");
    std::fs::create_dir_all(home.path().join(".parallax")).expect("data");
    bin()
        .env("HOME", home.path())
        .args(["prune", "--execute", "--yes", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("plan_id"));
}
