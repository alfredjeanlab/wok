// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Error handling tests - converted from tests/specs/cli/edge_cases/errors.bats
//!
//! BATS test mapping:
//! - "commands without initialization fail with helpful message"
//!   -> commands_without_initialization_fail (parameterized: list, new, show)
//! - "invalid issue IDs fail"
//!   -> invalid_issue_ids_fail (parameterized: show, start, dep blocker, dep blocked)
//! - "missing required arguments fail"
//!   -> missing_required_arguments_fail (parameterized: new, note, dep, close, reopen)
//! - "invalid values fail"
//!   -> invalid_values_fail (parameterized: new type, edit type, dep relationship,
//!      list status, list type)
//! - "exit codes: success returns 0, not found and invalid return non-zero"
//!   -> exit_code_success_returns_0, exit_code_not_found_returns_nonzero,
//!      exit_code_invalid_command_returns_nonzero
//! - "removed commands (sync/daemon) are not available"
//!   -> removed_commands_not_in_help (parameterized: sync, daemon)
//!   -> removed_commands_fail (parameterized: sync, daemon)
//! - "new fails when project has no prefix configured"
//!   -> new_fails_when_project_has_no_prefix

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::common::*;
use yare::parameterized;

// =============================================================================
// Helpers
// =============================================================================

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Commands Without Initialization
// =============================================================================

#[parameterized(
    list = { &["list"] },
    new = { &["new", "Test"] },
    show = { &["show", "test-abc"] },
)]
fn commands_without_initialization_fail(args: &[&str]) {
    let temp = TempDir::new().unwrap();

    // Verify .wok doesn't exist
    assert!(!temp.path().join(".wok").exists());

    wk().args(args).current_dir(temp.path()).assert().failure();
}

// =============================================================================
// Invalid Issue IDs
// =============================================================================

#[parameterized(
    show_invalid = { &["show", "invalid-id-format"] },
    start_invalid = { &["start", "not-an-id"] },
)]
fn invalid_issue_ids_fail(args: &[&str]) {
    let temp = init_temp();

    wk().args(args).current_dir(temp.path()).assert().failure();
}

#[test]
fn dep_with_invalid_blocker_id_fails() {
    let temp = init_temp();
    let b = create_issue(&temp, "task", "Task B");

    wk().args(["dep", "invalid", "blocks", &b])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn dep_with_invalid_blocked_id_fails() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Task A");

    wk().args(["dep", &a, "blocks", "invalid"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Missing Required Arguments
// =============================================================================

#[test]
fn new_without_title_fails() {
    let temp = init_temp();

    wk().arg("new").current_dir(temp.path()).assert().failure();
}

#[test]
fn note_without_content_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test");

    wk().args(["note", &id])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn dep_without_relationship_fails() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "A");
    let b = create_issue(&temp, "task", "B");

    wk().args(["dep", &a, &b])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn close_without_reason_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test2");

    wk().args(["close", &id])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn reopen_from_done_without_reason_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test3");

    wk().args(["start", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["done", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["reopen", &id])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Invalid Values
// =============================================================================

#[test]
fn new_with_invalid_type_fails() {
    let temp = init_temp();

    wk().args(["new", "invalid", "Test"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn edit_with_invalid_type_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test");

    wk().args(["edit", &id, "--type", "invalid"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn dep_with_invalid_relationship_fails() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "A");
    let b = create_issue(&temp, "task", "B");

    wk().args(["dep", &a, "requires", &b])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn list_with_invalid_status_fails() {
    let temp = init_temp();

    wk().args(["list", "--status", "invalid"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn list_with_invalid_type_fails() {
    let temp = init_temp();

    wk().args(["list", "--type", "invalid"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Exit Codes
// =============================================================================

#[test]
fn exit_code_success_returns_0() {
    let temp = init_temp();

    wk().args(["new", "task", "Test"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn exit_code_not_found_returns_nonzero() {
    let temp = init_temp();

    wk().args(["show", "test-nonexistent"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn exit_code_invalid_command_returns_nonzero() {
    wk().arg("nonexistent").assert().failure();
}

// =============================================================================
// Removed Commands (sync/daemon)
// =============================================================================

#[parameterized(
    sync = { "sync" },
    daemon = { "daemon" },
)]
fn removed_commands_not_in_help(cmd_name: &str) {
    let pattern = format!("^  {}\\s", cmd_name);

    wk().arg("help")
        .assert()
        .success()
        .stdout(predicate::str::is_match(pattern).unwrap().not());
}

#[parameterized(
    sync = { "sync" },
    daemon = { "daemon" },
)]
fn removed_commands_fail(cmd_name: &str) {
    wk().arg(cmd_name).assert().failure();
}

// =============================================================================
// No Prefix Configured
// =============================================================================

#[test]
#[ignore = "--workspace flag not yet implemented"]
fn new_fails_when_project_has_no_prefix() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("workspace")).unwrap();

    // Initialize a project with a prefix at workspace path
    wk().args(["init", "--path", "workspace", "--prefix", "ws"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Initialize a workspace (no prefix) pointing to that project
    wk().args(["init", "--workspace", "workspace"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Creating a new issue should fail because we're in a workspace with no prefix
    wk().args(["new", "task", "Test task"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no prefix configured"));
}
