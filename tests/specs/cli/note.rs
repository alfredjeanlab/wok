// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk note` command.
//! Converted from tests/specs/cli/unit/note.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let output = wk()
        .args(["new", type_, title, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic Note Tests
// =============================================================================

#[test]
fn note_adds_note_to_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteBasic Test task");

    wk().args(["note", &id, "My note"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn note_appears_in_show_output() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteBasic Show task");

    wk().args(["note", &id, "Important note"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Important note"));
}

#[test]
fn note_multiple_preserve_order() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteBasic Multi task");

    wk().args(["note", &id, "First"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["note", &id, "Second"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["note", &id, "Third"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First"))
        .stdout(predicate::str::contains("Second"))
        .stdout(predicate::str::contains("Third"));
}

#[test]
fn note_logs_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteBasic Log task");

    wk().args(["note", &id, "My note"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("noted"));
}

#[test]
fn note_shows_timestamp() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteBasic Timestamp task");

    wk().args(["note", &id, "Test note"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}").unwrap());
}

// =============================================================================
// Status Recording Tests (parameterized)
// =============================================================================

#[parameterized(
    todo = { "NoteStatus Todo task", &[], "todo", "Todo note" },
    in_progress = { "NoteStatus Progress task", &["start"], "in_progress", "Progress note" },
    done = { "NoteStatus Done task", &["start", "done"], "done", "Done note" },
)]
fn note_records_status(title: &str, setup_cmds: &[&str], expected_status: &str, note_text: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", title);

    // Run setup commands (start, done, etc.)
    for cmd in setup_cmds {
        wk().args([*cmd, id.as_str()])
            .current_dir(temp.path())
            .assert()
            .success();
    }

    wk().args(["note", &id, note_text])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_status))
        .stdout(predicate::str::contains(note_text));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn note_nonexistent_issue_fails() {
    let temp = init_temp();

    wk().args(["note", "test-nonexistent", "My note"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn note_requires_content() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteErr Test task");

    wk().args(["note", &id])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn note_on_closed_issue_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteErr Closed task");

    wk().args(["close", &id, "--reason", "wontfix"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["note", &id, "Should fail"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot add notes to closed issues",
        ));
}

#[test]
fn note_rejects_r_shorthand() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NoteErr Shorthand task");

    wk().args(["note", &id, "Original note"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["note", &id, "-r", "Replacement"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '-r'"));
}

// =============================================================================
// Semantic Labels Tests (parameterized)
// =============================================================================

#[parameterized(
    todo_description = { "NoteSem Todo task", &[], "Description:", "Requirements and context" },
    in_progress_progress = { "NoteSem Progress task", &["start"], "Progress:", "Working on implementation" },
    done_summary = { "NoteSem Done task", &["start", "done"], "Summary:", "Completed successfully" },
)]
fn note_semantic_labels_by_status(
    title: &str,
    setup_cmds: &[&str],
    expected_label: &str,
    note_text: &str,
) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", title);

    // Run setup commands to change status
    for cmd in setup_cmds {
        wk().args([*cmd, id.as_str()])
            .current_dir(temp.path())
            .assert()
            .success();
    }

    wk().args(["note", &id, note_text])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_label))
        .stdout(predicate::str::contains(note_text));
}
