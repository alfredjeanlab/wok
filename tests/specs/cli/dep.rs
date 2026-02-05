// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk dep` and `wk undep` commands.
//! Converted from tests/specs/cli/unit/dep.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use yare::parameterized;

fn wk() -> Command {
    cargo_bin_cmd!("wok")
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

// =============================================================================
// dep blocks - Creating blocking relationships
// =============================================================================

#[test]
fn dep_blocks_creates_relationship() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepBlocks Task A");
    let b = create_issue(&temp, "task", "DepBlocks Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn dep_blocks_default_list_shows_both() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepBlocksList Task A");
    let b = create_issue(&temp, "task", "DepBlocksList Task B");

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
        .stdout(predicate::str::contains("Task A"))
        .stdout(predicate::str::contains("Task B"));
}

#[test]
fn dep_blocks_blocked_flag_filters() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepBlocksFilter Task A");
    let b = create_issue(&temp, "task", "DepBlocksFilter Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    // --blocked filters to show only blocked issues
    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Task A").not())
        .stdout(predicate::str::contains("Task B"));
}

#[test]
fn dep_blocks_multiple_targets() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepBlocksMulti Task A");
    let b = create_issue(&temp, "task", "DepBlocksMulti Task B");
    let c = create_issue(&temp, "task", "DepBlocksMulti Task C");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    // Both B and C should be blocked
    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&b))
        .stdout(predicate::str::contains(&c));
}

// =============================================================================
// dep tracks - Creating parent-child relationships
// =============================================================================

#[test]
fn dep_tracks_creates_relationship() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "DepTracks Feature");
    let task = create_issue(&temp, "task", "DepTracks Task");

    wk().arg("dep")
        .arg(&feature)
        .arg("tracks")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn dep_tracks_shows_in_parent_output() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "DepTracksParent Feature");
    let task = create_issue(&temp, "task", "DepTracksParent Task");

    wk().arg("dep")
        .arg(&feature)
        .arg("tracks")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success();

    // Shows in parent show output
    wk().arg("show")
        .arg(&feature)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracks"))
        .stdout(predicate::str::contains(&task));
}

#[test]
fn dep_tracks_shows_in_child_output() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "DepTracksChild Feature");
    let task = create_issue(&temp, "task", "DepTracksChild Task");

    wk().arg("dep")
        .arg(&feature)
        .arg("tracks")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success();

    // Shows in child show output
    wk().arg("show")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracked by"))
        .stdout(predicate::str::contains(&feature));
}

// =============================================================================
// undep - Removing relationships
// =============================================================================

#[test]
fn undep_removes_blocking_relationship() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepUndepBlock Task A");
    let b = create_issue(&temp, "task", "DepUndepBlock Task B");

    // Create relationship
    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    // Remove relationship
    wk().arg("undep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    // B should no longer appear in blocked list
    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&b).not());
}

#[test]
fn undep_removes_tracks_relationship() {
    let temp = init_temp();
    let feature = create_issue(&temp, "feature", "DepUndepTrack Feature");
    let task = create_issue(&temp, "task", "DepUndepTrack Task");

    // Create relationship
    wk().arg("dep")
        .arg(&feature)
        .arg("tracks")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success();

    // Remove relationship
    wk().arg("undep")
        .arg(&feature)
        .arg("tracks")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success();

    // Should no longer show "Tracked by" in child output
    wk().arg("show")
        .arg(&task)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tracked by").not());
}

#[test]
fn undep_nonexistent_relationship_graceful() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepUndepNone Task A");
    let b = create_issue(&temp, "task", "DepUndepNone Task B");

    // Undep on a relationship that doesn't exist
    // Should either succeed (idempotent) or fail gracefully
    let result = wk()
        .arg("undep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert();

    // The command should complete (either success or failure is acceptable)
    // This matches the BATS test which just had `true` after the command
    let _ = result;
}

// =============================================================================
// dep error handling
// =============================================================================

#[test]
fn dep_to_self_fails() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepErr Self Task");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[parameterized(
    nonexistent_from = { "test-nonexistent", "to" },
    nonexistent_to = { "from", "test-nonexistent" },
)]
fn dep_nonexistent_id_fails(from_marker: &str, to_marker: &str) {
    let temp = init_temp();

    let from = if from_marker == "from" {
        create_issue(&temp, "task", "DepErr From Task")
    } else {
        from_marker.to_string()
    };

    let to = if to_marker == "to" {
        create_issue(&temp, "task", "DepErr To Task")
    } else {
        to_marker.to_string()
    };

    wk().arg("dep")
        .arg(&from)
        .arg("blocks")
        .arg(&to)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn dep_invalid_relationship_fails() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DepErr Invalid A");
    let b = create_issue(&temp, "task", "DepErr Invalid B");

    wk().arg("dep")
        .arg(&a)
        .arg("requires")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .failure();
}
