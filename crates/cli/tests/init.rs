// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

mod common;
use common::*;
use yare::parameterized;

#[test]
fn creates_work_dir() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized issue tracker"));

    assert!(temp.path().join(".wok").exists());
    assert!(temp.path().join(".wok/config.toml").exists());
    assert!(temp.path().join(".wok/issues.db").exists());
}

#[test]
fn fails_if_already_initialized() {
    let temp = TempDir::new().unwrap();

    // First init should succeed
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();

    // Second init should fail
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn uses_directory_name_as_default_prefix() {
    let temp = TempDir::new().unwrap();

    // Init without prefix should succeed and use directory name
    let output = wk()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.contains("Prefix:"),
        "Expected prefix in output: {}",
        stdout
    );
}

#[test]
fn invalid_prefix() {
    let temp = TempDir::new().unwrap();

    // Single character prefix
    wk().arg("init")
        .arg("--prefix")
        .arg("a")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("prefix"));

    // Uppercase prefix
    wk().arg("init")
        .arg("--prefix")
        .arg("AB")
        .current_dir(temp.path())
        .assert()
        .failure();

    // Pure digits (no letters)
    wk().arg("init")
        .arg("--prefix")
        .arg("123")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn valid_alphanumeric_prefix() {
    let temp = TempDir::new().unwrap();

    // Alphanumeric prefix should work
    wk().arg("init")
        .arg("--prefix")
        .arg("v0")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Prefix: v0"));
}

#[test]
fn init_with_workspace_creates_link() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg(ws_dir.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized workspace link"));

    // Config should exist
    assert!(temp.path().join(".wok/config.toml").exists());
    // Database should NOT exist
    assert!(!temp.path().join(".wok/issues.db").exists());
}

#[test]
fn init_with_workspace_and_prefix() {
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
        .success()
        .stdout(predicate::str::contains("Prefix: prj"));

    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config_content.contains("prefix = \"prj\""));
    assert!(config_content.contains("workspace = "));
}

#[test]
fn init_with_workspace_no_prefix_in_config() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg(ws_dir.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success();

    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    // Should NOT contain prefix when not specified
    assert!(!config_content.contains("prefix"));
    assert!(config_content.contains("workspace = "));
}

#[test]
fn init_with_workspace_fails_if_workspace_not_exist() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg("/nonexistent/workspace/path")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("workspace not found"));
}

#[test]
fn init_with_workspace_rejects_invalid_prefix() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg("/some/workspace")
        .arg("--prefix")
        .arg("AB") // Invalid: uppercase
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn init_defaults_to_local_mode() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();

    // Config should not have remote section
    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(
        !config_content.contains("[remote]"),
        "Default init should not create remote config"
    );
    assert!(
        !config_content.contains("url ="),
        "Default init should not have remote url"
    );

    // Gitignore should include config.toml (local mode)
    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(
        gitignore.contains("config.toml"),
        "Local mode should ignore config.toml"
    );
}

#[test]
fn init_local_flag_is_no_op() {
    let temp = TempDir::new().unwrap();

    // --local flag should work (for backwards compatibility) and behave same as default
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--local")
        .current_dir(temp.path())
        .assert()
        .success();

    // Config should not have remote section
    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(!config_content.contains("[remote]"));

    // Gitignore should include config.toml
    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("config.toml"));
}

// ============================================================================
// --path option tests
// ============================================================================

#[test]
fn init_with_path_creates_at_specified_location() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("subdir");
    std::fs::create_dir_all(&target).unwrap();

    wk().arg("init")
        .arg("--path")
        .arg(target.to_str().unwrap())
        .arg("--prefix")
        .arg("sub")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(target.join(".wok").exists());
    assert!(target.join(".wok/config.toml").exists());
    assert!(target.join(".wok/issues.db").exists());

    let config_content = std::fs::read_to_string(target.join(".wok/config.toml")).unwrap();
    assert!(config_content.contains("prefix = \"sub\""));
}

#[test]
fn init_with_path_creates_parent_directories() {
    let temp = TempDir::new().unwrap();
    let nested = temp.path().join("nested/deep/dir");

    // Parent directories should be created automatically
    wk().arg("init")
        .arg("--path")
        .arg(nested.to_str().unwrap())
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(nested.join(".wok").exists());
    assert!(nested.join(".wok/config.toml").exists());
}

#[test]
fn init_with_path_fails_if_already_initialized() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("subdir");
    std::fs::create_dir_all(&target).unwrap();

    // First init at path should succeed
    wk().arg("init")
        .arg("--path")
        .arg(target.to_str().unwrap())
        .arg("--prefix")
        .arg("sub")
        .current_dir(temp.path())
        .assert()
        .success();

    // Second init at same path should fail
    wk().arg("init")
        .arg("--path")
        .arg(target.to_str().unwrap())
        .arg("--prefix")
        .arg("sub")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

