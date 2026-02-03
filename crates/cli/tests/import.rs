// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn wk() -> Command {
    cargo_bin_cmd!("wok")
}

#[test]
fn test_import_creates_issue() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    // Create import file
    let import_file = temp.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-imp1","issue_type":"task","title":"Imported task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    wk().arg("import")
        .arg(import_file.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("create: 1"));

    // Verify issue was created
    wk().arg("show")
        .arg("test-imp1")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported task"));
}

#[test]
fn test_import_dry_run() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    let import_file = temp.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-dry","issue_type":"task","title":"Dry run task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    wk().arg("import")
        .arg("--dry-run")
        .arg(import_file.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"))
        .stdout(predicate::str::contains("create: 1"));

    // Verify issue was NOT created
    wk().arg("show")
        .arg("test-dry")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn test_import_updates_existing() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    // Create an issue first
    let output = wk()
        .arg("new")
        .arg("Original title")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .find(|s| s.starts_with("test-"))
        .unwrap()
        .trim_end_matches(':');

    // Import with updated title
    let import_file = temp.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        format!(
            r#"{{"id":"{}","issue_type":"task","title":"Updated title","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}}"#,
            id
        ),
    )
    .unwrap();

    wk().arg("import")
        .arg(import_file.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("update: 1"));

    // Verify title was updated
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated title"));
}

#[test]
fn test_import_beads_format() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    let import_file = temp.path().join("beads.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-test","title":"Beads issue","status":"open","priority":2,"issue_type":"bug","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent"]}"#,
    )
    .unwrap();

    wk().arg("import")
        .arg("--format")
        .arg("bd")
        .arg(import_file.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify issue was created with correct type conversion
    wk().arg("show")
        .arg("bd-test")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Beads issue"))
        .stdout(predicate::str::contains("[bug]"))
        .stdout(predicate::str::contains("Status: todo")); // "open" -> "todo"
}

#[test]
fn test_import_with_stdin() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("import")
        .arg("-")
        .current_dir(temp.path())
        .write_stdin(r#"{"id":"test-stdin","issue_type":"task","title":"Stdin task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#)
        .assert()
        .success();

    // Verify issue was created
    wk().arg("show")
        .arg("test-stdin")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Stdin task"));
}

#[test]
fn test_import_without_input_shows_error() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    // Import without file should show error
    wk().arg("import")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no input file"));
}

#[test]
fn test_import_status_filter() {
    let temp = TempDir::new().unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    let import_file = temp.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-todo","issue_type":"task","title":"Todo task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-done","issue_type":"task","title":"Done task","status":"done","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    wk().arg("import")
        .arg("--status")
        .arg("todo")
        .arg(import_file.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("create: 1"))
        .stdout(predicate::str::contains("filtered: 1"));

    // Only todo should be imported
    wk().arg("show")
        .arg("test-todo")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg("test-done")
        .current_dir(temp.path())
        .assert()
        .failure();
}
