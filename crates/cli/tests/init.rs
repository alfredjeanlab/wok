// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

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
