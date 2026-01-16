// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

#[test]
fn dependency_blocking() {
    let temp = init_temp();

    // Create two issues
    let id1 = create_issue(&temp, "Blocker issue");
    let id2 = create_issue(&temp, "Blocked issue");

    // Add blocking dependency
    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    // List shows all open issues (both blocked and unblocked)
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocker issue"))
        .stdout(predicate::str::contains("Blocked issue"));

    // List --blocked should show only the blocked issue
    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked issue"));

    // Ready should only show the unblocked (blocker) issue
    wk().arg("ready")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocker issue"));
}

#[test]
fn cycle_detection() {
    let temp = init_temp();

    // Create three issues
    let id_a = create_issue(&temp, "Issue A");
    let id_b = create_issue(&temp, "Issue B");
    let id_c = create_issue(&temp, "Issue C");

    // A blocks B
    wk().arg("dep")
        .arg(&id_a)
        .arg("blocks")
        .arg(&id_b)
        .current_dir(temp.path())
        .assert()
        .success();

    // B blocks C
    wk().arg("dep")
        .arg(&id_b)
        .arg("blocks")
        .arg(&id_c)
        .current_dir(temp.path())
        .assert()
        .success();

    // C blocks A should fail (cycle)
    wk().arg("dep")
        .arg(&id_c)
        .arg("blocks")
        .arg(&id_a)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cycle"));
}

#[test]
fn self_dependency_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "Self-block test");

    // Self-blocking should fail
    wk().arg("dep")
        .arg(&id)
        .arg("blocks")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("self"));
}

#[test]
fn tree_shows_blocked_indicator() {
    let temp = init_temp();

    // Create blocker issue
    let blocker_id = create_issue(&temp, "Blocker");

    // Create blocked issue
    let blocked_id = create_issue(&temp, "Blocked task");

    // Set up blocking relationship
    wk().arg("dep")
        .arg(&blocker_id)
        .arg("blocks")
        .arg(&blocked_id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Tree of blocked issue should show "blocked"
    wk().arg("tree")
        .arg(&blocked_id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("blocked"));
}

#[test]
fn completing_blocker_logs_unblocked_event() {
    let temp = init_temp();

    // Create blocker issue
    let blocker_id = create_issue(&temp, "Blocker task");

    // Create blocked issue
    let blocked_id = create_issue(&temp, "Blocked task");

    // Set up blocking relationship
    wk().arg("dep")
        .arg(&blocker_id)
        .arg("blocks")
        .arg(&blocked_id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Complete the blocker
    wk().arg("start")
        .arg(&blocker_id)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&blocker_id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Log of blocked issue should show "unblocked" event
    wk().arg("log")
        .arg(&blocked_id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unblocked"));
}

#[test]
fn ready_with_all_blocked() {
    let temp = init_temp();

    // Create a blocker (in_progress) and a blocked issue
    let blocker_id = create_issue(&temp, "Blocker");
    let blocked_id = create_issue(&temp, "Blocked");

    // Start the blocker so it's not in todo
    wk().arg("start")
        .arg(&blocker_id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Block the other issue
    wk().arg("dep")
        .arg(&blocker_id)
        .arg("blocks")
        .arg(&blocked_id)
        .current_dir(temp.path())
        .assert()
        .success();

    // Ready should show "No ready issues"
    wk().arg("ready")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No ready issues"));
}
