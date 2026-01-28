// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Init command tests - converted from tests/specs/cli/unit/init.bats
//!
//! BATS test mapping:
//! - "init creates .wok directory and fails if already initialized"
//!   → creates_wok_directory, fails_if_already_initialized, succeeds_if_wok_exists_without_config
//! - "init with --path creates at specified location"
//!   → path_option_*, 3 tests
//! - "init prefix handling and validation"
//!   → prefix handling tests, 6 tests
//! - "init creates valid database, config, and allows issue creation"
//!   → database/config tests, 5 tests
//! - "init with --workspace"
//!   → workspace tests, 6 tests
//! - "init creates .gitignore with correct entries"
//!   → gitignore tests, 4 tests
//! - "init with --remote excludes config.toml from .gitignore"
//!   → remote_mode_does_not_ignore_config_toml
//! - "init defaults to local mode without remote"
//!   → defaults_to_local_mode_no_remote_config
//! - "init with git remote creates worktree and supports sync"
//!   → remote worktree tests, 4 tests

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;

// =============================================================================
// Phase 2: Basic Init Tests
// From: "init creates .wok directory and fails if already initialized"
// =============================================================================

#[test]
fn creates_wok_directory() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("myapp")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".wok").exists());
    assert!(temp.path().join(".wok/config.toml").exists());
    assert!(temp.path().join(".wok/issues.db").exists());

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"myapp\""));
}

#[test]
fn fails_if_already_initialized() {
    let temp = TempDir::new().unwrap();

    // First init should succeed
    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    // Second init should fail
    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn succeeds_if_wok_exists_without_config() {
    let temp = TempDir::new().unwrap();

    // Create .wok directory without config
    std::fs::create_dir_all(temp.path().join(".wok")).unwrap();

    // Init should succeed
    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".wok/config.toml").exists());
}

// From: "init with --path creates at specified location"

#[test]
fn path_option_creates_at_specified_location() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("subdir")).unwrap();

    wk().arg("init")
        .arg("--path")
        .arg("subdir")
        .arg("--prefix")
        .arg("sub")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("subdir/.wok").exists());
    assert!(temp.path().join("subdir/.wok/config.toml").exists());
    assert!(temp.path().join("subdir/.wok/issues.db").exists());

    let config = std::fs::read_to_string(temp.path().join("subdir/.wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"sub\""));
}

#[test]
fn path_option_creates_parent_directories() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--path")
        .arg("nested/deep/dir")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("nested/deep/dir/.wok").exists());
}

#[test]
fn path_option_fails_if_already_initialized() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("subdir")).unwrap();

    // First init
    wk().arg("init")
        .arg("--path")
        .arg("subdir")
        .arg("--prefix")
        .arg("sub")
        .current_dir(temp.path())
        .assert()
        .success();

    // Second init should fail
    wk().arg("init")
        .arg("--path")
        .arg("subdir")
        .arg("--prefix")
        .arg("sub")
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Phase 3: Prefix Handling Tests
// From: "init prefix handling and validation"
// =============================================================================

#[test]
fn uses_directory_name_as_default_prefix() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("myproject");
    std::fs::create_dir_all(&project_dir).unwrap();

    wk().arg("init")
        .current_dir(&project_dir)
        .assert()
        .success();

    let config = std::fs::read_to_string(project_dir.join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"myproject\""));
}

#[test]
fn lowercases_and_filters_alphanumeric() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("MyProject123");
    std::fs::create_dir_all(&project_dir).unwrap();

    wk().arg("init")
        .current_dir(&project_dir)
        .assert()
        .success();

    let config = std::fs::read_to_string(project_dir.join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"myproject123\""));
}

#[test]
fn explicit_prefix_overrides_directory() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("somedir");
    std::fs::create_dir_all(&project_dir).unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("custom")
        .current_dir(&project_dir)
        .assert()
        .success();

    let config = std::fs::read_to_string(project_dir.join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"custom\""));
}

#[test]
fn fails_with_invalid_directory_name_for_prefix() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("a---");
    std::fs::create_dir_all(&project_dir).unwrap();

    wk().arg("init")
        .current_dir(&project_dir)
        .assert()
        .failure();
}

#[test]
fn valid_prefixes_accepted() {
    let temp = TempDir::new().unwrap();

    // Test: abc
    wk().arg("init")
        .arg("--prefix")
        .arg("abc")
        .current_dir(temp.path())
        .assert()
        .success();
    std::fs::remove_dir_all(temp.path().join(".wok")).unwrap();

    // Test: ab
    wk().arg("init")
        .arg("--prefix")
        .arg("ab")
        .current_dir(temp.path())
        .assert()
        .success();
    std::fs::remove_dir_all(temp.path().join(".wok")).unwrap();

    // Test: abc123
    wk().arg("init")
        .arg("--prefix")
        .arg("abc123")
        .current_dir(temp.path())
        .assert()
        .success();
    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"abc123\""));
    std::fs::remove_dir_all(temp.path().join(".wok")).unwrap();

    // Test: mylongprefix
    wk().arg("init")
        .arg("--prefix")
        .arg("mylongprefix")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn invalid_prefixes_rejected() {
    let temp = TempDir::new().unwrap();

    let invalid_prefixes = ["ABC", "123", "my-prefix", "my_prefix", "a"];

    for prefix in invalid_prefixes {
        wk().arg("init")
            .arg("--prefix")
            .arg(prefix)
            .current_dir(temp.path())
            .assert()
            .failure();
    }
}

// =============================================================================
// Phase 4: Database/Config Tests
// From: "init creates valid database, config, and allows issue creation"
// =============================================================================

