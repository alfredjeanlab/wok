// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk edit` command.
//! Converted from tests/specs/cli/unit/edit.bats

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

fn create_issue_with_opts(temp: &TempDir, type_: &str, title: &str, opts: &[&str]) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title);
    for opt in opts {
        cmd.arg(opt);
    }
    cmd.arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic Edit Tests
// =============================================================================

#[test]
fn edit_title_changes_issue_title() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original title");

    wk().args(["edit", &id, "title", "New title"]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"))
        .stdout(predicate::str::contains("Original title").not());
}

// =============================================================================
// Type Edit Tests (parameterized)
// =============================================================================

#[parameterized(
    task_to_bug = { "task", "bug", "[bug]" },
    task_to_feature = { "task", "feature", "[feature]" },
    task_to_idea = { "task", "idea", "[idea]" },
    idea_to_task = { "idea", "task", "[task]" },
)]
fn edit_type_changes_issue_type(from_type: &str, to_type: &str, expected_tag: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, from_type, "Test issue");

    wk().args(["edit", &id, "type", to_type]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_tag));
}

// =============================================================================
// Sequential Edit Tests
// =============================================================================

#[test]
fn edit_both_title_and_type_sequentially() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original");

    wk().args(["edit", &id, "title", "Updated"]).current_dir(temp.path()).assert().success();

    wk().args(["edit", &id, "type", "bug"]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"))
        .stdout(predicate::str::contains("[bug]"));
}

// =============================================================================
// Event Logging Tests
// =============================================================================

#[test]
fn edit_logs_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");

    wk().args(["edit", &id, "title", "New title"]).current_dir(temp.path()).assert().success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("edited"));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn edit_nonexistent_issue_fails() {
    let temp = init_temp();
    wk().args(["edit", "test-nonexistent", "title", "New"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn edit_with_invalid_type_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");

    wk().args(["edit", &id, "type", "bogus"]).current_dir(temp.path()).assert().failure();
}

#[test]
fn edit_requires_issue_id() {
    let temp = init_temp();
    wk().args(["edit", "title", "New"]).current_dir(temp.path()).assert().failure();
}

// =============================================================================
// Field Preservation Tests
// =============================================================================

#[test]
fn edit_preserves_other_fields() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Test task", &["--label", "mylabel"]);

    wk().args(["edit", &id, "title", "New title"]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"))
        .stdout(predicate::str::contains("mylabel"));
}

// =============================================================================
// Hidden Flag Variant Tests
// =============================================================================

#[test]
fn edit_title_flag_updates_title() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original title");

    wk().args(["edit", &id, "--title", "Updated title"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated title"));
}

#[test]
fn edit_description_flag_updates_description() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    wk().args(["edit", &id, "--description", "New description"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New description"));
}

#[test]
fn edit_type_flag_updates_type() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    wk().args(["edit", &id, "--type", "bug"]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[bug]"));
}

#[test]
fn edit_assignee_flag_updates_assignee() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    wk().args(["edit", &id, "--assignee", "alice"]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));
}
