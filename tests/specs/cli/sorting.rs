// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for sorting behavior.
//! Converted from tests/specs/cli/integration/sorting.bats
//!
//! Tests verifying priority-based sorting in `list` and `ready` commands.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use tempfile::TempDir;
use yare::parameterized;

// =============================================================================
// Helpers
// =============================================================================

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

fn add_label(temp: &TempDir, id: &str, label: &str) {
    wk().arg("label")
        .arg(id)
        .arg(label)
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Ready Command Sort Tests
// =============================================================================

#[test]
fn ready_high_priority_before_low_priority() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Low priority recent");
    add_label(&temp, &id1, "priority:3");
    let id2 = create_issue(&temp, "task", "High priority recent");
    add_label(&temp, &id2, "priority:1");

    let output = wk().arg("ready").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let high_pos = stdout.find("High priority recent").unwrap();
    let low_pos = stdout.find("Low priority recent").unwrap();
    assert!(
        high_pos < low_pos,
        "High priority should appear before low priority"
    );
}

#[test]
fn ready_priority_n_preferred_over_p_n() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Dual tagged issue");
    add_label(&temp, &id, "p:0");
    add_label(&temp, &id, "priority:4");
    let _id2 = create_issue(&temp, "task", "Default priority issue");

    let output = wk().arg("ready").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let default_pos = stdout.find("Default priority issue").unwrap();
    let dual_pos = stdout.find("Dual tagged issue").unwrap();
    // priority:4 should be used over p:0, so default (priority:2) appears first
    assert!(
        default_pos < dual_pos,
        "Default priority (2) should appear before dual-tagged (priority:4)"
    );
}

#[test]
fn ready_named_priority_values_work() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Highest priority");
    add_label(&temp, &id1, "priority:highest");
    let id2 = create_issue(&temp, "task", "Lowest priority");
    add_label(&temp, &id2, "priority:lowest");

    let output = wk().arg("ready").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let highest_pos = stdout.find("Highest priority").unwrap();
    let lowest_pos = stdout.find("Lowest priority").unwrap();
    assert!(
        highest_pos < lowest_pos,
        "Highest priority should appear before lowest"
    );
}

#[test]
fn ready_default_priority_is_2_medium() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "High priority task");
    add_label(&temp, &id1, "priority:1");
    let _id2 = create_issue(&temp, "task", "Default priority task");
    let id3 = create_issue(&temp, "task", "Low priority task");
    add_label(&temp, &id3, "priority:3");

    let output = wk().arg("ready").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let high_pos = stdout.find("High priority task").unwrap();
    let default_pos = stdout.find("Default priority task").unwrap();
    let low_pos = stdout.find("Low priority task").unwrap();

    assert!(
        high_pos < default_pos,
        "High (p1) should appear before default (p2)"
    );
    assert!(
        default_pos < low_pos,
        "Default (p2) should appear before low (p3)"
    );
}

// =============================================================================
// List Command Sort Tests
// =============================================================================

#[test]
fn list_sorts_by_priority_asc() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "P3 list task");
    add_label(&temp, &id1, "priority:3");
    let id2 = create_issue(&temp, "task", "P1 list task");
    add_label(&temp, &id2, "priority:1");

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let p1_pos = stdout.find("P1 list task").unwrap();
    let p3_pos = stdout.find("P3 list task").unwrap();
    assert!(p1_pos < p3_pos, "P1 should appear before P3");
}

#[test]
fn list_same_priority_sorts_by_created_at_desc() {
    let temp = init_temp();
    create_issue(&temp, "task", "Older list task");
    std::thread::sleep(std::time::Duration::from_millis(100));
    create_issue(&temp, "task", "Newer list task");

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let newer_pos = stdout.find("Newer list task").unwrap();
    let older_pos = stdout.find("Older list task").unwrap();
    assert!(
        newer_pos < older_pos,
        "Newer should appear before older when same priority"
    );
}

#[test]
fn list_priority_n_preferred_over_p_n() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Dual tagged list");
    add_label(&temp, &id, "p:0");
    add_label(&temp, &id, "priority:4");
    let _id2 = create_issue(&temp, "task", "Default list issue");

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let default_pos = stdout.find("Default list issue").unwrap();
    let dual_pos = stdout.find("Dual tagged list").unwrap();
    // priority:4 should be used over p:0, so default (priority:2) appears first
    assert!(
        default_pos < dual_pos,
        "Default priority should appear before dual-tagged (priority:4)"
    );
}

// =============================================================================
// JSON Format Tests - Parameterized for list and ready
// =============================================================================

#[parameterized(
    ready = { "ready" },
    list = { "list" },
)]
fn json_format_respects_priority_sorting(command: &str) {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Low priority json");
    add_label(&temp, &id1, "priority:3");
    let id2 = create_issue(&temp, "task", "High priority json");
    add_label(&temp, &id2, "priority:1");

    let output = wk()
        .arg(command)
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    let issues = json.as_array().expect("Output should be an array");

    // Find positions in array
    let high_idx = issues
        .iter()
        .position(|i| i["id"].as_str() == Some(&id2))
        .expect("High priority issue should be in output");
    let low_idx = issues
        .iter()
        .position(|i| i["id"].as_str() == Some(&id1))
        .expect("Low priority issue should be in output");

    assert!(
        high_idx < low_idx,
        "High priority ({}) should appear before low priority ({}) in JSON",
        high_idx,
        low_idx
    );
}