#[test]
fn creates_valid_sqlite_database() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify database is valid SQLite by querying it
    let output = std::process::Command::new("sqlite3")
        .arg(temp.path().join(".wok/issues.db"))
        .arg("SELECT name FROM sqlite_master WHERE type='table';")
        .output()
        .expect("sqlite3 command failed");

    assert!(output.status.success());
}

#[test]
fn database_has_required_tables() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    let tables = ["issues", "deps", "labels", "notes", "events"];

    for table in tables {
        let output = std::process::Command::new("sqlite3")
            .arg(temp.path().join(".wok/issues.db"))
            .arg(format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}';",
                table
            ))
            .output()
            .expect("sqlite3 command failed");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim() == table,
            "Expected table '{}' but got '{}'",
            table,
            stdout.trim()
        );
    }
}

#[test]
fn empty_database_shows_no_issues() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should not contain issue type markers
    assert!(!stdout.contains("[task]"));
    assert!(!stdout.contains("[bug]"));
    assert!(!stdout.contains("[feature]"));
}

#[test]
fn config_is_valid_toml() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();

    // Check that prefix line exists with correct format
    assert!(
        config.contains("prefix = \"prj\""),
        "Config should contain prefix"
    );

    // Check that there's at least one key = value line
    let key_value_lines: Vec<&str> = config
        .lines()
        .filter(|line| line.starts_with(|c: char| c.is_ascii_lowercase()))
        .collect();
    assert!(
        !key_value_lines.is_empty(),
        "Config should have at least one key-value line"
    );
}

#[test]
fn allows_immediate_issue_creation_with_prefix() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("myprj")
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Test issue")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Issue ID should start with the prefix
    assert!(
        stdout.contains("myprj-"),
        "Issue ID should contain prefix 'myprj-', got: {}",
        stdout
    );
}

// =============================================================================
// Phase 5: Workspace Mode Tests
// From: "init with --workspace"
// =============================================================================

#[test]
fn workspace_creates_config_without_database() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg(ws_dir.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".wok/config.toml").exists());
    assert!(!temp.path().join(".wok/issues.db").exists());

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("workspace = "));
    assert!(!config.contains("\nprefix"));
}

#[test]
fn workspace_with_prefix() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg(ws_dir.to_str().unwrap())
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("workspace = "));
    assert!(config.contains("prefix = \"prj\""));
    assert!(!temp.path().join(".wok/issues.db").exists());
}

#[test]
fn workspace_validates_prefix() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg(ws_dir.to_str().unwrap())
        .arg("--prefix")
        .arg("ABC")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn workspace_accepts_relative_path() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("external/workspace")).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg("external/workspace")
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("workspace = \"external/workspace\""));
}

#[test]
fn workspace_with_path_option() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("subdir")).unwrap();
    std::fs::create_dir_all(temp.path().join("subdir/external/workspace")).unwrap();

    wk().arg("init")
        .arg("--path")
        .arg("subdir")
        .arg("--workspace")
        .arg("external/workspace")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("subdir/.wok").exists());

    let config = std::fs::read_to_string(temp.path().join("subdir/.wok/config.toml")).unwrap();
    assert!(config.contains("workspace = \"external/workspace\""));
}

#[test]
fn workspace_fails_if_not_exist() {
    let temp = TempDir::new().unwrap();

    // Absolute path
    wk().arg("init")
        .arg("--workspace")
        .arg("/nonexistent/path")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("workspace not found"));

    // Relative path
    wk().arg("init")
        .arg("--workspace")
        .arg("./nonexistent/dir")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("workspace not found"));
}

// =============================================================================
// Phase 6: Gitignore and Remote Mode Tests
// From: "init creates .gitignore with correct entries"
// =============================================================================

#[test]
fn gitignore_contains_current_and_database() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("current/"));
    assert!(gitignore.contains("issues.db"));
}

#[test]
fn default_mode_ignores_config_toml() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("config.toml"));
}

#[test]
fn local_flag_ignores_config_toml() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--local")
        .current_dir(temp.path())
        .assert()
        .success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("config.toml"));
}

#[test]
fn workspace_mode_ignores_config_toml() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg(ws_dir.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("current/"));
    assert!(gitignore.contains("issues.db"));
    assert!(gitignore.contains("config.toml"));
}

// From: "init with --remote excludes config.toml from .gitignore"

#[test]
fn remote_mode_does_not_ignore_config_toml() {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("git init failed");

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("current/"));
    assert!(gitignore.contains("issues.db"));
    // Remote mode should NOT ignore config.toml
    assert!(!gitignore.contains("config.toml"));
}

// From: "init defaults to local mode without remote"

#[test]
fn defaults_to_local_mode_no_remote_config() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(!config.contains("[remote]"));
    assert!(!config.contains("url ="));

    // Should not create git worktree (we're not in a git repo anyway)
    assert!(!temp.path().join(".git/wk/oplog").exists());
}

// From: "init with git remote creates worktree and supports sync"

#[test]
fn remote_creates_git_worktree() {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("git init failed");

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".git/wk/oplog").exists());
    assert!(temp.path().join(".git/wk/oplog/oplog.jsonl").exists());
}

#[test]
fn remote_creates_orphan_branch() {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("git init failed");

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success();

    // Check orphan branch exists
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/wok/oplog"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[test]
fn remote_worktree_protects_branch() {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("git init failed");

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success();

    // Try to delete the branch - should fail because worktree exists
    let output = std::process::Command::new("git")
        .args(["branch", "-D", "wok/oplog"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("worktree"));
}

#[test]
fn remote_sync_works_with_worktree() {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("git init failed");

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success();

    // Create an issue
    wk().arg("new")
        .arg("task")
        .arg("Test issue")
        .current_dir(temp.path())
        .assert()
        .success();

    // Sync should work
    wk().arg("remote")
        .arg("sync")
        .current_dir(temp.path())
        .assert()
        .success();
}
