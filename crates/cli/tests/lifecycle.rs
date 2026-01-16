// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

#[test]
fn new_creates_issue() {
    let temp = init_temp();

    wk().arg("new")
        .arg("Test issue")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created [task]"))
        .stdout(predicate::str::contains("test-"));
}

#[test]
fn new_with_type() {
    let temp = init_temp();

    wk().arg("new")
        .arg("feature")
        .arg("My Feature")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created [feature]"))
        .stdout(predicate::str::contains("test-"));
}

#[test]
fn new_empty_title_fails() {
    let temp = init_temp();

    // Empty string title should fail
    wk().arg("new")
        .arg("")
        .current_dir(temp.path())
        .assert()
        .failure();

    // Whitespace-only title should fail
    wk().arg("new")
        .arg("   ")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn new_invalid_type_as_title() {
    let temp = init_temp();

    // "unknown" is not a valid type, so it becomes the title
    wk().arg("new")
        .arg("unknown")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[task]")) // Default type
        .stdout(predicate::str::contains("unknown")); // As title
}

#[test]
fn new_with_multiple_labels() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Multi-labeled issue")
        .arg("-l")
        .arg("urgent")
        .arg("-l")
        .arg("frontend")
        .arg("-l")
        .arg("v1.0")
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

    // Show should display all labels
    wk().arg("show")
        .arg(id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"))
        .stdout(predicate::str::contains("frontend"))
        .stdout(predicate::str::contains("v1.0"));
}

#[test]
fn lifecycle_start_reopen_done() {
    let temp = init_temp();
    let id = create_issue(&temp, "Task to complete");

    // Start
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Started"));

    // Reopen (from in_progress to todo, no reason needed)
    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Reopened"));

    // Start again
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Done
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Completed"));
}

#[test]
fn done_with_reason_from_todo() {
    let temp = init_temp();
    let id = create_issue(&temp, "Task to complete directly");

    // Done from todo without reason should fail (requires reason for agents)
    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--reason is required"));

    // Done from todo with reason should succeed
    wk().arg("done")
        .arg(&id)
        .arg("--reason")
        .arg("Already done externally")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Completed"))
        .stdout(predicate::str::contains("Already done externally"));
}

#[test]
fn close_requires_reason() {
    let temp = init_temp();
    let id = create_issue(&temp, "Issue to close");

    // Close without reason should fail
    wk().arg("close")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();

    // Close with reason should succeed
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Closed"));
}

#[test]
fn cannot_start_from_done_state() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test task");

    // Start -> Done
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Start from done should fail
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn cannot_start_from_closed_state() {
    let temp = init_temp();
    let id = create_issue(&temp, "Test task");

    // Close with reason
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("not needed")
        .current_dir(temp.path())
        .assert()
        .success();

    // Start from closed should fail
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn reopen_from_done() {
    let temp = init_temp();
    let id = create_issue(&temp, "Task to reopen");

    // Complete the task
    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Reopen requires reason from done state
    wk().arg("reopen")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();

    // Reopen with reason should succeed
    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg("Found regression")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Reopened"));

    // Issue should now be todo (reopen goes to todo, not in_progress)
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
}
