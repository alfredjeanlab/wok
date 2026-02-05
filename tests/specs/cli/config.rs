// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Config command tests - converted from tests/specs/cli/unit/config.bats
//!
//! BATS test mapping:
//! - "config rename requires both old and new prefix"
//!   -> rename_requires_both_prefixes, rename_with_one_arg_fails
//! - "config rename changes issue IDs and updates config prefix"
//!   -> rename_changes_issue_ids, rename_updates_config_prefix
//! - "config rename preserves dependencies and labels"
//!   -> rename_preserves_dependencies, rename_preserves_labels
//! - "config rename only affects matching prefix and same prefix is noop"
//!   -> rename_same_prefix_is_noop, rename_non_matching_prefix_does_not_update_config
//! - "config rename rejects invalid prefixes"
//!   -> invalid_new_prefix_* tests (parameterized with yare)
//! - "config remote configures git and websocket remotes"
//!   -> remote_configures_git_*, remote_configures_websocket
//! - "config remote handles already-configured and changing remotes"
//!   -> remote_noop_when_same, remote_changing_not_supported
//! - "config remote fails when workspace is configured"
//!   -> remote_fails_with_workspace

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let output = wk()
        .arg("new")
        .arg(type_)
        .arg(title)
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_issue_with_label(temp: &TempDir, type_: &str, title: &str, label: &str) -> String {
    let output = wk()
        .arg("new")
        .arg(type_)
        .arg(title)
        .arg("--label")
        .arg(label)
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn init_with_prefix(temp: &TempDir, prefix: &str) {
    wk().arg("init")
        .arg("--prefix")
        .arg(prefix)
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Config Rename: Argument Validation
// From: "config rename requires both old and new prefix"
// =============================================================================

#[test]
fn rename_requires_both_prefixes() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "test");

    wk().arg("config")
        .arg("rename")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn rename_with_one_arg_fails() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "test");

    wk().arg("config")
        .arg("rename")
        .arg("newprefix")
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Config Rename: Main Functionality
// From: "config rename changes issue IDs and updates config prefix"
// =============================================================================

#[test]
fn rename_changes_issue_ids() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "old");

    let id = create_issue(&temp, "task", "Test issue");
    assert!(id.starts_with("old-"), "Issue ID should start with old-");

    wk().arg("config")
        .arg("rename")
        .arg("old")
        .arg("new")
        .current_dir(temp.path())
        .assert()
        .success();

    // Old ID should not exist
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();

    // New ID should exist
    let new_id = id.replace("old-", "new-");
    wk().arg("show")
        .arg(&new_id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Test issue"));
}

#[test]
fn rename_updates_config_prefix() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "old");

    create_issue(&temp, "task", "First issue");

    wk().arg("config")
        .arg("rename")
        .arg("old")
        .arg("new")
        .current_dir(temp.path())
        .assert()
        .success();

    // New issues should use new prefix
    let new_id = create_issue(&temp, "task", "Another issue");
    assert!(
        new_id.starts_with("new-"),
        "New issue ID '{}' should start with 'new-'",
        new_id
    );
}

// =============================================================================
// Config Rename: Preserves Data
// From: "config rename preserves dependencies and labels"
// =============================================================================

#[test]
fn rename_preserves_dependencies() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "old");

    let id1 = create_issue(&temp, "task", "Blocker");
    let id2 = create_issue(&temp, "task", "Blocked");

    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("config")
        .arg("rename")
        .arg("old")
        .arg("new")
        .current_dir(temp.path())
        .assert()
        .success();

    let new_id2 = id2.replace("old-", "new-");
    wk().arg("show")
        .arg(&new_id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked by"));
}

#[test]
fn rename_preserves_labels() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "old");

    let id = create_issue_with_label(&temp, "task", "Test issue", "urgent");

    wk().arg("config")
        .arg("rename")
        .arg("old")
        .arg("new")
        .current_dir(temp.path())
        .assert()
        .success();

    let new_id = id.replace("old-", "new-");
    wk().arg("show")
        .arg(&new_id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"));
}

// =============================================================================
// Config Rename: Edge Cases
// From: "config rename only affects matching prefix and same prefix is noop"
// =============================================================================

