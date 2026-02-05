// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk search` command.
//! Converted from tests/specs/cli/unit/search.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

// =============================================================================
// Helpers
// =============================================================================

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    create_issue_with_opts(temp, type_, title, &[])
}

fn create_issue_with_opts(temp: &TempDir, type_: &str, title: &str, opts: &[&str]) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title);
    for opt in opts {
        cmd.arg(opt);
    }
    cmd.arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_issue_with_prefix(temp: &TempDir, type_: &str, title: &str, prefix: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new")
        .arg(type_)
        .arg(title)
        .arg("--prefix")
        .arg(prefix)
        .arg("-o")
        .arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic Search Tests
// =============================================================================

#[test]
fn search_requires_query() {
    let temp = init_temp();

    wk().arg("search")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn search_finds_by_title() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchBasic Authentication login");
    create_issue(&temp, "task", "SearchBasic Dashboard widget");

    wk().arg("search")
        .arg("login")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Authentication login"))
        .stdout(predicate::str::contains("SearchBasic Dashboard widget").not());
}

#[test]
fn search_finds_by_description() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchBasic Generic task");
    wk().arg("edit")
        .arg(&id)
        .arg("description")
        .arg("Implement OAuth2 flow")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("search")
        .arg("OAuth2")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Generic task"));
}

#[test]
fn search_finds_by_note_content() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchBasic Setup task");
    wk().arg("note")
        .arg(&id)
        .arg("Configure SSL certificates")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("search")
        .arg("SSL")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Setup task"));
}

#[test]
fn search_finds_by_label() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchBasic Important task");
    wk().arg("label")
        .arg(&id)
        .arg("priority:high")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("search")
        .arg("priority:high")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Important task"));
}

#[test]
fn search_finds_by_external_link_url() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchBasic Linked task");
    wk().arg("link")
        .arg(&id)
        .arg("https://github.com/org/repo/issues/123")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("search")
        .arg("github.com")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Linked task"));
}

#[test]
fn search_finds_by_external_id() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchBasic Jira linked");
    wk().arg("link")
        .arg(&id)
        .arg("jira://PE-5555")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("search")
        .arg("PE-5555")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Jira linked"));
}

#[test]
fn search_is_case_insensitive() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchBasic Authentication login");

    wk().arg("search")
        .arg("authentication")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Authentication login"));
}

// =============================================================================
// Empty Search Results
// =============================================================================

#[test]
fn search_with_no_matches_shows_empty_output() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchEmpty Some task");

    wk().arg("search")
        .arg("nonexistent")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchEmpty Some task").not());
}

// =============================================================================
// Filter Tests
// =============================================================================

#[test]
fn search_respects_status_filter() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "SearchFilter Todo task");
    let id2 = create_issue(&temp, "task", "SearchFilter Done task");
    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("search")
        .arg("SearchFilter")
        .arg("--status")
        .arg("todo")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchFilter Todo task"))
        .stdout(predicate::str::contains("SearchFilter Done task").not());
}

#[test]
fn search_respects_type_filter() {
    let temp = init_temp();
    create_issue(&temp, "bug", "SearchFilter Bug with auth");
    create_issue(&temp, "task", "SearchFilter Task with auth");

    wk().arg("search")
        .arg("auth")
        .arg("--type")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchFilter Bug with auth"))
        .stdout(predicate::str::contains("SearchFilter Task with auth").not());
}

#[test]
fn search_respects_label_filter() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "SearchFilter Task A");
    let _id2 = create_issue(&temp, "task", "SearchFilter Task B");
    wk().arg("label")
        .arg(&id1)
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("search")
        .arg("Task")
        .arg("--label")
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchFilter Task A"))
        .stdout(predicate::str::contains("SearchFilter Task B").not());
}

// =============================================================================
// JSON Output Tests
// =============================================================================

#[test]
fn search_output_json_is_valid() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchJSON test task");

    let output = wk()
        .arg("search")
        .arg("SearchJSON")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
}

#[test]
fn search_output_json_short_flag() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchJSON Short flag test");

    let output = wk()
        .arg("search")
        .arg("SearchJSON Short")
        .arg("-o")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
}

// =============================================================================
// Help Tests
// =============================================================================

#[test]
fn search_help_shows_examples() {
    wk().arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples"));
}

// =============================================================================
// Limit Tests
// =============================================================================

#[test]
fn search_limits_results_to_25_and_shows_n_more() {
    let temp = init_temp();

    for i in 1..=30 {
        create_issue(&temp, "task", &format!("SearchLimit test item {}", i));
    }

    let output = wk()
        .arg("search")
        .arg("SearchLimit test")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("SearchLimit test item").count();
    assert_eq!(count, 25, "Should show exactly 25 results");
    assert!(
        stdout.contains("... 5 more"),
        "Should show '... 5 more' footer"
    );
}

#[test]
fn search_limits_json_output_to_25() {
    let temp = init_temp();

    for i in 1..=30 {
        create_issue(&temp, "task", &format!("SearchLimitJSON test {}", i));
    }

    let output = wk()
        .arg("search")
        .arg("SearchLimitJSON test")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let count = json.as_array().unwrap().len();
    assert_eq!(count, 25, "JSON output should be limited to 25 items");
}

#[test]
fn search_does_not_show_n_more_when_under_limit() {
    let temp = init_temp();

    for i in 1..=10 {
        create_issue(&temp, "task", &format!("SearchUnderLimit {}", i));
    }

    wk().arg("search")
        .arg("SearchUnderLimit")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("more").not());
}

