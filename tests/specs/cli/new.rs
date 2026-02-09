// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk new` command.
//! Converted from tests/specs/cli/unit/new.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;

fn create_issue_with_opts(temp: &TempDir, type_: &str, title: &str, opts: &[&str]) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title);
    for opt in opts {
        cmd.arg(opt);
    }
    cmd.arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Type Tests
// =============================================================================

#[test]
fn new_default_creates_task() {
    let temp = init_temp();
    wk().arg("new")
        .arg("My task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[task]"));
}

#[test]
fn new_explicit_task() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("My explicit task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[task]"));
}

#[test]
fn new_feature() {
    let temp = init_temp();
    wk().arg("new")
        .arg("feature")
        .arg("My feature")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[feature]"));
}

#[test]
fn new_bug() {
    let temp = init_temp();
    wk().arg("new")
        .arg("bug")
        .arg("My bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[bug]"));
}

#[test]
fn new_chore() {
    let temp = init_temp();
    wk().arg("new")
        .arg("chore")
        .arg("My chore")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[chore]"));
}

#[test]
fn new_idea() {
    let temp = init_temp();
    wk().arg("new")
        .arg("idea")
        .arg("My idea")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[idea]"));
}

#[test]
fn new_starts_in_todo_status() {
    let temp = init_temp();
    wk().arg("new")
        .arg("Status task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("(todo)"));
}

// =============================================================================
// Label Tests
// =============================================================================

#[test]
fn new_with_single_label() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Labeled task", &["--label", "project:auth"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth"));
}

#[test]
fn new_with_multiple_labels() {
    let temp = init_temp();
    let id = create_issue_with_opts(
        &temp,
        "task",
        "Multi-labeled",
        &["--label", "project:auth", "--label", "priority:high"],
    );

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth"))
        .stdout(predicate::str::contains("priority:high"));
}

#[test]
fn new_with_comma_separated_labels() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Comma labels", &["--label", "a,b,c"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Labels: a, b, c"));
}

#[test]
fn new_comma_labels_trim_whitespace() {
    let temp = init_temp();
    let id =
        create_issue_with_opts(&temp, "task", "Whitespace labels", &["--label", "  x  ,  y  "]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Labels: x, y"));
}

#[test]
fn new_with_mixed_labels() {
    let temp = init_temp();
    let id =
        create_issue_with_opts(&temp, "task", "Mixed labels", &["--label", "a,b", "--label", "c"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a"))
        .stdout(predicate::str::contains("b"))
        .stdout(predicate::str::contains("c"));
}

// =============================================================================
// Note/Description Tests
// =============================================================================

#[test]
fn new_with_note_adds_description() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Noted task", &["--note", "Initial context"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial context"));
}

#[test]
fn new_with_description_adds_note() {
    let temp = init_temp();
    let id = create_issue_with_opts(
        &temp,
        "task",
        "Described task",
        &["--description", "My description"],
    );

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Description:"))
        .stdout(predicate::str::contains("My description"));
}

// =============================================================================
// Priority Tests
// =============================================================================

#[test]
fn new_with_priority_0() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Critical task", &["--priority", "0"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("priority:0"));
}

#[test]
fn new_with_priority_4() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Low priority", &["--priority", "4"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("priority:4"));
}

#[test]
fn new_priority_and_label_combine() {
    let temp = init_temp();
    let id = create_issue_with_opts(
        &temp,
        "task",
        "Labeled priority",
        &["--priority", "1", "--label", "backend"],
    );

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("priority:1"))
        .stdout(predicate::str::contains("backend"));
}

#[test]
fn new_invalid_priority_5_fails() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Invalid")
        .arg("--priority")
        .arg("5")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn new_invalid_priority_negative_fails() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Invalid")
        .arg("--priority")
        .arg("-1")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn new_invalid_priority_text_fails() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Invalid")
        .arg("--priority")
        .arg("high")
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Validation Tests
// =============================================================================

#[test]
fn new_requires_title() {
    let temp = init_temp();
    wk().arg("new").current_dir(temp.path()).assert().failure();
}

