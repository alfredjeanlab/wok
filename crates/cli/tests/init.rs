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
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized issue tracker"));

    assert!(temp.path().join(".wok").exists());
    assert!(temp.path().join(".wok/config.toml").exists());
    assert!(temp.path().join(".wok/issues.db").exists());
}

#[test]
fn creates_work_dir_user_level() {
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
    // User-level mode: no local database
    assert!(!temp.path().join(".wok/issues.db").exists());
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
fn init_defaults_to_user_level_mode() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();

    // Config should not have private = true
    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(
        !config_content.contains("private = true"),
        "Default init should not be private mode"
    );

    // Gitignore should include config.toml
    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(
        gitignore.contains("config.toml"),
        "Should ignore config.toml"
    );
    // User-level mode: no local database to ignore
    assert!(
        !gitignore.contains("issues.db"),
        "User-level mode should not ignore issues.db"
    );
}

#[test]
fn init_private_mode() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    // Config should have private = true
    let config_content = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
    assert!(
        config_content.contains("private = true"),
        "Private init should set private = true"
    );

    // Database should be in .wok/
    assert!(temp.path().join(".wok/issues.db").exists());

    // Gitignore should include both config.toml and issues.db
    let gitignore = std::fs::read_to_string(temp.path().join(".wok/.gitignore")).unwrap();
    assert!(gitignore.contains("config.toml"));
    assert!(gitignore.contains("issues.db"));
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
        .arg("--private")
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
// Database schema validation
// ============================================================================

#[test]
fn init_creates_database_with_required_tables() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("prj")
        .arg("--private")
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
        .arg("--private")
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
