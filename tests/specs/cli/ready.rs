// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk ready` command.
//! Converted from tests/specs/cli/unit/ready.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let output = wk()
        .args(["new", type_, title, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_issue_with_prefix(temp: &TempDir, type_: &str, title: &str, prefix: &str) -> String {
    let output = wk()
        .args(["new", type_, title, "--prefix", prefix, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic Ready Tests
// =============================================================================

#[test]
fn ready_shows_unblocked_todo_issues_and_excludes_blocked() {
    let temp = init_temp();
    create_issue(&temp, "task", "Ready task");
    let a = create_issue(&temp, "task", "Blocker");
    let b = create_issue(&temp, "task", "Blocked issue");

    wk().args(["dep", &a, "blocks", &b])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("ready")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Ready task"))
        .stdout(predicate::str::contains("Blocker"))
        .stdout(predicate::str::contains("Blocked issue").not());
}

#[test]
fn ready_with_label_filter() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Labeled task");
    wk().args(["label", &id, "priority:high"])
        .current_dir(temp.path())
        .assert()
        .success();
    create_issue(&temp, "task", "Unlabeled task");

    // With matching label
    wk().args(["ready", "--label", "priority:high"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Labeled task"))
        .stdout(predicate::str::contains("Unlabeled task").not());

    // With non-matching label
    wk().args(["ready", "--label", "nonexistent-label-xyz123"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No ready issues"));
}

// =============================================================================
// JSON Output Tests
// =============================================================================

#[test]
fn ready_output_json_valid_with_expected_fields() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "JSON ready task");
    wk().args(["label", &id, "module:api"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Test --output json
    let output = wk()
        .args(["ready", "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    let issues = json.as_array().expect("Output should be an array");
    assert!(!issues.is_empty());

    let issue = &issues[0];
    assert!(issue.get("id").is_some());
    assert!(issue.get("issue_type").is_some());
    assert!(issue.get("status").is_some());
    assert!(issue.get("title").is_some());
    assert!(issue.get("labels").is_some());

    // Verify labels included
    let target_issue = issues
        .iter()
        .find(|i| i["title"] == "JSON ready task")
        .expect("Should find the issue");
    let labels = target_issue["labels"].as_array().unwrap();
    assert!(labels.iter().any(|l| l.as_str() == Some("module:api")));

    // Test -o json short flag
    let output = wk()
        .args(["ready", "-o", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: serde_json::Value =
        serde_json::from_str(&stdout).expect("Short flag output should be valid JSON");
}

#[test]
fn ready_output_json_excludes_blocked_and_respects_filters() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Blocker JSON ready");
    wk().args(["label", &a, "test:json-blocked"])
        .current_dir(temp.path())
        .assert()
        .success();

    let b = create_issue(&temp, "task", "Blocked JSON ready");
    wk().args(["label", &b, "test:json-blocked"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["dep", &a, "blocks", &b])
        .current_dir(temp.path())
        .assert()
        .success();

    let id = create_issue(&temp, "task", "Labeled ready");
    wk().args(["label", &id, "team:backend"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Blocked issues excluded (filter by test label to avoid 5-issue limit interference)
    let output = wk()
        .args(["ready", "--label", "test:json-blocked", "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let issues = json.as_array().unwrap();
    let ids: Vec<&str> = issues.iter().filter_map(|i| i["id"].as_str()).collect();
    assert!(
        !ids.contains(&b.as_str()),
        "Blocked issue should be excluded"
    );
    assert!(ids.contains(&a.as_str()), "Blocker should be included");

    // Label filter works
    let output = wk()
        .args(["ready", "--label", "team:backend", "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let issues = json.as_array().unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["title"].as_str(), Some("Labeled ready"));

    // Non-matching label returns empty
    let output = wk()
        .args([
            "ready",
            "--label",
            "nonexistent-label-abc456",
            "--output",
            "json",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let issues = json.as_array().unwrap();
    assert_eq!(issues.len(), 0);
}

// =============================================================================
// Sort Order Tests
// =============================================================================

#[test]
fn ready_sorts_recent_high_priority_before_recent_low_priority() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "ReadySort Low priority recent");
    wk().args(["label", &id1, "priority:3"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id1, "test:sort-recent"])
        .current_dir(temp.path())
        .assert()
        .success();

    let id2 = create_issue(&temp, "task", "ReadySort High priority recent");
    wk().args(["label", &id2, "priority:1"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id2, "test:sort-recent"])
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .args(["ready", "--label", "test:sort-recent"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let high_pos = stdout.find("ReadySort High priority recent").unwrap();
    let low_pos = stdout.find("ReadySort Low priority recent").unwrap();
    assert!(
        high_pos < low_pos,
        "High priority should appear before low priority"
    );
}

#[test]
fn ready_uses_priority_n_tag_for_priority() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "P0 task");
    wk().args(["label", &id, "priority:0"])
        .current_dir(temp.path())
        .assert()
        .success();
    create_issue(&temp, "task", "Default task");

    let output = wk().arg("ready").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // P0 should appear before default (priority 2)
    // Find first issue line (starts with "- [")
    let first_issue = stdout.lines().find(|line| line.starts_with("- [")).unwrap();
    assert!(first_issue.contains("P0 task"), "P0 should be first");
}

#[test]
fn ready_prefers_priority_over_p_tag() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ReadyPref Dual tagged");
    wk().args(["label", &id, "p:0"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id, "priority:4"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id, "test:pref-priority"])
        .current_dir(temp.path())
        .assert()
        .success();

    let id2 = create_issue(&temp, "task", "ReadyPref Default priority issue");
    wk().args(["label", &id2, "test:pref-priority"])
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .args(["ready", "--label", "test:pref-priority"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dual_pos = stdout.find("ReadyPref Dual tagged").unwrap();
    let default_pos = stdout.find("ReadyPref Default priority issue").unwrap();
    // Dual tagged (priority:4) should appear after default (priority:2)
    assert!(
        default_pos < dual_pos,
        "Default should appear before dual-tagged"
    );
}

#[test]
fn ready_treats_missing_priority_as_2_medium() {
    let temp = init_temp();
    // Create high priority issue
    let id1 = create_issue(&temp, "task", "ReadyMiss High priority task");
    wk().args(["label", &id1, "priority:1"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id1, "test:miss-priority"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Create default priority issue (no tag = 2)
    let id2 = create_issue(&temp, "task", "ReadyMiss Default priority task");
    wk().args(["label", &id2, "test:miss-priority"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Create low priority issue
    let id3 = create_issue(&temp, "task", "ReadyMiss Low priority task");
    wk().args(["label", &id3, "priority:3"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id3, "test:miss-priority"])
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .args(["ready", "--label", "test:miss-priority"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let high_pos = stdout.find("ReadyMiss High priority task").unwrap();
    let default_pos = stdout.find("ReadyMiss Default priority task").unwrap();
    let low_pos = stdout.find("ReadyMiss Low priority task").unwrap();

    // Order should be: high (1), default (2), low (3)
    assert!(high_pos < default_pos, "High should be before default");
    assert!(default_pos < low_pos, "Default should be before low");
}

#[parameterized(
    highest = { "highest", "lowest", true },
    lowest = { "lowest", "highest", false },
)]
fn ready_named_priority_values_work(high_name: &str, low_name: &str, high_first: bool) {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "ReadyNamed Highest priority");
    wk().args(["label", &id1, &format!("priority:{}", high_name)])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id1, "test:named-priority"])
        .current_dir(temp.path())
        .assert()
        .success();

    let id2 = create_issue(&temp, "task", "ReadyNamed Lowest priority");
    wk().args(["label", &id2, &format!("priority:{}", low_name)])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["label", &id2, "test:named-priority"])
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .args(["ready", "--label", "test:named-priority"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let highest_pos = stdout.find("ReadyNamed Highest priority").unwrap();
    let lowest_pos = stdout.find("ReadyNamed Lowest priority").unwrap();

    if high_first {
        assert!(
            highest_pos < lowest_pos,
            "Highest should appear before lowest"
        );
    }
}

// =============================================================================
// Limit and Hint Tests
// =============================================================================

#[test]
fn ready_returns_at_most_5_issues() {
    let temp = init_temp();

    // Create 8 ready issues
    for i in 1..=8 {
        create_issue(&temp, "task", &format!("ReadyLimit Issue {}", i));
    }

    let output = wk().arg("ready").current_dir(temp.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Count issues shown (lines starting with "- [")
    let count = stdout.lines().filter(|l| l.starts_with("- [")).count();
    assert!(count <= 5, "Should return at most 5, got {}", count);

    // JSON also respects limit
    let output = wk()
        .args(["ready", "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let issues = json.as_array().unwrap();
    assert!(issues.len() <= 5, "JSON should return at most 5 issues");
}

#[test]
fn ready_shows_hint_when_more_issues_exist() {
    let temp = init_temp();

    // Create 8 ready issues (more than the 5 limit)
    for i in 1..=8 {
        let id = create_issue(&temp, "task", &format!("ReadyHint Issue {}", i));
        wk().args(["label", &id, "test:hint-more"])
            .current_dir(temp.path())
            .assert()
            .success();
    }

    wk().args(["ready", "--label", "test:hint-more"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 more"))
        .stdout(predicate::str::contains("wk list"));
}

#[test]
fn ready_does_not_show_hint_when_all_issues_fit() {
    let temp = init_temp();

    // Create 3 ready issues (fewer than 5 limit)
    for i in 1..=3 {
        let id = create_issue(&temp, "task", &format!("ReadyNoHint Issue {}", i));
        wk().args(["label", &id, "test:hint-none"])
            .current_dir(temp.path())
            .assert()
            .success();
    }

    wk().args(["ready", "--label", "test:hint-none"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("more").not());
}

// =============================================================================
// Prefix Filter Tests
// =============================================================================

#[test]
fn ready_filters_by_prefix() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PrefixReady Alpha task");
    let _id2 = create_issue_with_prefix(&temp, "task", "PrefixReady Beta task", "beta");

    let prefix1 = id1.split('-').next().unwrap();

    wk().args(["ready", "-p", prefix1])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PrefixReady Alpha task"))
        .stdout(predicate::str::contains("PrefixReady Beta task").not());

    wk().args(["ready", "--prefix", "beta"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PrefixReady Beta task"))
        .stdout(predicate::str::contains("PrefixReady Alpha task").not());
}

#[test]
fn ready_auto_filters_by_configured_project_prefix() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "AutoReady Own task");
    let _id2 = create_issue_with_prefix(&temp, "task", "AutoReady Other task", "beta");

    // Without -p flag, should only show issues matching configured prefix
    wk().args(["ready", "--all-assignees"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AutoReady Own task"))
        .stdout(predicate::str::contains("AutoReady Other task").not());
}
