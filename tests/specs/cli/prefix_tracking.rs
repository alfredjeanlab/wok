// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for prefix tracking commands.
//! Converted from tests/specs/cli/unit/prefix-tracking.bats
//!
//! BATS test mapping:
//! - "config prefixes shows all prefixes with counts"
//!   -> config_prefixes_shows_all_prefixes_with_counts
//! - "config prefixes with empty database shows no prefixes"
//!   -> config_prefixes_empty_database_shows_no_prefixes
//! - "config prefixes -o json outputs valid JSON"
//!   -> config_prefixes_json_output_valid
//! - "config prefixes -o id outputs only prefix names"
//!   -> config_prefixes_id_output_only_names
//! - "new --prefix creates issue with different prefix"
//!   -> new_prefix_creates_issue_with_different_prefix
//! - "new -p short form works for prefix"
//!   -> new_prefix_short_flag
//! - "new --prefix rejects invalid prefix"
//!   -> new_prefix_rejects_invalid_* (parameterized)
//! - "new --prefix updates prefix table"
//!   -> new_prefix_updates_prefix_table
//! - "new --prefix works with all flags"
//!   -> new_prefix_works_with_all_flags
//! - "config rename updates prefixes table"
//!   -> config_rename_updates_prefixes_table
//! - "existing databases backfill prefixes table"
//!   -> existing_databases_backfill_prefixes_table

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use yare::parameterized;

fn wk() -> Command {
    cargo_bin_cmd!("wok")
}

/// Initialize a temp directory with a custom prefix.
fn init_temp_with_prefix(prefix: &str) -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg(prefix)
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}

/// Create an issue and return its ID.
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

/// Create an issue with a specific prefix and return its ID.
fn create_issue_with_prefix(temp: &TempDir, type_: &str, title: &str, prefix: &str) -> String {
    let output = wk()
        .arg("new")
        .arg(type_)
        .arg(title)
        .arg("--prefix")
        .arg(prefix)
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// config prefixes Tests
// =============================================================================

#[test]
fn config_prefixes_shows_all_prefixes_with_counts() {
    let temp = init_temp_with_prefix("proj");

    create_issue(&temp, "task", "Proj task 1");
    create_issue(&temp, "task", "Proj task 2");
    create_issue_with_prefix(&temp, "task", "Other task", "other");

    wk().arg("config")
        .arg("prefixes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("proj: 2 issues"))
        .stdout(predicate::str::contains("other: 1 issue"))
        .stdout(predicate::str::contains("(default)"));
}

#[test]
fn config_prefixes_empty_database_shows_no_prefixes() {
    let temp = init_temp_with_prefix("test");

    wk().arg("config")
        .arg("prefixes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No prefixes found"));
}

#[test]
fn config_prefixes_json_output_valid() {
    let temp = init_temp_with_prefix("main");

    create_issue(&temp, "task", "Test task");

    let output = wk()
        .arg("config")
        .arg("prefixes")
        .arg("-o")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check structure
    assert_eq!(json.get("default").and_then(|v| v.as_str()), Some("main"));
    let prefixes = json.get("prefixes").and_then(|v| v.as_array()).unwrap();
    assert!(!prefixes.is_empty());

    let first = &prefixes[0];
    assert_eq!(first.get("prefix").and_then(|v| v.as_str()), Some("main"));
    assert_eq!(first.get("issue_count").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(
        first.get("is_default").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn config_prefixes_id_output_only_names() {
    let temp = init_temp_with_prefix("main");

    create_issue(&temp, "task", "Main task");
    create_issue_with_prefix(&temp, "task", "Other task", "api");

    wk().arg("config")
        .arg("prefixes")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("api"))
        // Should not include counts or markers
        .stdout(predicate::str::contains("issue").not())
        .stdout(predicate::str::contains("(default)").not());
}

// =============================================================================
// new --prefix Tests
// =============================================================================

#[test]
fn new_prefix_creates_issue_with_different_prefix() {
    let temp = init_temp_with_prefix("main");

    let id = create_issue_with_prefix(&temp, "task", "Other project task", "other");
    assert!(
        id.starts_with("other-"),
        "Expected other- prefix, got: {}",
        id
    );

    // Issue should be visible
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Other project task"));
}

#[test]
fn new_prefix_short_flag() {
    let temp = init_temp_with_prefix("main");

    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Short prefix")
        .arg("-p")
        .arg("short")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        id.starts_with("short-"),
        "Expected short- prefix, got: {}",
        id
    );
}

#[parameterized(
    too_short = { "a" },
    contains_dash = { "my-proj" },
    uppercase = { "ABC" },
)]
fn new_prefix_rejects_invalid(prefix: &str) {
    let temp = init_temp_with_prefix("main");

    wk().arg("new")
        .arg("task")
        .arg("Invalid")
        .arg("--prefix")
        .arg(prefix)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn new_prefix_updates_prefix_table() {
    let temp = init_temp_with_prefix("main");

    create_issue(&temp, "task", "Main task");
    create_issue_with_prefix(&temp, "task", "API task", "api");

    wk().arg("config")
        .arg("prefixes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("main: 1 issue"))
        .stdout(predicate::str::contains("api: 1 issue"));
}

#[test]
fn new_prefix_works_with_all_flags() {
    let temp = init_temp_with_prefix("main");

    // Combine with --label, --note, type
    wk().arg("new")
        .arg("bug")
        .arg("Flagged issue")
        .arg("--prefix")
        .arg("other")
        .arg("--label")
        .arg("urgent")
        .arg("--note")
        .arg("Description")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("other-"))
        .stdout(predicate::str::contains("[bug]"));

    // Get another ID for dependency test
    let id = create_issue_with_prefix(&temp, "task", "Another", "other");

    // Verify dependencies work with prefix
    wk().arg("new")
        .arg("task")
        .arg("Blocked")
        .arg("--prefix")
        .arg("other")
        .arg("--blocked-by")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("config")
        .arg("prefixes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("other: 3 issues"));
}

// =============================================================================
// config rename Tests
// =============================================================================

#[test]
fn config_rename_updates_prefixes_table() {
    let temp = init_temp_with_prefix("old");

    create_issue(&temp, "task", "Test");

    wk().arg("config")
        .arg("prefixes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("old: 1 issue"));

    wk().arg("config")
        .arg("rename")
        .arg("old")
        .arg("new")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("config")
        .arg("prefixes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("new: 1 issue"))
        .stdout(predicate::str::contains("old:").not());
}

// =============================================================================
// Backfill Tests
// =============================================================================

#[test]
fn existing_databases_backfill_prefixes_table() {
    let temp = init_temp_with_prefix("proj");

    // Create issues first
    create_issue(&temp, "task", "Task 1");
    create_issue(&temp, "task", "Task 2");

    // The prefixes should be auto-populated from existing issues
    wk().arg("config")
        .arg("prefixes")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("proj: 2 issues"));
}
