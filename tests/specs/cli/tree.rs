// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk tree` command.
//! Converted from tests/specs/cli/unit/tree.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let output = wk()
        .args(["new", type_, title, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic tree display tests
// =============================================================================

#[test]
fn tree_shows_issue_and_children() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "Parent feature");
    let task = create_issue(&temp, "task", "Child task");

    wk().args(["dep", &feature, "tracks", &task])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["tree", &feature])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Parent feature"))
        .stdout(predicate::str::contains("Child task"));
}

#[test]
fn tree_shows_status_of_children() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "Parent");
    let task = create_issue(&temp, "task", "Child");

    wk().args(["dep", &feature, "tracks", &task])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["start", &task])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["tree", &feature])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("in_progress"));
}

#[test]
fn tree_shows_nested_hierarchy() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "Feature");
    let sub = create_issue(&temp, "task", "Subtask");
    let subsub = create_issue(&temp, "task", "Sub-subtask");

    wk().args(["dep", &feature, "tracks", &sub])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["dep", &sub, "tracks", &subsub])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["tree", &feature])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature"))
        .stdout(predicate::str::contains("Subtask"))
        .stdout(predicate::str::contains("Sub-subtask"));
}

#[test]
fn tree_shows_blocking_relationships() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Blocker");
    let b = create_issue(&temp, "task", "Blocked");

    wk().args(["dep", &a, "blocks", &b])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["tree", &b])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("blocked"));
}

#[test]
fn tree_with_no_children_shows_just_the_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Standalone task");

    wk().args(["tree", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Standalone task"));
}

// =============================================================================
// Error handling tests
// =============================================================================

#[test]
fn tree_nonexistent_issue_fails() {
    let temp = init_temp();

    wk().args(["tree", "test-nonexistent"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn tree_requires_issue_id() {
    let temp = init_temp();

    wk().args(["tree"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Multiple children tests
// =============================================================================

#[test]
fn tree_shows_multiple_children() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "Feature");
    let t1 = create_issue(&temp, "task", "Task 1");
    let t2 = create_issue(&temp, "task", "Task 2");
    let t3 = create_issue(&temp, "task", "Task 3");

    wk().args(["dep", &feature, "tracks", &t1, &t2, &t3])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["tree", &feature])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1"))
        .stdout(predicate::str::contains("Task 2"))
        .stdout(predicate::str::contains("Task 3"));
}
