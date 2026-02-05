// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for lifecycle transitions (start, done, close, reopen).
//! Converted from tests/specs/cli/unit/lifecycle.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic Transitions
// =============================================================================

#[test]
fn basic_transitions_start_reopen_done() {
    let temp = init_temp();

    // start transitions todo to in_progress
    let id = create_issue(&temp, "task", "LifeBasic Test task");
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));

    // reopen transitions in_progress to todo (no reason needed)
    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));

    // start again, then done
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
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
}

// =============================================================================
// Close and Reopen Require --reason
// =============================================================================

#[test]
fn close_requires_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeReason Test close");
    wk().arg("close")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn close_with_reason_succeeds_from_todo() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeReason Test close todo");
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: closed"));
}

#[test]
fn close_with_reason_succeeds_from_in_progress() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeReason Test close inprog");
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("abandoned")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: closed"));
}

#[test]
fn reopen_requires_reason_from_done() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeReason Test reopen");
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

#[test]
fn reopen_with_reason_succeeds_from_done() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeReason Test reopen done");
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
        .arg("--reason")
        .arg("regression found")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
}

#[test]
fn reopen_with_reason_succeeds_from_closed() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeReason Test reopen closed");
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg("not actually a duplicate")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
}

// =============================================================================
// Lenient Transitions (succeed from any state)
// =============================================================================

#[test]
fn start_on_done_issue_transitions_to_in_progress() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient done-start");
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
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));
}

#[test]
fn start_on_closed_issue_transitions_to_in_progress() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient closed-start");
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));
}

#[test]
fn start_on_in_progress_issue_is_idempotent() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient inprog-start");
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn done_on_closed_issue_with_reason_transitions_to_done() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient closed-done");
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("mistake")
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
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
}

#[test]
fn done_on_done_issue_is_idempotent() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient done-done");
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
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn close_on_done_issue_transitions_to_closed() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient done-close");
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
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("actually not needed")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: closed"));
}

#[test]
fn close_on_closed_issue_is_idempotent() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient closed-close");
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("not needed")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("still not needed")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn reopen_on_todo_issue_is_idempotent() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient todo-reopen");
    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn done_from_todo_without_reason_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeLenient todo-done-noreason");
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Reason Notes Appear in Correct Sections
// =============================================================================

#[test]
fn close_reason_creates_note_in_close_reason_section() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeNotes Test close");
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("duplicate of other-123")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Close Reason:"))
        .stdout(predicate::str::contains("duplicate of other-123"));
}

#[test]
fn done_reason_from_todo_creates_note_in_summary_section() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeNotes Test done");
    wk().arg("done")
        .arg(&id)
        .arg("--reason")
        .arg("already completed upstream")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary:"))
        .stdout(predicate::str::contains("already completed upstream"));
}

#[test]
fn reopen_reason_creates_note_in_description_section() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LifeNotes Test reopen");
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
        .arg("--reason")
        .arg("regression found in v2")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Description:"))
        .stdout(predicate::str::contains("regression found in v2"));
}

// =============================================================================
// Batch Operations
// =============================================================================

#[test]
fn batch_start_transitions_multiple_from_todo_to_in_progress() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LifeBatch Task 1");
    let id2 = create_issue(&temp, "task", "LifeBatch Task 2");
    wk().arg("start")
        .arg(&id1)
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));
    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));
}

#[test]
fn batch_reopen_transitions_multiple_from_in_progress_to_todo() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LifeBatchReopen Task 1");
    let id2 = create_issue(&temp, "task", "LifeBatchReopen Task 2");
    wk().arg("start")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("reopen")
        .arg(&id1)
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
}

#[test]
fn batch_done_transitions_multiple_from_in_progress_to_done() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LifeBatchDone Task 1");
    let id2 = create_issue(&temp, "task", "LifeBatchDone Task 2");
    wk().arg("start")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id1)
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
}

#[test]
fn batch_done_with_reason_transitions_multiple_from_todo_to_done() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LifeBatchDoneReason Task 1");
    let id2 = create_issue(&temp, "task", "LifeBatchDoneReason Task 2");
    wk().arg("done")
        .arg(&id1)
        .arg(&id2)
        .arg("--reason")
        .arg("already completed")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
}

#[test]
fn batch_close_with_reason() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LifeBatchClose Task 1");
    let id2 = create_issue(&temp, "task", "LifeBatchClose Task 2");
    wk().arg("close")
        .arg(&id1)
        .arg(&id2)
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: closed"));
    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: closed"));
}

#[test]
fn batch_reopen_with_reason() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LifeBatchReopen2 Task 1");
    let id2 = create_issue(&temp, "task", "LifeBatchReopen2 Task 2");
    wk().arg("start")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("reopen")
        .arg(&id1)
        .arg(&id2)
        .arg("--reason")
        .arg("regression")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
}

