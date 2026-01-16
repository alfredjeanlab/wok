// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

#[test]
fn list_empty_database() {
    let temp = init_temp();

    // List should succeed even with no issues
    wk().arg("list").current_dir(temp.path()).assert().success();
}

#[test]
fn list_status_or_filter() {
    let temp = init_temp();

    // Create issues with different statuses
    wk().arg("new")
        .arg("Todo task")
        .current_dir(temp.path())
        .assert()
        .success();

    let id2 = create_issue(&temp, "Started task");
    let id3 = create_issue(&temp, "Done task");

    // Start second task
    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    // Complete third task
    wk().arg("start")
        .arg(&id3)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id3)
        .current_dir(temp.path())
        .assert()
        .success();

    // Filter by todo OR in_progress should show first two, not the done one
    let result = wk()
        .arg("list")
        .arg("-s")
        .arg("todo,in_progress")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("Todo task"), "Should show todo task");
    assert!(
        stdout.contains("Started task"),
        "Should show in_progress task"
    );
    assert!(!stdout.contains("Done task"), "Should NOT show done task");
}

#[test]
fn list_label_and_filter() {
    let temp = init_temp();

    // Create issues with different label combinations
    wk().arg("new")
        .arg("Has only label A")
        .arg("-l")
        .arg("labelA")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Has only label B")
        .arg("-l")
        .arg("labelB")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Has both labels")
        .arg("-l")
        .arg("labelA")
        .arg("-l")
        .arg("labelB")
        .current_dir(temp.path())
        .assert()
        .success();

    // Filter by labelA AND labelB (repeated flags)
    let result = wk()
        .arg("list")
        .arg("-l")
        .arg("labelA")
        .arg("-l")
        .arg("labelB")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Has both labels"),
        "Should show issue with both labels"
    );
    assert!(
        !stdout.contains("Has only label A"),
        "Should NOT show issue with only labelA"
    );
    assert!(
        !stdout.contains("Has only label B"),
        "Should NOT show issue with only labelB"
    );
}

#[test]
fn list_label_or_filter() {
    let temp = init_temp();

    // Create issues with different labels
    wk().arg("new")
        .arg("Frontend issue")
        .arg("-l")
        .arg("frontend")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Backend issue")
        .arg("-l")
        .arg("backend")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Database issue")
        .arg("-l")
        .arg("database")
        .current_dir(temp.path())
        .assert()
        .success();

    // Filter by frontend OR backend (comma-separated)
    let result = wk()
        .arg("list")
        .arg("-l")
        .arg("frontend,backend")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Frontend issue"),
        "Should show frontend issue"
    );
    assert!(
        stdout.contains("Backend issue"),
        "Should show backend issue"
    );
    assert!(
        !stdout.contains("Database issue"),
        "Should NOT show database issue"
    );
}

#[test]
fn list_combined_filters() {
    let temp = init_temp();

    // Create issues with various label combinations
    wk().arg("new")
        .arg("Issue A with urgent")
        .arg("-l")
        .arg("a")
        .arg("-l")
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Issue B with urgent")
        .arg("-l")
        .arg("b")
        .arg("-l")
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Issue A without urgent")
        .arg("-l")
        .arg("a")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Issue C with urgent")
        .arg("-l")
        .arg("c")
        .arg("-l")
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();

    // Filter: (label a OR label b) AND label urgent
    let result = wk()
        .arg("list")
        .arg("-l")
        .arg("a,b")
        .arg("-l")
        .arg("urgent")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Issue A with urgent"),
        "Should show A+urgent"
    );
    assert!(
        stdout.contains("Issue B with urgent"),
        "Should show B+urgent"
    );
    assert!(
        !stdout.contains("Issue A without urgent"),
        "Should NOT show A without urgent"
    );
    assert!(
        !stdout.contains("Issue C with urgent"),
        "Should NOT show C+urgent (not a or b)"
    );
}

#[test]
fn list_all_statuses() {
    let temp = init_temp();

    // Create and complete an issue
    let id = create_issue(&temp, "Completed task");

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

    // Create another issue
    wk().arg("new")
        .arg("Todo task")
        .current_dir(temp.path())
        .assert()
        .success();

    // List without status filter shows only open (todo + in_progress), not done
    let result = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        !stdout.contains("Completed task"),
        "Done issues should not show by default"
    );
    assert!(
        stdout.contains("Todo task"),
        "Todo issues should show by default"
    );

    // List with -s done should show done issues
    let result = wk()
        .arg("list")
        .arg("-s")
        .arg("done")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Completed task"),
        "Done issues should show with -s done"
    );
    assert!(
        !stdout.contains("Todo task"),
        "Todo issues should not show with -s done"
    );
}

#[test]
fn log_with_limit() {
    let temp = init_temp();

    // Create multiple issues to generate events
    for i in 0..5 {
        wk().arg("new")
            .arg(format!("Issue {}", i))
            .current_dir(temp.path())
            .assert()
            .success();
    }

    // Log with limit should only show limited events
    let result = wk()
        .arg("log")
        .arg("--limit")
        .arg("2")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    let lines: Vec<_> = stdout.lines().filter(|l| l.contains("created")).collect();
    assert!(
        lines.len() <= 2,
        "Should only show 2 events, got {}",
        lines.len()
    );
}
