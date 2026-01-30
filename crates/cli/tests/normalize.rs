// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

#[test]
fn new_trims_title_whitespace() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("  Padded title  ")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    // Extract ID and check show output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .find(|s| s.starts_with("test-"))
        .unwrap()
        .trim_end_matches(':');

    // Show should display trimmed title
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Padded title"));
}

#[test]
fn note_trims_content() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test issue");

    wk().arg("note")
        .arg(&id)
        .arg("  Note with padding  ")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Note with padding"));
}

#[test]
fn edit_trims_description() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test issue");

    wk().arg("edit")
        .arg(&id)
        .arg("description")
        .arg("  Description with spaces  ")
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify trimmed
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Description with spaces"));
}

#[test]
fn close_trims_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test issue");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("  Duplicate issue  ")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Duplicate issue"));
}

#[test]
fn new_splits_title_on_double_newline() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("Fix the bug\n\nThis is a detailed description")
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

    // Show should display both title and extracted description
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix the bug"))
        .stdout(predicate::str::contains("detailed description"));
}

#[test]
fn new_no_split_before_threshold() {
    let temp = init_temp();

    // "Hi" is only 1 word, 2 chars - below threshold
    let output = wk()
        .arg("new")
        .arg("Hi\n\nthere")
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

    // Title should be "Hi there" (no split, newlines collapsed)
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi there"));
}

#[test]
fn new_split_prepends_to_existing_note() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("Fix the bug\n\nExtracted description")
        .arg("--note")
        .arg("Explicit note content")
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

    // Should see both extracted description and explicit note
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Extracted description"))
        .stdout(predicate::str::contains("Explicit note content"));
}

#[test]
fn new_collapses_unquoted_newlines() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("Line one\nLine two")
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

    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Line one Line two"));
}

#[test]
fn new_escapes_quoted_newlines() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("Error: \"line1\nline2\"")
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

    // The newline in quotes should be escaped as \n
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\\n"));
}

#[test]
fn new_collapses_consecutive_spaces() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("Word   with   spaces")
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

    // Should have single spaces
    let show_output = wk()
        .arg("show")
        .arg(id)
        .current_dir(temp.path())
        .output()
        .unwrap();

    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(show_stdout.contains("Word with spaces"));
    // Ensure no double spaces in title
    assert!(!show_stdout.contains("Word  "));
}

#[test]
fn edit_title_normalizes() {
    let temp = init_temp();
    let id = create_issue(&temp, "Original");

    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg("  New  title  ")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"));
}

#[test]
fn edit_title_with_double_newline_normalizes() {
    let temp = init_temp();
    let id = create_issue(&temp, "Original title");

    // Editing title with double-newline normalizes like `new` command:
    // splits at \n\n, uses first part as title, adds second part as note
    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg("Fix the bug\n\nWith description")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix the bug"));

    // Verify the title was set and description was added as a note
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Title: Fix the bug"))
        .stdout(predicate::str::contains("With description"));
}

#[test]
fn empty_note_after_trim_rejected() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test issue");

    // Whitespace-only note should fail
    wk().arg("note")
        .arg(&id)
        .arg("   ")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn empty_reason_after_trim_rejected() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test issue");

    // Whitespace-only reason should fail
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("   ")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}
