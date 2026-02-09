// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk log` command.
//! Converted from tests/specs/cli/unit/log.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let output =
        wk().args(["new", type_, title, "-o", "id"]).current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic Log Tests
// =============================================================================

#[test]
fn log_shows_recent_activity() {
    let temp = init_temp();
    create_issue(&temp, "task", "LogBasic Task 1");
    create_issue(&temp, "task", "LogBasic Task 2");

    wk().arg("log")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));
}

#[test]
fn log_for_specific_issue_shows_its_history() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogBasic Specific task");

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));
}

#[test]
fn log_with_empty_database_succeeds() {
    let temp = init_temp();

    wk().arg("log").current_dir(temp.path()).assert().success();
}

// =============================================================================
// Lifecycle Event Tests
// =============================================================================

#[test]
fn log_shows_start_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogEvent Start task");

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("started"));
}

#[test]
fn log_shows_reopen_event_from_in_progress() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogEvent Reopen inprog task");

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["reopen", &id]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("reopened"));
}

#[test]
fn log_shows_done_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogEvent Done task");

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["done", &id]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

#[test]
fn log_shows_close_event_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogEvent Close task");

    wk().args(["close", &id, "--reason", "duplicate"]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("closed"))
        .stdout(predicate::str::contains("duplicate"));
}

#[test]
fn log_shows_reopen_event_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogEvent Reopen done task");

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["done", &id]).current_dir(temp.path()).assert().success();

    wk().args(["reopen", &id, "--reason", "regression"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("reopened"))
        .stdout(predicate::str::contains("regression"));
}

// =============================================================================
// Label and Note Event Tests
// =============================================================================

#[test]
fn log_shows_labeled_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogMeta Label task");

    wk().args(["label", &id, "mylabel"]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("labeled"));
}

#[test]
fn log_shows_noted_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogMeta Note task");

    wk().args(["note", &id, "My note"]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("noted"));
}

// =============================================================================
// Options and Error Handling Tests
// =============================================================================

#[test]
fn log_limit_restricts_output() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogOpts Limit task");

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["reopen", &id]).current_dir(temp.path()).assert().success();

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["log", "--limit", "2"]).current_dir(temp.path()).assert().success();
}

#[test]
fn log_nonexistent_issue_fails() {
    let temp = init_temp();

    wk().args(["log", "test-nonexistent"]).current_dir(temp.path()).assert().failure();
}

#[test]
fn log_rejects_short_l_flag() {
    let temp = init_temp();

    wk().args(["log", "-l", "5"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '-l'"));
}

// =============================================================================
// Parameterized Lifecycle Event Tests
// =============================================================================

#[parameterized(
    start = { "start", "started" },
    done = { "done", "done" },
)]
fn log_shows_lifecycle_event(action: &str, expected: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogParam Lifecycle task");

    // Start is required for done
    if action == "done" {
        wk().args(["start", &id]).current_dir(temp.path()).assert().success();
    }

    wk().args([action, &id]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[parameterized(
    duplicate = { "duplicate" },
    wontfix = { "wontfix" },
)]
fn log_shows_close_reason(reason: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogParam Close task");

    wk().args(["close", &id, "--reason", reason]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("closed"))
        .stdout(predicate::str::contains(reason));
}