#[test]
fn batch_start_with_already_started_is_idempotent() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LifeBatchFail Task 1");
    let id2 = create_issue(&temp, "task", "LifeBatchFail Task 2");
    wk().arg("start")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("start")
        .arg(&id1)
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Started 2 of 2"));
    // both should be in_progress
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));
    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));
}

// =============================================================================
// Partial Updates with Unknown IDs
// =============================================================================

#[test]
fn batch_start_with_unknown_ids_performs_partial_update() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PartialBatch Task 1");
    wk().arg("start")
        .arg(&id1)
        .arg("unknown-123")
        .arg("unknown-456")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Started 1 of 3"))
        .stdout(predicate::str::contains(
            "Unknown IDs: unknown-123, unknown-456",
        ));

    // Verify the valid issue was still transitioned
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));
}

#[test]
fn batch_start_with_mixed_unknown_and_already_started() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PartialMixed Task 1");
    let id2 = create_issue(&temp, "task", "PartialMixed Task 2");
    wk().arg("start")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success(); // Now in_progress, start again is idempotent

    wk().arg("start")
        .arg(&id1)
        .arg(&id2)
        .arg("unknown-789")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Started 2 of 3"))
        .stdout(predicate::str::contains("Unknown IDs: unknown-789"));
}

#[test]
fn batch_done_with_unknown_ids_performs_partial_update() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PartialDone Task 1");
    wk().arg("start")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id1)
        .arg("unknown-123")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Completed 1 of 2"))
        .stdout(predicate::str::contains("Unknown IDs: unknown-123"));
}

#[test]
fn batch_close_with_unknown_ids_performs_partial_update() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PartialClose Task 1");
    wk().arg("close")
        .arg(&id1)
        .arg("unknown-123")
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Closed 1 of 2"))
        .stdout(predicate::str::contains("Unknown IDs: unknown-123"));
}

#[test]
fn batch_reopen_with_unknown_ids_performs_partial_update() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PartialReopen Task 1");
    wk().arg("start")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("reopen")
        .arg(&id1)
        .arg("unknown-123")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Reopened 1 of 2"))
        .stdout(predicate::str::contains("Unknown IDs: unknown-123"));
}

#[test]
fn batch_operation_all_unknown_ids_shows_all_as_unknown() {
    let temp = init_temp();
    wk().arg("start")
        .arg("unknown-1")
        .arg("unknown-2")
        .arg("unknown-3")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Started 0 of 3"))
        .stdout(predicate::str::contains(
            "Unknown IDs: unknown-1, unknown-2, unknown-3",
        ));
}

#[test]
fn batch_operation_with_single_id_preserves_current_behavior() {
    let temp = init_temp();
    // Single unknown ID should show simple error, no summary
    wk().arg("start")
        .arg("unknown-single")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("issue not found: unknown-single"))
        .stdout(predicate::str::contains("Started 0 of 1").not());
}

// =============================================================================
// Parameterized Tests for Lenient Transitions
// =============================================================================

#[parameterized(
    start_from_todo = { "start", "todo", "in_progress", false },
    start_from_in_progress = { "start", "in_progress", "in_progress", false },
    start_from_done = { "start", "done", "in_progress", false },
    start_from_closed = { "start", "closed", "in_progress", false },
    done_from_in_progress = { "done", "in_progress", "done", false },
    done_from_done = { "done", "done", "done", false },
    reopen_from_todo = { "reopen", "todo", "todo", false },
    reopen_from_in_progress = { "reopen", "in_progress", "todo", false },
)]
fn lenient_transition(
    command: &str,
    initial_state: &str,
    expected_state: &str,
    needs_reason: bool,
) {
    let temp = init_temp();
    let id = create_issue(
        &temp,
        "task",
        &format!("Lenient {} from {}", command, initial_state),
    );

    // Set up initial state
    match initial_state {
        "todo" => {}
        "in_progress" => {
            wk().arg("start")
                .arg(&id)
                .current_dir(temp.path())
                .assert()
                .success();
        }
        "done" => {
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
        }
        "closed" => {
            wk().arg("close")
                .arg(&id)
                .arg("--reason")
                .arg("test")
                .current_dir(temp.path())
                .assert()
                .success();
        }
        _ => panic!("Unknown initial state: {}", initial_state),
    }

    // Execute the transition
    let mut cmd = wk();
    cmd.arg(command).arg(&id);
    if needs_reason {
        cmd.arg("--reason").arg("test reason");
    }
    cmd.current_dir(temp.path()).assert().success();

    // Verify the expected state
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Status: {}",
            expected_state
        )));
}
