// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for import edge cases.
//! Converted from tests/specs/cli/edge_cases/import_edge.bats
//!
//! Tests verifying import command behavior with various input formats and edge cases.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::super::common::*;
use std::fs;
use yare::parameterized;

#[test]
fn import_empty_file_succeeds() {
    let temp = init_temp();
    let empty_file = temp.path().join("empty.jsonl");
    fs::write(&empty_file, "").unwrap();

    wk().arg("import")
        .arg(&empty_file)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn import_handles_multiple_issues() {
    let temp = init_temp();
    let multi_file = temp.path().join("multi.jsonl");
    let content = r#"{"id":"test-m1","issue_type":"task","title":"Task 1","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-m2","issue_type":"task","title":"Task 2","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-m3","issue_type":"task","title":"Task 3","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#;
    fs::write(&multi_file, content).unwrap();

    wk().arg("import")
        .arg(&multi_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test-m1"))
        .stdout(predicate::str::contains("test-m2"))
        .stdout(predicate::str::contains("test-m3"));
}

#[test]
fn import_preserves_labels() {
    let temp = init_temp();
    let labeled_file = temp.path().join("labeled.jsonl");
    let content = r#"{"id":"test-labels","issue_type":"task","title":"Labeled task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["project:auth","urgent"],"notes":[],"deps":[],"events":[]}"#;
    fs::write(&labeled_file, content).unwrap();

    wk().arg("import")
        .arg(&labeled_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg("test-labels")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth"))
        .stdout(predicate::str::contains("urgent"));
}

#[test]
fn import_preserves_notes() {
    let temp = init_temp();
    let noted_file = temp.path().join("noted.jsonl");
    let content = r#"{"id":"test-notes","issue_type":"task","title":"Noted task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[{"id":1,"issue_id":"test-notes","status":"todo","content":"A note","created_at":"2024-01-01T00:00:00Z"}],"deps":[],"events":[]}"#;
    fs::write(&noted_file, content).unwrap();

    wk().arg("import")
        .arg(&noted_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg("test-notes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("A note"));
}

#[test]
fn import_idempotent_running_twice_produces_same_result() {
    let temp = init_temp();
    let idem_file = temp.path().join("idem.jsonl");
    let content = r#"{"id":"test-idem","issue_type":"task","title":"Idempotent","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#;
    fs::write(&idem_file, content).unwrap();

    // First import
    wk().arg("import")
        .arg(&idem_file)
        .current_dir(temp.path())
        .assert()
        .success();

    // Second import should also succeed
    wk().arg("import")
        .arg(&idem_file)
        .current_dir(temp.path())
        .assert()
        .success();

    // Count should still be 1
    let output = wk()
        .arg("list")
        .arg("--all")
        .current_dir(temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("test-idem").count();
    assert_eq!(count, 1, "Should have exactly one test-idem issue");
}

#[test]
fn import_preserves_dependencies() {
    let temp = init_temp();
    let deps_file = temp.path().join("deps.jsonl");
    let content = r#"{"id":"test-blocker","issue_type":"task","title":"Blocker task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-blocked","issue_type":"task","title":"Blocked task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[{"from_id":"test-blocker","to_id":"test-blocked","relation":"blocks","created_at":"2024-01-01T00:00:00Z"}],"events":[]}"#;
    fs::write(&deps_file, content).unwrap();

    wk().arg("import")
        .arg(&deps_file)
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify the blocked issue shows as blocked
    wk().arg("show")
        .arg("test-blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test-blocker"));
}

#[parameterized(
    feature = { "feature", "[feature]" },
    task = { "task", "[task]" },
    bug = { "bug", "[bug]" },
)]
fn import_handles_issue_type(type_name: &str, expected_output: &str) {
    let temp = init_temp();
    let types_file = temp.path().join("types.jsonl");
    let content = format!(
        r#"{{"id":"test-{type_name}","issue_type":"{type_name}","title":"A {type_name}","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}}"#,
    );
    fs::write(&types_file, content).unwrap();

    wk().arg("import")
        .arg(&types_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(format!("test-{type_name}"))
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_output));
}

#[parameterized(
    todo = { "todo", "Status: todo" },
    in_progress = { "in_progress", "Status: in_progress" },
    done = { "done", "Status: done" },
    closed = { "closed", "Status: closed" },
)]
fn import_handles_status(status: &str, expected_output: &str) {
    let temp = init_temp();
    let statuses_file = temp.path().join("statuses.jsonl");
    // Use a unique ID based on status name
    let id = format!("test-{}", status.replace('_', ""));
    let content = format!(
        r#"{{"id":"{id}","issue_type":"task","title":"Status issue","status":"{status}","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}}"#,
    );
    fs::write(&statuses_file, content).unwrap();

    wk().arg("import")
        .arg(&statuses_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_output));
}

#[test]
fn import_skips_empty_lines() {
    let temp = init_temp();
    let withempty_file = temp.path().join("withempty.jsonl");
    let content = r#"{"id":"test-skip1","issue_type":"task","title":"Task 1","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}

{"id":"test-skip2","issue_type":"task","title":"Task 2","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#;
    fs::write(&withempty_file, content).unwrap();

    wk().arg("import")
        .arg(&withempty_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg("test-skip1")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg("test-skip2")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[parameterized(
    open_to_todo = { "open", "todo", "bd-open" },
    in_progress_to_in_progress = { "in_progress", "in_progress", "bd-inprog" },
    closed_to_done = { "closed", "done", "bd-closed" },
)]
fn import_beads_format_converts_status(beads_status: &str, expected_status: &str, id: &str) {
    let temp = init_temp();
    let beads_file = temp.path().join("beads_status.jsonl");
    let content = format!(
        r#"{{"id":"{id}","title":"Issue","status":"{beads_status}","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}}"#,
    );
    fs::write(&beads_file, content).unwrap();

    wk().arg("import")
        .arg("--format")
        .arg("bd")
        .arg(&beads_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Status: {expected_status}"
        )));
}

#[parameterized(
    task = { "task", "[task]", "bd-task" },
    bug = { "bug", "[bug]", "bd-bug" },
    feature = { "feature", "[feature]", "bd-feature" },
    epic = { "epic", "[epic]", "bd-epic" },
)]
fn import_beads_format_converts_types(type_name: &str, expected_output: &str, id: &str) {
    let temp = init_temp();
    let beads_file = temp.path().join("beads_types.jsonl");
    let content = format!(
        r#"{{"id":"{id}","title":"A {type_name}","status":"open","priority":2,"issue_type":"{type_name}","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}}"#,
    );
    fs::write(&beads_file, content).unwrap();

    wk().arg("import")
        .arg("--format")
        .arg("bd")
        .arg(&beads_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_output));
}

#[test]
fn import_beads_format_preserves_labels() {
    let temp = init_temp();
    let beads_file = temp.path().join("beads_labels.jsonl");
    let content = r#"{"id":"bd-labels","title":"Labeled issue","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent","area:backend"]}"#;
    fs::write(&beads_file, content).unwrap();

    wk().arg("import")
        .arg("--format")
        .arg("bd")
        .arg(&beads_file)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg("bd-labels")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"))
        .stdout(predicate::str::contains("area:backend"));
}
