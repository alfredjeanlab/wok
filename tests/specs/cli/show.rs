// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk show` command.
//! Converted from tests/specs/cli/unit/show.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;

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
// Basic Display Tests
// =============================================================================

#[test]
fn show_displays_issue_details() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ShowBasic Test task");

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("[task] {}", id)))
        .stdout(predicate::str::contains("Title: ShowBasic Test task"))
        .stdout(predicate::str::contains("Status: todo"))
        .stdout(predicate::str::contains("Created:"))
        .stdout(predicate::str::contains("Updated:"));
}

#[test]
fn show_displays_single_label() {
    let temp = init_temp();
    let id = create_issue_with_opts(
        &temp,
        "task",
        "ShowBasic Labeled task",
        &["--label", "project:auth"],
    );

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth"));
}

#[test]
fn show_displays_multiple_labels() {
    let temp = init_temp();
    let id = create_issue_with_opts(
        &temp,
        "task",
        "ShowBasic Multi-labeled",
        &["--label", "label1", "--label", "label2"],
    );

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("label1"))
        .stdout(predicate::str::contains("label2"));
}

// =============================================================================
// Notes Tests
// =============================================================================

#[test]
fn show_displays_notes() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ShowNotes Test task");

    wk().args(["note", &id, "My note content"]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("My note content"));
}

#[test]
fn show_groups_notes_by_status() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ShowNotes Grouped task");

    wk().args(["note", &id, "Todo note"]).current_dir(temp.path()).assert().success();

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["note", &id, "In progress note"]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Todo note"))
        .stdout(predicate::str::contains("In progress note"));
}

// =============================================================================
// Dependency Tests
// =============================================================================

#[test]
fn show_displays_blocked_by() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "ShowDep Blocker");
    let b = create_issue(&temp, "task", "ShowDep Blocked");

    wk().args(["dep", &a, "blocks", &b]).current_dir(temp.path()).assert().success();

    wk().args(["show", &b])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked by"))
        .stdout(predicate::str::contains(&a));
}

#[test]
fn show_displays_blocks() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "ShowDep Blocker");
    let b = create_issue(&temp, "task", "ShowDep Blocked");

    wk().args(["dep", &a, "blocks", &b]).current_dir(temp.path()).assert().success();

    wk().args(["show", &a])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocks"))
        .stdout(predicate::str::contains(&b));
}

#[test]
fn show_displays_tracked_by() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "ShowDep Parent feature");
    let task = create_issue(&temp, "task", "ShowDep Child task");

    wk().args(["dep", &feature, "tracks", &task]).current_dir(temp.path()).assert().success();

    wk().args(["show", &task])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracked by"))
        .stdout(predicate::str::contains(&feature));
}

#[test]
fn show_displays_tracks() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "ShowDep Parent feature");
    let task = create_issue(&temp, "task", "ShowDep Child task");

    wk().args(["dep", &feature, "tracks", &task]).current_dir(temp.path()).assert().success();

    wk().args(["show", &feature])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracks"))
        .stdout(predicate::str::contains(&task));
}

#[test]
fn show_displays_log() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ShowDep Log task");

    wk().args(["start", &id]).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Log:"))
        .stdout(predicate::str::contains("started"));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn show_nonexistent_fails() {
    let temp = init_temp();
    wk().args(["show", "test-nonexistent"]).current_dir(temp.path()).assert().failure();
}

#[test]
fn show_requires_issue_id() {
    let temp = init_temp();
    wk().args(["show"]).current_dir(temp.path()).assert().failure();
}

#[test]
fn show_multiple_issues_separator() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "First issue");
    let id2 = create_issue(&temp, "task", "Second issue");

    wk().args(["show", &id1, &id2])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First issue"))
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("Second issue"));
}

#[test]
fn show_fails_if_any_id_invalid() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    wk().args(["show", &id, "nonexistent"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// =============================================================================
// JSON Output Tests
// =============================================================================

#[test]
fn show_json_single_issue_compact() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    let output = wk().args(["show", &id, "-o", "json"]).current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Single line (compact JSONL format)
    assert_eq!(stdout.trim().lines().count(), 1);

    // Valid JSON
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    assert!(json.get("id").is_some());
}

#[test]
fn show_json_multiple_issues_jsonl() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "First issue");
    let id2 = create_issue(&temp, "task", "Second issue");

    let output =
        wk().args(["show", &id1, &id2, "-o", "json"]).current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();

    // Two lines (one per issue)
    assert_eq!(lines.len(), 2);

    // Each line is valid JSON
    for line in lines {
        let _: serde_json::Value =
            serde_json::from_str(line).expect("Each line should be valid JSON");
    }
}