#[test]
fn search_json_output_under_limit_shows_all() {
    let temp = init_temp();

    for i in 1..=10 {
        create_issue(&temp, "task", &format!("SearchUnderLimitJSON {}", i));
    }

    let output = wk()
        .arg("search")
        .arg("SearchUnderLimitJSON")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let count = json.as_array().unwrap().len();
    assert_eq!(count, 10, "JSON output should show all 10 items");
}

// =============================================================================
// Filter Expression Tests
// =============================================================================

#[test]
fn search_filter_age() {
    let temp = init_temp();
    let _old_id = create_issue(&temp, "task", "SearchAge Old");
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _new_id = create_issue(&temp, "task", "SearchAge New");

    wk().arg("search")
        .arg("SearchAge")
        .arg("--filter")
        .arg("age < 100ms")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchAge New"))
        .stdout(predicate::str::contains("SearchAge Old").not());
}

#[test]
fn search_filter_short_flag() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchFilterShort Task");

    wk().arg("search")
        .arg("SearchFilterShort")
        .arg("-q")
        .arg("age < 1h")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchFilterShort"));
}

#[test]
fn search_filter_invalid_field_shows_error() {
    let temp = init_temp();

    wk().arg("search")
        .arg("test")
        .arg("--filter")
        .arg("invalid < 3d")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown filter field"));
}

// =============================================================================
// Limit Override Tests
// =============================================================================

#[parameterized(
    long_flag = { "--limit", "3" },
    short_flag = { "-n", "2" },
)]
fn search_limit_overrides_default(flag: &str, value: &str) {
    let temp = init_temp();

    for i in 1..=10 {
        create_issue(&temp, "task", &format!("SearchLimitOverride {}", i));
    }

    let output = wk()
        .arg("search")
        .arg("SearchLimitOverride")
        .arg(flag)
        .arg(value)
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("SearchLimitOverride").count();
    let expected: usize = value.parse().unwrap();
    assert_eq!(
        count, expected,
        "Should return exactly {} results",
        expected
    );
}

// =============================================================================
// Combined Filter and Limit Tests
// =============================================================================

#[test]
fn search_filter_and_limit_work_together() {
    let temp = init_temp();

    for i in 1..=5 {
        create_issue(&temp, "task", &format!("SearchCombo {}", i));
    }

    let output = wk()
        .arg("search")
        .arg("SearchCombo")
        .arg("--filter")
        .arg("age < 1h")
        .arg("--limit")
        .arg("2")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("SearchCombo").count();
    assert_eq!(count, 2, "Should return exactly 2 results");
}

#[test]
fn search_json_output_with_filter() {
    let temp = init_temp();

    for i in 1..=5 {
        create_issue(&temp, "task", &format!("SearchComboJSON {}", i));
    }

    let output = wk()
        .arg("search")
        .arg("SearchComboJSON")
        .arg("--filter")
        .arg("age < 1d")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let count = json.as_array().unwrap().len();
    assert_eq!(count, 5, "JSON output should show all 5 matching items");
}

#[test]
fn search_json_output_respects_limit() {
    let temp = init_temp();

    for i in 1..=5 {
        create_issue(&temp, "task", &format!("SearchComboLimitJSON {}", i));
    }

    let output = wk()
        .arg("search")
        .arg("SearchComboLimitJSON")
        .arg("--limit")
        .arg("3")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let count = json.as_array().unwrap().len();
    assert_eq!(count, 3, "JSON output should respect --limit");
}

// =============================================================================
// Closed Filter Tests
// =============================================================================

#[test]
fn search_filter_closed_works_with_query_and_excludes_open() {
    let temp = init_temp();
    let closed_id = create_issue(&temp, "task", "SearchClosed Closed");
    wk().arg("start")
        .arg(&closed_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&closed_id)
        .current_dir(temp.path())
        .assert()
        .success();
    create_issue(&temp, "task", "SearchClosed Open");

    wk().arg("search")
        .arg("SearchClosed")
        .arg("--filter")
        .arg("closed < 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchClosed Closed"))
        .stdout(predicate::str::contains("SearchClosed Open").not());
}

// =============================================================================
// Prefix Filter Tests
// =============================================================================

#[test]
fn search_filters_by_prefix_flag() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PrefixSearch Alpha task");
    let _id2 = create_issue_with_prefix(&temp, "task", "PrefixSearch Beta task", "beta");

    // Extract prefix from id1
    let prefix1 = id1.split('-').next().unwrap();

    wk().arg("search")
        .arg("PrefixSearch")
        .arg("-p")
        .arg(prefix1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PrefixSearch Alpha task"))
        .stdout(predicate::str::contains("PrefixSearch Beta task").not());
}

#[test]
fn search_filters_by_prefix_long_flag() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "PrefixSearchLong Alpha task");
    let _id2 = create_issue_with_prefix(&temp, "task", "PrefixSearchLong Beta task", "beta");

    wk().arg("search")
        .arg("PrefixSearchLong")
        .arg("--prefix")
        .arg("beta")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PrefixSearchLong Beta task"))
        .stdout(predicate::str::contains("PrefixSearchLong Alpha task").not());
}

#[test]
fn search_auto_filters_by_configured_project_prefix() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "AutoSearch Own task");
    let _id2 = create_issue_with_prefix(&temp, "task", "AutoSearch Other task", "beta");

    // Without -p flag, should only show issues matching configured prefix (test)
    wk().arg("search")
        .arg("AutoSearch")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AutoSearch Own task"))
        .stdout(predicate::str::contains("AutoSearch Other task").not());
}
