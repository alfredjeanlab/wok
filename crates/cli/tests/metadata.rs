// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

#[test]
fn labels() {
    let temp = init_temp();
    let id = create_issue(&temp, "Labeled issue");

    // Add label
    wk().arg("label")
        .arg(&id)
        .arg("project:auth")
        .current_dir(temp.path())
        .assert()
        .success();

    // Show should display label
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth"));

    // List with label filter
    wk().arg("list")
        .arg("--label")
        .arg("project:auth")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Labeled issue"));
}

#[test]
fn notes() {
    let temp = init_temp();
    let id = create_issue(&temp, "Issue with notes");

    // Add note
    wk().arg("note")
        .arg(&id)
        .arg("This is a note")
        .current_dir(temp.path())
        .assert()
        .success();

    // Show should display note
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("This is a note"));
}

#[test]
fn special_characters_in_labels() {
    let temp = init_temp();
    let id = create_issue(&temp, "Label test");

    // Add labels with special characters
    wk().arg("label")
        .arg(&id)
        .arg("project:auth-v2")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("label")
        .arg(&id)
        .arg("priority_high")
        .current_dir(temp.path())
        .assert()
        .success();

    // Show should display labels
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth-v2"))
        .stdout(predicate::str::contains("priority_high"));
}

#[test]
fn unlabel_nonexistent_label() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test issue");

    // Removing a label that doesn't exist should not fail
    // (idempotent operation)
    wk().arg("unlabel")
        .arg(&id)
        .arg("nonexistent")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn edit_title_and_description() {
    let temp = init_temp();
    let id = create_issue(&temp, "Original title");

    // Edit title
    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg("New title")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated title"));

    // Edit description
    wk().arg("edit")
        .arg(&id)
        .arg("description")
        .arg("This is a description")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated description"));

    // Show should reflect changes
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"));
}

#[test]
fn edit_does_not_show_old_title_in_log() {
    let temp = init_temp();
    let id = create_issue(&temp, "Original title");

    // Edit title
    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg("New title")
        .current_dir(temp.path())
        .assert()
        .success();

    // Show should NOT contain "Original title" anywhere
    let show_output = wk()
        .arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .output()
        .unwrap();

    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(
        !show_stdout.contains("Original title"),
        "Show output should not contain old title. Got: {}",
        show_stdout
    );
    assert!(show_stdout.contains("New title"));
}

#[test]
fn edit_title_then_description() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("task")
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

    // Edit title first
    wk().arg("edit")
        .arg(id)
        .arg("title")
        .arg("New title")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated title"));

    // Then edit description
    wk().arg("edit")
        .arg(id)
        .arg("description")
        .arg("New description")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated description"));

    // Verify the changes
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"));
}

#[test]
fn edit_without_id_shows_help() {
    let temp = init_temp();
    create_issue(&temp, "Test issue");

    // Running edit without ID should show help/error
    wk().arg("edit")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn edit_nonexistent_issue_fails() {
    let temp = init_temp();

    // Edit nonexistent issue should fail
    wk().arg("edit")
        .arg("nonexistent")
        .arg("description")
        .arg("Some description")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn edit_type_changes_type() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test issue");

    // type attr should change issue type
    wk().arg("edit")
        .arg(&id)
        .arg("type")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated type"));

    // Verify the type was changed (shown as [bug] in output)
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[bug]"));
}

#[test]
fn unicode_in_title_and_notes() {
    let temp = init_temp();

    // Create issue with emoji and CJK characters
    let output = wk()
        .arg("new")
        .arg("Fix bug \u{1F41B} in \u{65E5}\u{672C}\u{8A9E} module")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .find(|s| s.starts_with("test-"))
        .unwrap()
        .trim_end_matches(':');

    // Add note with unicode
    wk().arg("note")
        .arg(id)
        .arg("Note with emoji \u{2705} and accents: caf\u{E9} r\u{E9}sum\u{E9}")
        .current_dir(temp.path())
        .assert()
        .success();

    // Show should display unicode correctly
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{65E5}\u{672C}\u{8A9E}"))
        .stdout(predicate::str::contains("caf\u{E9}"));
}

#[test]
fn list_shows_issues() {
    let temp = init_temp();

    wk().arg("new")
        .arg("First issue")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First issue"));
}
