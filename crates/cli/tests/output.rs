// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

#[test]
fn show_omits_log_for_new_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test task");

    // Show should NOT include Log section for newly created issue
    // (created event is redundant with Created: line)
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Log:").not())
        .stdout(predicate::str::contains("created").not());
}

#[test]
fn show_includes_log_for_issue_with_activity() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test task");

    // Add a label to generate an event
    wk().arg("label")
        .arg(&id)
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();

    // Show should include Log section with labeled event, but not created
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Log:"))
        .stdout(predicate::str::contains("labeled"))
        .stdout(predicate::str::contains("  created").not());
}

#[test]
fn show_nonexistent_issue() {
    let temp = init_temp();

    wk().arg("show")
        .arg("test-nonexistent")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn show_json_format() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Test JSON output")
        .arg("-l")
        .arg("backend")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .find(|s| s.starts_with("test-"))
        .unwrap()
        .trim_end_matches(':');

    // Show with JSON output should output valid compact JSONL
    wk().arg("show")
        .arg(id)
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"))
        .stdout(predicate::str::contains("\"title\":\"Test JSON output\""))
        .stdout(predicate::str::contains("\"issue_type\":\"task\""))
        .stdout(predicate::str::contains("\"status\":\"todo\""))
        .stdout(predicate::str::contains("\"labels\":"))
        .stdout(predicate::str::contains("\"backend\""))
        .stdout(predicate::str::contains("\"events\":"));
}

#[test]
fn show_json_output_short_flag() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test short flag");

    // Short flag -o should work
    wk().arg("show")
        .arg(&id)
        .arg("-o")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"));
}

#[test]
fn show_invalid_output() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test invalid output");

    // Invalid output format should fail
    wk().arg("show")
        .arg(&id)
        .arg("--output")
        .arg("xml")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown format"));
}

#[test]
fn tree_shows_hierarchy() {
    let temp = init_temp();

    // Create feature
    let output = wk()
        .arg("new")
        .arg("feature")
        .arg("Parent Feature")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let feature_id = stdout
        .split_whitespace()
        .find(|s| s.starts_with("test-"))
        .unwrap()
        .trim_end_matches(':');

    // Create task
    let task_id = create_issue(&temp, "Child Task");

    // Link feature tracks task
    wk().arg("dep")
        .arg(feature_id)
        .arg("tracks")
        .arg(&task_id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Tree should show hierarchy
    wk().arg("tree")
        .arg(feature_id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Parent Feature"))
        .stdout(predicate::str::contains("Child Task"))
        .stdout(predicate::str::contains("\u{2514}\u{2500}\u{2500}"));
}

#[test]
fn export_creates_jsonl() {
    let temp = init_temp();
    let export_path = temp.path().join("export.jsonl");

    wk().arg("new")
        .arg("Issue to export")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("export")
        .arg(export_path.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported 1 issues"));

    assert!(export_path.exists());

    let content = std::fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("Issue to export"));
}

#[test]
fn completion_bash() {
    // Completion doesn't need an initialized project
    wk().arg("completion")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("_wk"));
}

#[test]
fn completion_zsh() {
    wk().arg("completion")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef"));
}

#[test]
fn completion_fish() {
    wk().arg("completion")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}
