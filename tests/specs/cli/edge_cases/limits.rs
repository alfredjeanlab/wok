// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for input limit handling.
//! Converted from tests/specs/cli/edge_cases/limits.bats
//!
//! Input limit tests based on REQUIREMENTS.md:
//! - Issue titles: auto-truncated at 120 characters (with ellipsis)
//! - Note content: max 200,000 characters
//! - Label names: max 100 characters
//! - Reason text: max 500 characters

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::super::common::*;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");
    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Title limits (auto-truncation at 120 characters)
// =============================================================================

#[test]
fn new_accepts_title_at_120_character_limit_without_truncation() {
    let temp = init_temp();
    let title: String = "x".repeat(120);
    let id = create_issue(&temp, "task", &title);

    // Title should be preserved exactly as given (no ellipsis)
    wk().arg("show")
        .arg(&id)
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("...\"").not());
}

#[test]
fn new_truncates_title_exceeding_120_characters() {
    let temp = init_temp();
    let title: String = "x".repeat(501);
    let id = create_issue(&temp, "task", &title);

    // Title should be truncated with ellipsis, full content in description
    wk().arg("show")
        .arg(&id)
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("...\""));
}

#[test]
fn edit_accepts_title_at_120_character_limit_without_truncation() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original title");
    let title: String = "x".repeat(120);

    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg(&title)
        .current_dir(temp.path())
        .assert()
        .success();

    // Title should be preserved exactly as given (no ellipsis)
    wk().arg("show")
        .arg(&id)
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("...\"").not());
}

#[test]
fn edit_normalizes_title_exceeding_120_characters() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original title");
    let title: String = "x".repeat(121);

    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg(&title)
        .current_dir(temp.path())
        .assert()
        .success();

    // Title should be truncated with ellipsis
    wk().arg("show")
        .arg(&id)
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("...\""));
}

// =============================================================================
// Note limits
// =============================================================================
// Exact 200K boundary is tested in validate_tests.rs unit tests.
// Integration tests use 100K to stay within Linux ARG_MAX.

#[test]
fn note_accepts_large_content() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");
    let content: String = "a".repeat(100000);

    wk().arg("note")
        .arg(&id)
        .arg(&content)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn new_note_accepts_large_content() {
    let temp = init_temp();
    let content: String = "a".repeat(100000);

    wk().arg("new")
        .arg("task")
        .arg("Test task")
        .arg("--note")
        .arg(&content)
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Label limits (100 characters)
// =============================================================================

#[test]
fn label_accepts_name_at_100_character_limit() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");
    let label: String = "a".repeat(100);

    wk().arg("label")
        .arg(&id)
        .arg(&label)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn label_rejects_name_exceeding_100_characters() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");
    let label: String = "a".repeat(101);

    wk().arg("label")
        .arg(&id)
        .arg(&label)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("100"));
}

#[test]
fn new_label_accepts_name_at_100_character_limit() {
    let temp = init_temp();
    let label: String = "a".repeat(100);

    wk().arg("new")
        .arg("task")
        .arg("Test task")
        .arg("--label")
        .arg(&label)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn new_label_rejects_name_exceeding_100_characters() {
    let temp = init_temp();
    let label: String = "a".repeat(101);

    wk().arg("new")
        .arg("task")
        .arg("Test task")
        .arg("--label")
        .arg(&label)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("100"));
}

// =============================================================================
// Reason limits (500 characters)
// =============================================================================

#[test]
fn close_accepts_reason_at_500_character_limit() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");
    let reason: String = "a".repeat(500);

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg(&reason)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn close_rejects_reason_exceeding_500_characters() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");
    let reason: String = "a".repeat(501);

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg(&reason)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("500"));
}

#[test]
fn reopen_accepts_reason_at_500_character_limit() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");

    // Must be done before reopening
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

    let reason: String = "a".repeat(500);
    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg(&reason)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn reopen_rejects_reason_exceeding_500_characters() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test task");

    // Must be done before reopening
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

    let reason: String = "a".repeat(501);
    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg(&reason)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("500"));
}