// ============================================================================
// Partial initialization recovery tests
// ============================================================================

#[test]
fn init_succeeds_if_wok_dir_exists_without_config() {
    let temp = TempDir::new().unwrap();

    // Create .wok directory without config
    std::fs::create_dir_all(temp.path().join(".wok")).unwrap();

    // Init should succeed (recovery case)
    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".wok/config.toml").exists());
    assert!(temp.path().join(".wok/issues.db").exists());
}

// ============================================================================
// Prefix derivation parameterized tests
// ============================================================================

#[parameterized(
    mixed_case = { "MyProject", "myproject" },
    with_symbols = { "my-project_v2", "myprojectv2" },
    with_digits = { "Project123", "project123" },
)]
fn should_derive_prefix_from_directory_name(dir_name: &str, expected_prefix: &str) {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join(dir_name);
    std::fs::create_dir_all(&target).unwrap();

    wk().arg("init")
        .current_dir(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Prefix: {}",
            expected_prefix
        )));

    let config_content = std::fs::read_to_string(target.join(".wok/config.toml")).unwrap();
    assert!(
        config_content.contains(&format!("prefix = \"{}\"", expected_prefix)),
        "Expected prefix '{}' in config, got: {}",
        expected_prefix,
        config_content
    );
}

#[parameterized(
    too_short = { "a---" },
    digits_only = { "123" },
    all_symbols = { "---" },
)]
fn should_fail_with_underivable_prefix(dir_name: &str) {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join(dir_name);
    std::fs::create_dir_all(&target).unwrap();

    wk().arg("init").current_dir(&target).assert().failure();
}

// ============================================================================
// Workspace edge cases
// ============================================================================

#[test]
fn init_workspace_accepts_relative_path() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("external/workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--workspace")
        .arg("external/workspace")
        .current_dir(temp.path())
        .assert()
        .success();

    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(config_content.contains("workspace = \"external/workspace\""));
}

#[test]
fn init_workspace_with_path_option() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("subdir");
    let ws_dir = target.join("external/workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();

    wk().arg("init")
        .arg("--path")
        .arg(target.to_str().unwrap())
        .arg("--workspace")
        .arg("external/workspace")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(target.join(".wok").exists());
    assert!(target.join(".wok/config.toml").exists());
    assert!(!target.join(".wok/issues.db").exists()); // Workspace mode has no DB

    let config_content = std::fs::read_to_string(target.join(".wok/config.toml")).unwrap();
    assert!(config_content.contains("workspace = \"external/workspace\""));
}

#[test]
fn init_workspace_creates_gitignore_with_config() {
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
    assert!(
        gitignore.contains("config.toml"),
        "Workspace mode should ignore config.toml"
    );
}

// ============================================================================
// Remote mode tests
// ============================================================================

#[test]
fn init_remote_excludes_config_from_gitignore() {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first (required for remote mode)
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
    assert!(
        !gitignore.contains("config.toml"),
        "Remote mode should NOT ignore config.toml (it's shared via git)"
    );
}

#[test]
fn init_remote_creates_oplog_worktree() {
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

    // Should create oplog worktree at .git/wk/oplog
    assert!(
        temp.path().join(".git/wk/oplog").exists(),
        "Remote mode should create oplog worktree"
    );
    assert!(
        temp.path().join(".git/wk/oplog/oplog.jsonl").exists(),
        "Oplog worktree should contain oplog.jsonl"
    );
}

#[test]
fn init_remote_creates_config_with_remote_section() {
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

    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(
        config_content.contains("[remote]"),
        "Remote mode should create remote config section"
    );
    assert!(
        config_content.contains("url = \"git:.\""),
        "Remote config should have url"
    );
}

// ============================================================================
// Database schema validation
// ============================================================================

#[test]
fn init_creates_database_with_required_tables() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify database has required tables using sqlite3
    let output = std::process::Command::new("sqlite3")
        .arg(temp.path().join(".wok/issues.db"))
        .arg("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;")
        .output()
        .expect("sqlite3 command failed");

    let tables = String::from_utf8_lossy(&output.stdout);
    assert!(tables.contains("issues"), "Missing 'issues' table");
    assert!(tables.contains("deps"), "Missing 'deps' table");
    assert!(tables.contains("labels"), "Missing 'labels' table");
    assert!(tables.contains("notes"), "Missing 'notes' table");
    assert!(tables.contains("events"), "Missing 'events' table");
}

#[test]
fn init_allows_immediate_issue_creation() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("myprj")
        .current_dir(temp.path())
        .assert()
        .success();

    // Create an issue immediately after init
    wk().arg("new")
        .arg("task")
        .arg("Test issue")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"myprj-[a-z0-9]+").unwrap());
}