#[test]
fn new_empty_title_fails() {
    let temp = init_temp();
    wk().arg("new").arg("").current_dir(temp.path()).assert().failure();
}

#[test]
fn new_invalid_type_fails() {
    let temp = init_temp();
    wk().arg("new").arg("bogus").arg("My bogus").current_dir(temp.path()).assert().failure();
}

// =============================================================================
// Prefix Tests
// =============================================================================

#[test]
fn new_id_uses_configured_prefix() {
    let temp = TempDir::new().unwrap();
    wk().arg("init").arg("--prefix").arg("myproj").current_dir(temp.path()).assert().success();

    wk().arg("new")
        .arg("Test task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("myproj-"));
}

#[test]
fn new_prefix_flag_overrides_config() {
    let temp = TempDir::new().unwrap();
    wk().arg("init").arg("--prefix").arg("main").current_dir(temp.path()).assert().success();

    let output = wk()
        .arg("new")
        .arg("Task")
        .arg("--prefix")
        .arg("other")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(id.starts_with("other-"), "Expected other- prefix, got: {}", id);
}

#[test]
fn new_prefix_short_flag() {
    let temp = TempDir::new().unwrap();
    wk().arg("init").arg("--prefix").arg("main").current_dir(temp.path()).assert().success();

    let output = wk()
        .arg("new")
        .arg("Task")
        .arg("-p")
        .arg("short")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(id.starts_with("short-"), "Expected short- prefix, got: {}", id);
}

#[test]
fn new_id_format_prefix_hex() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Test")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // ID format: prefix-xxxx where xxxx is hex
    // Validate without regex: should have format like "test-abc123"
    let parts: Vec<&str> = id.splitn(2, '-').collect();
    assert_eq!(parts.len(), 2, "ID should have prefix-suffix format: {}", id);
    assert!(
        parts[0].chars().all(|c| c.is_ascii_lowercase()),
        "Prefix should be lowercase letters: {}",
        id
    );
    assert!(
        parts[1].chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "Suffix should be lowercase hex: {}",
        id
    );
}

// =============================================================================
// Output Format Tests
// =============================================================================

#[test]
fn new_output_id_only() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("ID output task")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-"), "Should output ID");
    assert!(!stdout.contains("Created"), "Should NOT contain verbose message");
    assert!(!stdout.contains("[task]"), "Should NOT contain type tag");
}

#[test]
fn new_output_ids_alias() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("IDs alias task")
        .arg("-o")
        .arg("ids")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-"), "Should output ID");
    assert!(!stdout.contains("Created"), "Should NOT contain verbose message");
}

#[test]
fn new_output_json_valid() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("JSON task")
        .arg("--label")
        .arg("test:json")
        .arg("-o")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(json.get("id").is_some(), "Should have id field");
    assert_eq!(json.get("type").and_then(|v| v.as_str()), Some("task"));
    assert_eq!(json.get("title").and_then(|v| v.as_str()), Some("JSON task"));
    assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("todo"));
    assert!(json
        .get("labels")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|l| l.as_str() == Some("test:json")))
        .unwrap_or(false));
}

#[test]
fn new_output_id_scripting_workflow() {
    let temp = init_temp();

    // Create issue and capture ID
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Scripted task")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!id.is_empty(), "ID should not be empty");

    // Use ID in subsequent command
    wk().arg("label").arg(&id).arg("scripted").current_dir(temp.path()).assert().success();

    // Verify label was added
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("scripted"));
}

#[test]
fn new_default_text_output_includes_message() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Text task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created [task]"));
}

#[test]
fn new_prefix_with_other_flags() {
    let temp = TempDir::new().unwrap();
    wk().arg("init").arg("--prefix").arg("main").current_dir(temp.path()).assert().success();

    let output = wk()
        .arg("new")
        .arg("bug")
        .arg("Bug")
        .arg("--prefix")
        .arg("api")
        .arg("--label")
        .arg("urgent")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(id.starts_with("api-"), "Expected api- prefix, got: {}", id);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"));
}
