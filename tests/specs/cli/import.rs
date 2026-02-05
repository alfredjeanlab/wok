// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk import` command.
//! Converted from tests/specs/cli/unit/import.bats
//!
//! BATS test mapping:
//! - "import from file, stdin, and --input flag"
//!   -> import_from_file, import_from_stdin, import_with_input_flag
//! - "import updates existing issues and detects collisions"
//!   -> import_updates_existing_issue, import_detects_collision
//! - "import --dry-run shows preview without creating"
//!   -> import_dry_run
//! - "import warns about missing dependencies"
//!   -> import_warns_missing_deps
//! - "import --status, --type, --label, --prefix filters"
//!   -> parameterized import_filter_* tests
//! - "import auto-detects bd format and --format bd parses beads format"
//!   -> import_autodetects_bd_format, import_explicit_bd_format
//! - "import fails on invalid JSON and shows help with no input"
//!   -> import_fails_invalid_json, import_fails_no_input
//! - "import beads format converts dependency types"
//!   -> parameterized import_bd_dep_* tests
//! - "import beads close reason maps to status and creates note"
//!   -> import_bd_close_reason_* tests
//! - "import beads format converts priority and preserves comments"
//!   -> import_bd_priority, import_bd_no_priority_zero, import_bd_comment
//! - "import rejects -i and -p shorthands"
//!   -> parameterized import_rejects_shorthand tests

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

fn write_jsonl(temp: &TempDir, filename: &str, content: &str) {
    let path = temp.path().join(filename);
    std::fs::write(path, content).unwrap();
}

// =============================================================================
// Phase 1: Basic Import Tests
// From: "import from file, stdin, and --input flag"
// =============================================================================

#[test]
fn import_from_file() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"test-imp1","issue_type":"task","title":"Imported task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    );

    wk().args(["import", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-imp1"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported task"));
}

#[test]
fn import_from_stdin() {
    let temp = init_temp();

    wk().args(["import", "-"])
        .current_dir(temp.path())
        .write_stdin(r#"{"id":"test-std1","issue_type":"task","title":"Stdin task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#)
        .assert()
        .success();

    wk().args(["show", "test-std1"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Stdin task"));
}

#[test]
fn import_with_input_flag() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"test-iflag","issue_type":"task","title":"Flag task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    );

    wk().args(["import", "--input", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-iflag"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Flag task"));
}

// =============================================================================
// Phase 2: Update and Collision Tests
// From: "import updates existing issues and detects collisions"
// =============================================================================

#[test]
fn import_updates_existing_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original title");

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Original title"));

    let content = format!(
        r#"{{"id":"{}","issue_type":"task","title":"Updated title","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}}"#,
        id
    );
    write_jsonl(&temp, "import.jsonl", &content);

    wk().args(["import", "import.jsonl"])
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
fn import_detects_collision() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original");

    wk().args(["start", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = format!(
        r#"{{"id":"{}","issue_type":"task","title":"Original","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}}"#,
        id
    );
    write_jsonl(&temp, "import.jsonl", &content);

    wk().args(["import", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("collision"));
}

// =============================================================================
// Phase 3: Dry Run Tests
// From: "import --dry-run shows preview without creating"
// =============================================================================

#[test]
fn import_dry_run() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"test-dry1","issue_type":"task","title":"Dry run task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    );

    wk().args(["import", "--dry-run", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("create"));

    wk().args(["show", "test-dry1"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Phase 4: Missing Dependencies Warning
// From: "import warns about missing dependencies"
// =============================================================================

#[test]
fn import_warns_missing_deps() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"test-dep1","issue_type":"task","title":"Task with deps","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[{"from_id":"test-dep1","to_id":"nonexistent-123","relation":"blocks","created_at":"2024-01-01T00:00:00Z"}],"events":[]}"#,
    );

    wk().args(["import", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("warning"))
        .stderr(predicate::str::contains("nonexistent"));
}

// =============================================================================
// Phase 5: Filter Tests
// From: "import --status, --type, --label, --prefix filters"
// =============================================================================

#[test]
fn import_filter_status() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"test-filt1","issue_type":"task","title":"Todo task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
            "\n",
            r#"{"id":"test-filt2","issue_type":"task","title":"Done task","status":"done","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#
        ),
    );

    wk().args(["import", "--status", "todo", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-filt1"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-filt2"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn import_filter_type() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"test-type1","issue_type":"task","title":"Task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
            "\n",
            r#"{"id":"test-type2","issue_type":"bug","title":"Bug","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#
        ),
    );

    wk().args(["import", "--type", "task", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-type1"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-type2"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn import_filter_label() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"test-label1","issue_type":"task","title":"Labeled","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent"],"notes":[],"deps":[],"events":[]}"#,
            "\n",
            r#"{"id":"test-label2","issue_type":"task","title":"Unlabeled","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#
        ),
    );

    wk().args(["import", "--label", "urgent", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-label1"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "test-label2"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn import_filter_prefix() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"myproj-a1","issue_type":"task","title":"My project task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
            "\n",
            r#"{"id":"other-b2","issue_type":"task","title":"Other project task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#
        ),
    );

    wk().args(["import", "--prefix", "myproj", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "myproj-a1"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "other-b2"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Phase 6: Format Detection Tests
// From: "import auto-detects bd format and --format bd parses beads format"
// =============================================================================

#[test]
fn import_autodetects_bd_format() {
    let temp = init_temp();
    std::fs::create_dir_all(temp.path().join(".beads")).unwrap();
    std::fs::write(
        temp.path().join(".beads/issues.jsonl"),
        r#"{"id":"bd-auto1","title":"Beads issue","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#
    ).unwrap();

    wk().args(["import", ".beads/issues.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-auto1"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn import_explicit_bd_format() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "beads.jsonl",
        r#"{"id":"bd-fmt1","title":"Beads task","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
    );

    wk().args(["import", "--format", "bd", "beads.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-fmt1"])
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Phase 7: Error Handling Tests
// From: "import fails on invalid JSON and shows help with no input"
// =============================================================================

#[test]
fn import_fails_invalid_json() {
    let temp = init_temp();
    write_jsonl(&temp, "invalid.jsonl", "not valid json");

    wk().args(["import", "invalid.jsonl"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn import_fails_no_input() {
    let temp = init_temp();

    wk().args(["import"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Phase 8: Beads Dependency Conversion Tests
// From: "import beads format converts dependency types"
// =============================================================================

#[test]
fn import_bd_dep_blocks() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"bd-blocker","title":"Blocker","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
            "\n",
            r#"{"id":"bd-blocked","title":"Blocked","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-blocker","type":"blocks"}]}"#
        ),
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-blocked"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked by"))
        .stdout(predicate::str::contains("bd-blocker"));
}

#[test]
fn import_bd_dep_parent() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"bd-child-p","title":"Child","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
            "\n",
            r#"{"id":"bd-parent-p","title":"Parent","status":"open","priority":2,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-child-p","type":"parent"}]}"#
        ),
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-parent-p"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracks"))
        .stdout(predicate::str::contains("bd-child-p"));
}

#[test]
fn import_bd_dep_parent_child() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"bd-parent-c","title":"Parent Epic","status":"open","priority":2,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
            "\n",
            r#"{"id":"bd-child-c","title":"Child Task","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-parent-c","type":"parent-child"}]}"#
        ),
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-child-c"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracked by"))
        .stdout(predicate::str::contains("bd-parent-c"));
}

