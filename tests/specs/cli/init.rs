// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Init command tests - converted from tests/specs/cli/unit/init.bats
//!
//! BATS test mapping:
//! - "init creates .wok directory and fails if already initialized"
//!   -> creates_wok_directory, fails_if_already_initialized, succeeds_if_wok_exists_without_config
//! - "init with --path creates at specified location"
//!   -> path_option_*, 3 tests
//! - "init prefix handling and validation"
//!   -> prefix handling tests, 6 tests
//! - "init creates valid database, config, and allows issue creation"
//!   -> database/config tests, 5 tests
//! - "init creates .gitignore with correct entries"
//!   -> gitignore tests
//! - "init --private creates local database"
//!   -> private mode tests

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

    wk().arg("init").arg("--prefix").arg("myapp").current_dir(temp.path()).assert().success();

    assert!(temp.path().join(".wok").exists());
    assert!(temp.path().join(".wok/config.toml").exists());
    // User-level mode: database is NOT in .wok/
    assert!(!temp.path().join(".wok/issues.db").exists());

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"myapp\""));
}

#[test]
fn fails_if_already_initialized() {
    let temp = TempDir::new().unwrap();

    // First init should succeed
    wk().arg("init").arg("--prefix").arg("prj").current_dir(temp.path()).assert().success();

    // Second init should fail
    wk().arg("init").arg("--prefix").arg("prj").current_dir(temp.path()).assert().failure();
}

#[test]
fn succeeds_if_wok_exists_without_config() {
    let temp = TempDir::new().unwrap();

    // Create .wok directory without config
    std::fs::create_dir_all(temp.path().join(".wok")).unwrap();

    // Init should succeed
    wk().arg("init").arg("--prefix").arg("prj").current_dir(temp.path()).assert().success();

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
    // User-level mode: no local database
    assert!(!temp.path().join("subdir/.wok/issues.db").exists());

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

    wk().arg("init").current_dir(&project_dir).assert().success();

    let config = std::fs::read_to_string(project_dir.join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"myproject\""));
}

#[test]
fn lowercases_and_filters_alphanumeric() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("MyProject123");
    std::fs::create_dir_all(&project_dir).unwrap();

    wk().arg("init").current_dir(&project_dir).assert().success();

    let config = std::fs::read_to_string(project_dir.join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"myproject123\""));
}

#[test]
fn explicit_prefix_overrides_directory() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("somedir");
    std::fs::create_dir_all(&project_dir).unwrap();

    wk().arg("init").arg("--prefix").arg("custom").current_dir(&project_dir).assert().success();

    let config = std::fs::read_to_string(project_dir.join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"custom\""));
}

#[test]
fn fails_with_invalid_directory_name_for_prefix() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("a---");
    std::fs::create_dir_all(&project_dir).unwrap();

    wk().arg("init").current_dir(&project_dir).assert().failure();
}

#[test]
fn valid_prefixes_accepted() {
    let temp = TempDir::new().unwrap();

    // Test: abc
    wk().arg("init").arg("--prefix").arg("abc").current_dir(temp.path()).assert().success();
    std::fs::remove_dir_all(temp.path().join(".wok")).unwrap();

    // Test: ab
    wk().arg("init").arg("--prefix").arg("ab").current_dir(temp.path()).assert().success();
    std::fs::remove_dir_all(temp.path().join(".wok")).unwrap();

    // Test: abc123
    wk().arg("init").arg("--prefix").arg("abc123").current_dir(temp.path()).assert().success();
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
        wk().arg("init").arg("--prefix").arg(prefix).current_dir(temp.path()).assert().failure();
    }
}

// =============================================================================
// Phase 4: Database/Config Tests
// From: "init creates valid database, config, and allows issue creation"
// =============================================================================

#[test]
fn private_creates_valid_sqlite_database() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    // Private mode: database is in .wok/
    assert!(temp.path().join(".wok/issues.db").exists());

    // Verify database is valid SQLite by querying it
    let output = std::process::Command::new("sqlite3")
        .arg(temp.path().join(".wok/issues.db"))
        .arg("SELECT name FROM sqlite_master WHERE type='table';")
        .output()
        .expect("sqlite3 command failed");

    assert!(output.status.success());
}

#[test]
fn private_database_has_required_tables() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    let tables = ["issues", "deps", "labels", "notes", "events"];

    for table in tables {
        let output = std::process::Command::new("sqlite3")
            .arg(temp.path().join(".wok/issues.db"))
            .arg(format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{}';", table))
            .output()
            .expect("sqlite3 command failed");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.trim() == table, "Expected table '{}' but got '{}'", table, stdout.trim());
    }
}

#[test]
fn private_empty_database_shows_no_issues() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--private")
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

    wk().arg("init").arg("--prefix").arg("prj").current_dir(temp.path()).assert().success();

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();

    // Check that prefix line exists with correct format
    assert!(config.contains("prefix = \"prj\""), "Config should contain prefix");

    // Check that there's at least one key = value line
    let key_value_lines: Vec<&str> =
        config.lines().filter(|line| line.starts_with(|c: char| c.is_ascii_lowercase())).collect();
    assert!(!key_value_lines.is_empty(), "Config should have at least one key-value line");
}

#[test]
fn allows_immediate_issue_creation_with_prefix() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("myprj")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    let output =
        wk().arg("new").arg("task").arg("Test issue").current_dir(temp.path()).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Issue ID should start with the prefix
    assert!(stdout.contains("myprj-"), "Issue ID should contain prefix 'myprj-', got: {}", stdout);
}

// =============================================================================
// Phase 5: Private Mode Tests
// =============================================================================

#[test]
fn private_creates_local_database() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".wok").exists());
    assert!(temp.path().join(".wok/config.toml").exists());
    assert!(temp.path().join(".wok/issues.db").exists());

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config.contains("prefix = \"prj\""));
    assert!(config.contains("private = true"));
}

#[test]
fn private_with_path_option() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("subdir")).unwrap();

    wk().arg("init")
        .arg("--path")
        .arg("subdir")
        .arg("--prefix")
        .arg("sub")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("subdir/.wok/issues.db").exists());

    let config = std::fs::read_to_string(temp.path().join("subdir/.wok/config.toml")).unwrap();
    assert!(config.contains("private = true"));
}

// =============================================================================
// Phase 6: Gitignore Tests
// =============================================================================

#[test]
fn user_level_gitignore_contains_config_only() {
    let temp = TempDir::new().unwrap();

    wk().arg("init").arg("--prefix").arg("prj").current_dir(temp.path()).assert().success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("config.toml"));
    // User-level mode: no local database to ignore
    assert!(!gitignore.contains("issues.db"));
}

#[test]
fn private_gitignore_contains_config_and_database() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("config.toml"));
    assert!(gitignore.contains("issues.db"));
}

#[test]
fn default_mode_ignores_config_toml() {
    let temp = TempDir::new().unwrap();

    wk().arg("init").arg("--prefix").arg("prj").current_dir(temp.path()).assert().success();

    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("config.toml"));
}

// =============================================================================
// Phase 7: Default mode (user-level) Tests
// =============================================================================

#[test]
fn defaults_to_user_level_mode() {
    let temp = TempDir::new().unwrap();

    wk().arg("init").arg("--prefix").arg("prj").current_dir(temp.path()).assert().success();

    let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    // Default mode should not have private = true
    assert!(!config.contains("private = true"));
    // No local database in user-level mode
    assert!(!temp.path().join(".wok/issues.db").exists());
}
