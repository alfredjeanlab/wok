// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for ID collision handling.
//! Converted from tests/specs/cli/edge_cases/collisions.bats
//!
//! Tests verifying that issues with identical titles get unique IDs.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::super::common::*;
use std::collections::HashSet;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");
    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn identical_titles_get_unique_ids() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Duplicate title");
    let id2 = create_issue(&temp, "task", "Duplicate title");

    assert!(!id1.is_empty(), "id1 should not be empty");
    assert!(!id2.is_empty(), "id2 should not be empty");
    assert_ne!(id1, id2, "IDs should be different");
}

#[test]
fn collision_suffix_increments() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Same title");
    let id2 = create_issue(&temp, "task", "Same title");
    let id3 = create_issue(&temp, "task", "Same title");

    // All should be unique
    assert_ne!(id1, id2, "id1 and id2 should differ");
    assert_ne!(id2, id3, "id2 and id3 should differ");
    assert_ne!(id1, id3, "id1 and id3 should differ");
}

#[test]
fn all_collided_ids_are_valid() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Collision test");
    let id2 = create_issue(&temp, "task", "Collision test");

    wk().arg("show").arg(&id1).current_dir(temp.path()).assert().success();

    wk().arg("show").arg(&id2).current_dir(temp.path()).assert().success();
}

#[test]
fn collided_ids_can_be_used_independently() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Same name");
    let id2 = create_issue(&temp, "task", "Same name");

    wk().arg("start").arg(&id1).current_dir(temp.path()).assert().success();

    // id1 should be in_progress, id2 should still be todo
    wk().arg("show")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: in_progress"));

    wk().arg("show")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: todo"));
}

#[test]
fn many_collisions_handled() {
    let temp = init_temp();
    let mut ids = Vec::new();

    for _ in 0..10 {
        let id = create_issue(&temp, "task", "Repeated title");
        ids.push(id);
    }

    // All should be unique
    let unique: HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), 10, "All 10 IDs should be unique");
}

#[parameterized(
    task = { "task" },
    bug = { "bug" },
    feature = { "feature" },
)]
fn different_types_with_same_title_get_unique_ids(type_: &str) {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Multi-type");
    let id2 = create_issue(&temp, "bug", "Multi-type");
    let id3 = create_issue(&temp, "feature", "Multi-type");

    // Suppress warning for parameterized type
    let _ = type_;

    assert_ne!(id1, id2, "task and bug IDs should differ");
    assert_ne!(id2, id3, "bug and feature IDs should differ");
    assert_ne!(id1, id3, "task and feature IDs should differ");
}
