// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for issue state machine transitions.
//! Converted from tests/specs/cli/integration/state_machine.bats
//!
//! Tests verifying issue state transitions (todo, in_progress, done, closed).

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::super::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_status(temp: &TempDir, id: &str) -> String {
    let output = wk()
        .arg("show")
        .arg(id)
        .current_dir(temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse "Status: todo" -> "todo"
    stdout
        .lines()
        .find(|line| line.starts_with("Status:"))
        .map(|line| line.split_whitespace().nth(1).unwrap_or(""))
        .unwrap_or("")
        .to_string()
}

// =============================================================================
// Valid Transitions
// =============================================================================

#[test]
fn todo_to_in_progress_via_start() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    assert_eq!(get_status(&temp, &id), "todo");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "in_progress");
}

#[test]
fn todo_to_closed_via_close_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    assert_eq!(get_status(&temp, &id), "todo");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("won't fix")
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "closed");
}

#[test]
fn todo_to_done_via_done_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    assert_eq!(get_status(&temp, &id), "todo");

    wk().arg("done")
        .arg(&id)
        .arg("--reason")
        .arg("already completed")
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "done");
}

#[test]
fn in_progress_to_todo_via_reopen() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "in_progress");

    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "todo");
}

#[test]
fn in_progress_to_done_via_done() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "in_progress");

    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "done");
}

#[test]
fn in_progress_to_closed_via_close_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "in_progress");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("abandoned")
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "closed");
}

#[test]
fn done_to_todo_via_reopen_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "done");

    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg("regression")
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "todo");
}

#[test]
fn closed_to_todo_via_reopen_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "closed");

    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg("not duplicate")
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "todo");
}

// =============================================================================
// Permissive Transitions (lenient state machine)
// =============================================================================

#[test]
fn reopen_from_todo_is_idempotent() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "todo");
}

#[test]
fn cannot_done_from_todo_without_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[parameterized(
    from_done = { "done" },
    from_closed = { "closed" },
)]
fn start_from_terminal_state_succeeds(terminal_state: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    // Move to terminal state
    if terminal_state == "done" {
        wk().arg("start")
            .arg(&id)
            .current_dir(temp.path())
            .assert()
            .success();
        wk().arg("done")
            .arg(&id)
            .current_dir(temp.path())
            .assert()
            .success();
    } else {
        wk().arg("close")
            .arg(&id)
            .arg("--reason")
            .arg("skip")
            .current_dir(temp.path())
            .assert()
            .success();
    }

    // Start should succeed (permissive transitions)
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "in_progress");
}

#[test]
fn done_from_closed_succeeds_with_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("skip")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&id)
        .arg("--reason")
        .arg("actually completed")
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "done");
}

// =============================================================================
// Reason Requirements
// =============================================================================

#[test]
fn close_requires_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("close")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn reopen_requires_reason_from_done() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Multiple Transitions
// =============================================================================

#[test]
fn can_cycle_through_states() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SM Test");

    // todo -> in_progress
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // in_progress -> todo
    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // todo -> in_progress
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // in_progress -> done
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // done -> todo
    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg("more work")
        .current_dir(temp.path())
        .assert()
        .success();

    // todo -> in_progress
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // in_progress -> done (final)
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    assert_eq!(get_status(&temp, &id), "done");
}
