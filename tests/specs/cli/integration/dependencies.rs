// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for dependency integration behaviors.
//! Converted from tests/specs/cli/integration/dependencies.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn wk() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("wok").unwrap()
}

fn init_temp() -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_issue_with_label(temp: &TempDir, type_: &str, title: &str, label: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new")
        .arg(type_)
        .arg(title)
        .arg("--label")
        .arg(label)
        .arg("-o")
        .arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// List filtering with blocked issues
// =============================================================================

#[test]
fn blocks_relationship_affects_list_blocked_view() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Task A");
    let b = create_issue(&temp, "task", "Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    // Default list shows both blocked and unblocked issues
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&a))
        .stdout(predicate::str::contains(&b));

    // --blocked shows only blocked issues
    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&a).not())
        .stdout(predicate::str::contains(&b));
}

#[test]
fn list_blocked_shows_blocked_issues() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Task A");
    let b = create_issue(&temp, "task", "Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&b));
}

#[test]
fn list_all_shows_all_issues() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Task A");
    let b = create_issue(&temp, "task", "Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&a))
        .stdout(predicate::str::contains(&b));
}

// =============================================================================
// Completing blockers
// =============================================================================

#[test]
fn completing_blocker_unblocks_dependent() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Task A");
    let b = create_issue(&temp, "task", "Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("start")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&b));
}

// =============================================================================
// Transitive blocking
// =============================================================================

#[test]
fn transitive_blocking_works() {
    let temp = init_temp();
    let a = create_issue_with_label(&temp, "task", "Task A", "test:trans-block");
    let b = create_issue_with_label(&temp, "task", "Task B", "test:trans-block");
    let c = create_issue_with_label(&temp, "task", "Task C", "test:trans-block");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    // list shows all open issues
    wk().arg("list")
        .arg("--label")
        .arg("test:trans-block")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&a))
        .stdout(predicate::str::contains(&b))
        .stdout(predicate::str::contains(&c));

    // ready shows only unblocked (A), not B or C
    wk().arg("ready")
        .arg("--label")
        .arg("test:trans-block")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&a))
        .stdout(predicate::str::contains(&b).not())
        .stdout(predicate::str::contains(&c).not());
}

#[test]
fn transitive_blocking_unblocks_in_chain() {
    let temp = init_temp();
    let a = create_issue_with_label(&temp, "task", "Task A", "test:trans-unblock");
    let b = create_issue_with_label(&temp, "task", "Task B", "test:trans-unblock");
    let c = create_issue_with_label(&temp, "task", "Task C", "test:trans-unblock");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    // Complete A - B becomes ready, C still blocked
    wk().arg("start")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("ready")
        .arg("--label")
        .arg("test:trans-unblock")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&b))
        .stdout(predicate::str::contains(&c).not());

    // Complete B - C becomes ready
    wk().arg("start")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("ready")
        .arg("--label")
        .arg("test:trans-unblock")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&c));
}

// =============================================================================
// Tracks relationship
// =============================================================================

#[test]
fn tracks_creates_tracks_relationship() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "My Feature");
    let task = create_issue(&temp, "task", "My Task");

    wk().arg("dep")
        .arg(&feature)
        .arg("tracks")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&feature)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracks"))
        .stdout(predicate::str::contains(&task));

    wk().arg("show")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracked by"))
        .stdout(predicate::str::contains(&feature));
}

#[test]
fn tracks_does_not_block() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "My Feature");
    let task = create_issue(&temp, "task", "My Task");

    wk().arg("dep")
        .arg(&feature)
        .arg("tracks")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success();

    // Task should be ready (tracks doesn't block)
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&task));
}

// =============================================================================
// Multiple blockers
// =============================================================================

#[test]
fn multiple_blockers_all_must_complete() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Blocker A");
    let b = create_issue(&temp, "task", "Blocker B");
    let c = create_issue(&temp, "task", "Blocked");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    // C is blocked
    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&c));

    // Complete A - C still blocked by B
    wk().arg("start")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&c));

    // Complete B - C now ready
    wk().arg("start")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&c));
}

// =============================================================================
// Blocking and status transitions
// =============================================================================

#[test]
fn blocking_does_not_prevent_status_transitions() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Blocker");
    let b = create_issue(&temp, "task", "Blocked");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    // Can still start blocked issue (informational only)
    wk().arg("start")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Dependencies persist after completion
// =============================================================================

#[test]
fn dependencies_remain_after_completion() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Task A");
    let b = create_issue(&temp, "task", "Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("start")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("done")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success();

    // Dependency still exists in show output
    wk().arg("show")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocks"))
        .stdout(predicate::str::contains(&b));
}

// =============================================================================
// Show displays blockers
// =============================================================================

#[test]
fn show_displays_all_blockers() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Blocker A");
    let b = create_issue(&temp, "task", "Blocker B");
    let c = create_issue(&temp, "task", "Blocked");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked by"))
        .stdout(predicate::str::contains(&a))
        .stdout(predicate::str::contains(&b));
}
