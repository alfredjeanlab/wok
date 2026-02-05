// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs verifying examples shown in help text are accurate.
//! Converted from tests/specs/cli/unit/help_examples.bats
//!
//! These tests run example commands that should match help output patterns.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    create_issue_with_opts(temp, type_, title, &[])
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
// Init Examples
// From: "init and new examples work"
// =============================================================================

#[test]
fn init_basic_example() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn init_with_prefix_example() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();
}

// =============================================================================
// New Examples (parameterized)
// From: "init and new examples work"
// =============================================================================

#[parameterized(
    default_task = { None, "Fix login bug", "[task]" },
    explicit_task = { Some("task"), "My task title", "[task]" },
    bug_type = { Some("bug"), "Memory leak", "[bug]" },
    feature_type = { Some("feature"), "User authentication", "[feature]" },
)]
fn new_type_examples(type_: Option<&str>, title: &str, expected: &str) {
    let temp = init_temp();

    let mut cmd = wk();
    cmd.arg("new");
    if let Some(t) = type_ {
        cmd.arg(t);
    }
    cmd.arg(title);

    cmd.current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[test]
fn new_with_label_and_note_example() {
    let temp = init_temp();

    wk().arg("new")
        .arg("task")
        .arg("My task")
        .arg("--label")
        .arg("auth")
        .arg("--note")
        .arg("Initial context")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Lifecycle Examples
// From: "lifecycle and edit examples work"
// =============================================================================

#[test]
fn start_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Start test");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn done_after_start_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Done test");

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

#[test]
fn done_with_reason_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Done reason test");

    wk().arg("done")
        .arg(&id)
        .arg("--reason")
        .arg("already fixed")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn close_with_reason_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Close test");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("duplicate of another issue")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn reopen_with_reason_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Reopen test");

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
}

// =============================================================================
// Edit Examples
// From: "lifecycle and edit examples work"
// =============================================================================

#[test]
fn edit_title_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original");

    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg("new title")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn edit_type_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original");

    wk().arg("edit")
        .arg(&id)
        .arg("type")
        .arg("feature")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// List Examples
// From: "list, show, and tree examples work"
// =============================================================================

#[test]
fn list_basic_example() {
    let temp = init_temp();
    create_issue(&temp, "task", "List test");

    wk().arg("list").current_dir(temp.path()).assert().success();
}

#[test]
fn list_status_filter_example() {
    let temp = init_temp();
    create_issue(&temp, "task", "Status test");

    wk().arg("list")
        .arg("--status")
        .arg("todo")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn list_type_and_all_example() {
    let temp = init_temp();
    create_issue(&temp, "task", "Type test");

    wk().arg("list")
        .arg("--type")
        .arg("task")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn list_label_example() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "task", "Labeled", &["--label", "mylabel"]);

    wk().arg("list")
        .arg("--label")
        .arg("mylabel")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn list_all_example() {
    let temp = init_temp();
    create_issue(&temp, "task", "All test");

    wk().arg("list")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn list_blocked_example() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Blocker");
    let id2 = create_issue(&temp, "task", "Blocked");

    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Show and Tree Examples
// From: "list, show, and tree examples work"
// =============================================================================

#[test]
fn show_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Show test");

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn tree_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Tree test");

    wk().arg("tree")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Dep Examples
// From: "dep, label, note, and log examples work"
// =============================================================================

#[test]
fn dep_blocks_single_example() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Task A");
    let id2 = create_issue(&temp, "task", "Task B");

    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn dep_blocks_multiple_example() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Task A");
    let id2 = create_issue(&temp, "task", "Task B");
    let id3 = create_issue(&temp, "task", "Task C");

    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .arg(&id3)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn dep_tracks_example() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "My feature");
    let t1 = create_issue(&temp, "task", "Task 1");
    let t2 = create_issue(&temp, "task", "Task 2");

    wk().arg("dep")
        .arg(&feature)
        .arg("tracks")
        .arg(&t1)
        .arg(&t2)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn undep_example() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Task A");
    let id2 = create_issue(&temp, "task", "Task B");

    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("undep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Label Examples
// From: "dep, label, note, and log examples work"
// =============================================================================

#[test]
fn label_with_colon_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Label test");

    wk().arg("label")
        .arg(&id)
        .arg("project:auth")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn label_simple_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Label test");

    wk().arg("label")
        .arg(&id)
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn unlabel_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Unlabel test");

    wk().arg("label")
        .arg(&id)
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("unlabel")
        .arg(&id)
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Note Examples
// From: "dep, label, note, and log examples work"
// =============================================================================

#[test]
fn note_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Note test");

    wk().arg("note")
        .arg(&id)
        .arg("This is a note about the issue")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Log Examples
// From: "dep, label, note, and log examples work"
// =============================================================================

#[test]
fn log_basic_example() {
    let temp = init_temp();
    create_issue(&temp, "task", "Log test");

    wk().arg("log").current_dir(temp.path()).assert().success();
}

#[test]
fn log_for_issue_example() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Log id test");

    wk().arg("log")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn log_with_limit_example() {
    let temp = init_temp();
    create_issue(&temp, "task", "Log limit test");

    wk().arg("log")
        .arg("--limit")
        .arg("5")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Export Examples
// From: "export and help examples work"
// =============================================================================

#[test]
fn export_example() {
    let temp = init_temp();
    create_issue(&temp, "task", "Export test");

    let export_path = temp.path().join("issues.jsonl");
    wk().arg("export")
        .arg(export_path.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(export_path.exists());
}

// =============================================================================
// Help Examples (parameterized)
// From: "export and help examples work"
// =============================================================================

#[parameterized(
    help_basic = { &["help"] },
    help_subcommand = { &["help", "new"] },
    help_flag_long = { &["--help"] },
    help_flag_short = { &["-h"] },
)]
fn help_examples(args: &[&str]) {
    wk().args(args).assert().success();
}
