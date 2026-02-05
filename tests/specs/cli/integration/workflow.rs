// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for workflow integration tests.
//! Converted from tests/specs/cli/integration/workflow.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let output = wk()
        .args(["new", type_, title, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_issue_with_label(temp: &TempDir, type_: &str, title: &str, label: &str) -> String {
    let output = wk()
        .args(["new", type_, title, "--label", label, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Full Workflow Test from DESIGN.md
// =============================================================================

#[test]
fn complete_workflow_from_design_md() {
    let temp = init_temp();

    // 2. Create issues
    let feature = create_issue_with_label(&temp, "feature", "Build auth system", "project:auth");
    let schema = create_issue_with_label(&temp, "task", "Design database schema", "project:auth");
    let login = create_issue_with_label(&temp, "task", "Implement login endpoint", "project:auth");
    let hash = create_issue_with_label(&temp, "bug", "Fix password hashing", "priority:high");

    // Verify all created
    assert!(!feature.is_empty());
    assert!(!schema.is_empty());
    assert!(!login.is_empty());
    assert!(!hash.is_empty());

    // 3. Set up dependencies
    wk().args(["dep", &feature, "tracks", &schema, &login])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["dep", &schema, "blocks", &login])
        .current_dir(temp.path())
        .assert()
        .success();

    // 4. Verify blocking - login should be blocked
    // list shows all open issues (blocked and unblocked)
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&login));

    // ready shows only unblocked issues
    wk().arg("ready")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&login).not());

    wk().args(["list", "--blocked"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&login));

    // 5. Start work and add notes
    wk().args(["start", &schema])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["note", &schema, "Using PostgreSQL with normalized schema"])
        .current_dir(temp.path())
        .assert()
        .success();

    // 6. Test reopen (return to backlog)
    wk().args(["reopen", &schema])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&schema));

    wk().args(["start", &schema])
        .current_dir(temp.path())
        .assert()
        .success();

    // 7. Complete work
    wk().args(["done", &schema])
        .current_dir(temp.path())
        .assert()
        .success();

    // Login should now be unblocked
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&login));

    wk().args(["start", &login])
        .current_dir(temp.path())
        .assert()
        .success();

    // 8. Test close/reopen
    wk().args(["close", &hash, "--reason", "not a bug, works as designed"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["reopen", &hash, "--reason", "actually is a bug, reproduced"])
        .current_dir(temp.path())
        .assert()
        .success();

    // 9. Verify log
    wk().args(["log", &schema])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("created"))
        .stdout(predicate::str::contains("started"));

    wk().args(["log", &hash])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("closed"))
        .stdout(predicate::str::contains("reopened"));

    // 10. Test filtering
    wk().args(["list", "--label", "project:auth"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["list", "--status", "done"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&schema));

    // 11. Test edit
    wk().args(["edit", &hash, "title", "Fix password hashing (revised)"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["edit", &hash, "type", "task"])
        .current_dir(temp.path())
        .assert()
        .success();

    // 12. View issue with deps
    wk().args(["show", &feature])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracks"));

    wk().args(["show", &login])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracked by"));

    wk().args(["tree", &feature])
        .current_dir(temp.path())
        .assert()
        .success();

    // 13. Test export
    let export_path = temp.path().join("issues.jsonl");
    wk().args(["export", export_path.to_str().unwrap()])
        .current_dir(temp.path())
        .assert()
        .success();
    assert!(export_path.exists());
}

// =============================================================================
// Concurrent Work Tests
// =============================================================================

#[test]
fn workflow_handles_concurrent_work() {
    let temp = init_temp();
    let t1 = create_issue(&temp, "task", "Task 1");
    let t2 = create_issue(&temp, "task", "Task 2");
    let t3 = create_issue(&temp, "task", "Task 3");

    // Start multiple tasks
    wk().args(["start", &t1])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["start", &t2])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["list", "--status", "in_progress"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&t1))
        .stdout(predicate::str::contains(&t2))
        .stdout(predicate::str::contains(&t3).not());
}

// =============================================================================
// Feature Hierarchy Tests
// =============================================================================

#[test]
fn workflow_with_feature_hierarchy() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "Main Feature");
    let t1 = create_issue(&temp, "task", "Task 1");
    let t2 = create_issue(&temp, "task", "Task 2");
    let t3 = create_issue(&temp, "task", "Task 3");

    // Feature tracks all tasks
    wk().args(["dep", &feature, "tracks", &t1, &t2, &t3])
        .current_dir(temp.path())
        .assert()
        .success();

    // T1 blocks T2, T2 blocks T3
    wk().args(["dep", &t1, "blocks", &t2])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["dep", &t2, "blocks", &t3])
        .current_dir(temp.path())
        .assert()
        .success();

    // Only T1 should be ready (unblocked)
    wk().arg("ready")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&t1))
        .stdout(predicate::str::contains(&t2).not())
        .stdout(predicate::str::contains(&t3).not());

    // Complete T1
    wk().args(["start", &t1])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["done", &t1])
        .current_dir(temp.path())
        .assert()
        .success();

    // Now T2 should be ready
    wk().arg("ready")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&t2))
        .stdout(predicate::str::contains(&t3).not());

    // Complete T2
    wk().args(["start", &t2])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["done", &t2])
        .current_dir(temp.path())
        .assert()
        .success();

    // Now T3 should be ready
    wk().arg("ready")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&t3));
}

// =============================================================================
// Issue Type Lifecycle Tests
// =============================================================================

#[parameterized(
    task = { "task" },
    bug = { "bug" },
    feature = { "feature" },
    chore = { "chore" },
)]
fn issue_type_full_lifecycle(issue_type: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, issue_type, &format!("Test {} issue", issue_type));

    // Verify type
    let type_marker = format!("[{}]", issue_type);
    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&type_marker))
        .stdout(predicate::str::contains("Status: todo"));

    // Start work
    wk().args(["start", &id])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));

    // Complete work
    wk().args(["done", &id])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: done"));
}

// =============================================================================
// Prime Command Integration Tests
// =============================================================================

#[test]
fn prime_works_alongside_initialized_project() {
    let temp = init_temp();
    // Should still output template even with .wok/ present
    wk().arg("prime")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("## Core Rules"))
        .stdout(predicate::str::contains("## Finding Work"));
}

#[test]
fn prime_output_can_be_piped_to_file() {
    let temp = TempDir::new().unwrap();
    let template_path = temp.path().join("template.md");

    // Run prime and capture output to file
    let output = wk().arg("prime").current_dir(temp.path()).output().unwrap();

    assert!(output.status.success());
    std::fs::write(&template_path, &output.stdout).unwrap();

    // Verify file exists and has content
    assert!(template_path.exists());
    let content = std::fs::read_to_string(&template_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("## Core Rules"));
}

#[test]
fn prime_output_is_consistent_across_calls() {
    let temp = TempDir::new().unwrap();

    let first_output = wk().arg("prime").current_dir(temp.path()).output().unwrap();
    let first_stdout = String::from_utf8_lossy(&first_output.stdout).to_string();

    let second_output = wk().arg("prime").current_dir(temp.path()).output().unwrap();
    let second_stdout = String::from_utf8_lossy(&second_output.stdout).to_string();

    assert_eq!(first_stdout, second_stdout);
}