#[test]
fn rename_same_prefix_is_noop() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "same");

    let id = create_issue(&temp, "task", "Test issue");

    wk().arg("config")
        .arg("rename")
        .arg("same")
        .arg("same")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("already"));

    // Issue should still exist
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn rename_non_matching_prefix_does_not_update_config() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "current");

    wk().arg("config")
        .arg("rename")
        .arg("other")
        .arg("new")
        .current_dir(temp.path())
        .assert()
        .success();

    // New issues should still use the original prefix
    let new_id = create_issue(&temp, "task", "After rename");
    assert!(
        new_id.starts_with("current-"),
        "New issue ID '{}' should start with 'current-'",
        new_id
    );
}

// =============================================================================
// Config Rename: Invalid Prefix Validation (parameterized)
// From: "config rename rejects invalid prefixes"
// =============================================================================

#[yare::parameterized(
    new_prefix_too_short = { "old", "a" },
    new_prefix_contains_dash = { "old", "my-proj" },
    new_prefix_uppercase = { "old", "ABC" },
    old_prefix_too_short = { "a", "new" },
    old_prefix_contains_dash = { "my-proj", "new" },
)]
fn rename_rejects_invalid_prefix(old_prefix: &str, new_prefix: &str) {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "old");

    wk().arg("config")
        .arg("rename")
        .arg(old_prefix)
        .arg(new_prefix)
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Config Remote: Basic Configuration
// From: "config remote configures git and websocket remotes"
// Note: config remote is not yet implemented - these tests are ignored
// =============================================================================

#[test]
#[ignore = "config remote subcommand not yet implemented"]
fn remote_configures_git_dot() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "test");

    wk().arg("config")
        .arg("remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote configured: git:."));

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("[remote]"));
    assert!(config.contains("url = \"git:.\""));
}

#[test]
#[ignore = "config remote subcommand not yet implemented"]
fn remote_configures_explicit_git() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "test");

    wk().arg("config")
        .arg("remote")
        .arg("git:.")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote configured: git:."));
}

#[test]
#[ignore = "config remote subcommand not yet implemented"]
fn remote_configures_websocket() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "test");

    wk().arg("config")
        .arg("remote")
        .arg("ws://localhost:7890")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Remote configured: ws://localhost:7890",
        ));
}

#[test]
#[ignore = "config remote subcommand not yet implemented"]
fn remote_updates_gitignore_for_remote_mode() {
    let temp = TempDir::new().unwrap();
    init_with_prefix(&temp, "test");

    // Verify local mode gitignore includes config.toml
    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(
        gitignore.contains("config.toml"),
        "Private mode .gitignore should contain config.toml"
    );

    wk().arg("config")
        .arg("remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify remote mode gitignore does NOT include config.toml
    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(
        !gitignore.contains("config.toml"),
        "Remote mode .gitignore should NOT contain config.toml"
    );
}

// =============================================================================
// Config Remote: Edge Cases
// From: "config remote handles already-configured and changing remotes"
// =============================================================================

#[test]
#[ignore = "config remote subcommand not yet implemented"]
fn remote_noop_when_same() {
    let temp = TempDir::new().unwrap();

    // Init without --private creates remote mode with git:.
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("config")
        .arg("remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote configured: git:."));
}

#[test]
#[ignore = "config remote subcommand not yet implemented"]
fn remote_changing_not_supported() {
    let temp = TempDir::new().unwrap();

    // Init without --private creates remote mode with git:.
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("config")
        .arg("remote")
        .arg("ws://other:7890")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not currently supported"))
        .stdout(predicate::str::contains("Current remote: git:."));
}

// =============================================================================
// Config Remote: Workspace Incompatibility
// From: "config remote fails when workspace is configured"
// =============================================================================

#[test]
#[ignore = "--workspace flag and config remote not yet implemented"]
fn remote_fails_with_workspace() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("shared_wok")).unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .arg("--workspace")
        .arg("shared_wok")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("config")
        .arg("remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("workspace are incompatible"));
}

#[test]
#[ignore = "--workspace flag and config remote not yet implemented"]
fn remote_fails_with_workspace_includes_hint() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("shared_wok")).unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .arg("--workspace")
        .arg("shared_wok")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("config")
        .arg("remote")
        .arg("ws://localhost:7890")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("hint:"));
}