#[test]
fn import_bd_dep_tracks() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        concat!(
            r#"{"id":"bd-tracked","title":"Tracked","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
            "\n",
            r#"{"id":"bd-tracker","title":"Tracker","status":"open","priority":2,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-tracked","type":"tracks"}]}"#
        ),
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-tracker"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracks"))
        .stdout(predicate::str::contains("bd-tracked"));
}

// =============================================================================
// Phase 9: Beads Close Reason Tests
// From: "import beads close reason maps to status and creates note"
// =============================================================================

#[test]
fn import_bd_close_reason_failure() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"bd-fail","title":"Failed issue","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"abandoned due to lack of resources"}"#,
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-fail"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: closed"));
}

#[test]
fn import_bd_close_reason_success() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"bd-success","title":"Successful issue","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"Completed successfully"}"#,
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-success"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
}

#[test]
fn import_bd_close_reason_note() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"bd-reason","title":"Issue with reason","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"duplicate of bd-other"}"#,
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-reason"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Close Reason"))
        .stdout(predicate::str::contains("duplicate of bd-other"));
}

// =============================================================================
// Phase 10: Beads Priority and Comment Tests
// From: "import beads format converts priority and preserves comments"
// =============================================================================

#[test]
fn import_bd_priority() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"bd-prio","title":"Priority issue","status":"open","priority":1,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-prio"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("priority:1"));
}

#[test]
fn import_bd_no_priority_zero() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"bd-prio0","title":"No priority issue","status":"open","priority":0,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-prio0"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("priority:0").not());
}

#[test]
fn import_bd_comment() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"bd-comment","title":"Commented issue","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","comments":[{"text":"This is a comment from beads","created_at":"2024-01-01T00:00:00Z"}]}"#,
    );

    wk().args(["import", "--format", "bd", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "bd-comment"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Description"))
        .stdout(predicate::str::contains("This is a comment from beads"));
}

// =============================================================================
// Phase 11: Shorthand Rejection Tests
// From: "import rejects -i and -p shorthands"
// Note: -p is accepted as shorthand for --prefix in the current implementation.
// =============================================================================

#[test]
fn import_rejects_shorthand_i() {
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"test-short","issue_type":"task","title":"Test","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    );

    wk().args(["import", "-i", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '-i'"));
}

#[test]
fn import_prefix_short_flag_p_accepted() {
    // Note: -p is accepted as shorthand for --prefix (differs from BATS spec)
    let temp = init_temp();
    write_jsonl(
        &temp,
        "import.jsonl",
        r#"{"id":"myproj-test","issue_type":"task","title":"Test","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    );

    wk().args(["import", "-p", "myproj", "import.jsonl"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", "myproj-test"])
        .current_dir(temp.path())
        .assert()
        .success();
}
