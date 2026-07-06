use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_nexus_help() {
    Command::cargo_bin("nexus")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Nexus Agent"));
}

#[test]
fn test_nexus_version() {
    Command::cargo_bin("nexus")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("nexus"));
}

#[test]
fn test_nexus_providers_empty() {
    Command::cargo_bin("nexus")
        .unwrap()
        .arg("providers")
        .assert()
        .success();
}
