// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk search` command.
//! Converted from tests/specs/cli/unit/search.bats
//!
//! BATS test mapping:
//! - "search requires query and finds by various fields"
//!   -> search_requires_query, search_finds_by_* (parameterized)
//! - "search with no matches shows empty output"
//!   -> search_no_matches_shows_empty_output
//! - "search respects --status, --type, --label filters"
//!   -> search_status_filter, search_type_filter, search_label_filter
//! - "search --output json outputs valid JSON (including short flag)"
//!   -> search_output_json_valid, search_output_json_short_flag
//! - "search help shows examples"
//!   -> search_help_shows_examples
//! - "search limits results to 25 and shows N more"
//!   -> search_limits_results_to_25
//! - "search does not show N more when under limit"
//!   -> search_under_limit_no_more_message
//! - "search --filter with age and validation"
//!   -> search_filter_age, search_filter_short_flag, search_filter_invalid
//! - "search --limit overrides default limit"
//!   -> search_limit_overrides_default
//! - "search --filter and --limit work together with JSON output"
//!   -> search_filter_and_limit_work_together
//! - "search --filter closed works with query and excludes open"
//!   -> search_filter_closed
//! - "search filters by prefix"
//!   -> search_filters_by_prefix
//! - "search auto-filters by configured project prefix"
//!   -> search_auto_filters_by_project_prefix

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
        .stderr(predicate::str::contains("Usage").or(predicate::str::contains("required")));
}

#[test]
fn search_finds_by_title() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchBasic Authentication login");
    create_issue(&temp, "task", "SearchBasic Dashboard widget");

    wk().args(["search", "login"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchBasic Authentication login"))
        .stdout(predicate::str::contains("SearchBasic Dashboard widget").not());
}

#[test]
fn search_finds_by_description() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchDesc Generic task");
    wk().args(["edit", &id, "description", "Implement OAuth2 flow"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["search", "OAuth2"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchDesc Generic task"));
}

#[test]
fn search_finds_by_note_content() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchNote Setup task");
    wk().args(["note", &id, "Configure SSL certificates"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["search", "SSL"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchNote Setup task"));
}

#[test]
fn search_finds_by_label() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchLabel Important task");
    wk().args(["label", &id, "priority:high"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["search", "priority:high"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchLabel Important task"));
}

#[test]
fn search_finds_by_external_link_url() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchLink Linked task");
    wk().args(["link", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["search", "github.com"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchLink Linked task"));
}

#[test]
fn search_finds_by_external_id() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "SearchExtId Jira linked");
    wk().args(["link", &id, "jira://PE-5555"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["search", "PE-5555"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchExtId Jira linked"));
}

#[test]
fn search_is_case_insensitive() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchCase Authentication login");

    wk().args(["search", "authentication"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchCase Authentication login"));
}

// =============================================================================
// Empty Search Results
// =============================================================================

#[test]
fn search_no_matches_shows_empty_output() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchEmpty Some task");

    wk().args(["search", "nonexistent"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchEmpty Some task").not());
}

// =============================================================================
// Filter Tests
// =============================================================================

#[test]
fn search_status_filter() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchFilter Todo task");
    let id2 = create_issue(&temp, "task", "SearchFilter Done task");
    wk().args(["start", &id2])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["done", &id2])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["search", "SearchFilter", "--status", "todo"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchFilter Todo task"))
        .stdout(predicate::str::contains("SearchFilter Done task").not());
}

#[test]
fn search_type_filter() {
    let temp = init_temp();
    create_issue(&temp, "bug", "SearchType Bug with auth");
    create_issue(&temp, "task", "SearchType Task with auth");

    wk().args(["search", "auth", "--type", "bug"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchType Bug with auth"))
        .stdout(predicate::str::contains("SearchType Task with auth").not());
}

#[test]
fn search_label_filter() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "SearchLabelF Task A");
    create_issue(&temp, "task", "SearchLabelF Task B");
    wk().args(["label", &id1, "urgent"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["search", "Task", "--label", "urgent"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchLabelF Task A"))
        .stdout(predicate::str::contains("SearchLabelF Task B").not());
}

// =============================================================================
// JSON Output Tests
// =============================================================================

#[test]
fn search_output_json_valid() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchJSON test task");

    let output = wk()
        .args(["search", "SearchJSON", "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
}

#[test]
fn search_output_json_short_flag() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchJSONShort test task");

    let output = wk()
        .args(["search", "SearchJSONShort", "-o", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
}

// =============================================================================
// Help Tests
// =============================================================================

#[test]
fn search_help_shows_examples() {
    wk().args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples"));
}

// =============================================================================
// Limit Tests
// =============================================================================

#[test]
fn search_limits_results_to_25() {
    let temp = init_temp();

    for i in 1..=30 {
        create_issue(&temp, "task", &format!("SearchLimit test item {}", i));
    }

    let output = wk()
        .args(["search", "SearchLimit test"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let count = stdout.matches("SearchLimit test item").count();
    assert_eq!(count, 25, "Should return exactly 25 results, got {}", count);
    assert!(
        stdout.contains("... 5 more"),
        "Should show '... 5 more' message"
    );

    // JSON output should also be limited to 25 items
    let json_output = wk()
        .args(["search", "SearchLimit test", "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let json_stdout = String::from_utf8_lossy(&json_output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_stdout).unwrap();
    let json_count = json.as_array().unwrap().len();
    assert_eq!(json_count, 25, "JSON output should have 25 items");
}

#[test]
fn search_under_limit_no_more_message() {
    let temp = init_temp();

    for i in 1..=10 {
        create_issue(&temp, "task", &format!("SearchUnderLimit {}", i));
    }

    let output = wk()
        .args(["search", "SearchUnderLimit"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("more"),
        "Should not show 'more' message when under limit"
    );

    // JSON output returns all results
    let json_output = wk()
        .args(["search", "SearchUnderLimit", "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let json_stdout = String::from_utf8_lossy(&json_output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_stdout).unwrap();
    let json_count = json.as_array().unwrap().len();
    assert_eq!(json_count, 10, "JSON output should have all 10 items");
}

// =============================================================================
// Filter Expression Tests
// =============================================================================

#[test]
fn search_filter_age() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchAge Old");
    std::thread::sleep(std::time::Duration::from_millis(200));
    create_issue(&temp, "task", "SearchAge New");

    wk().args(["search", "SearchAge", "--filter", "age < 100ms"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchAge New"))
        .stdout(predicate::str::contains("SearchAge Old").not());
}

#[test]
fn search_filter_short_flag() {
    let temp = init_temp();
    create_issue(&temp, "task", "SearchAgeShort Task");

    wk().args(["search", "SearchAgeShort", "-q", "age < 1h"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchAgeShort"));
}

#[test]
fn search_filter_invalid() {
    let temp = init_temp();

    wk().args(["search", "test", "--filter", "invalid < 3d"])
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
fn search_limit_overrides_default(flag: &str, limit: &str) {
    let temp = init_temp();

    for i in 1..=10 {
        create_issue(&temp, "task", &format!("SearchLimitOverride {}", i));
    }

    let output = wk()
        .args(["search", "SearchLimitOverride", flag, limit])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("SearchLimitOverride").count();
    let expected: usize = limit.parse().unwrap();
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

    // Filter + limit
    let output = wk()
        .args([
            "search",
            "SearchCombo",
            "--filter",
            "age < 1h",
            "--limit",
            "2",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("SearchCombo").count();
    assert_eq!(count, 2, "Should return exactly 2 results");

    // JSON output with filter
    let json_output = wk()
        .args([
            "search",
            "SearchCombo",
            "--filter",
            "age < 1d",
            "-o",
            "json",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let json_stdout = String::from_utf8_lossy(&json_output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_stdout).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 5, "Should have 5 items");

    // JSON output with limit
    let json_limit_output = wk()
        .args(["search", "SearchCombo", "--limit", "3", "-o", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let json_limit_stdout = String::from_utf8_lossy(&json_limit_output.stdout);
    let json_limit: serde_json::Value = serde_json::from_str(&json_limit_stdout).unwrap();
    assert_eq!(
        json_limit.as_array().unwrap().len(),
        3,
        "Should have 3 items"
    );
}

#[test]
fn search_filter_closed() {
    let temp = init_temp();
    let closed_id = create_issue(&temp, "task", "SearchClosed Closed");
    wk().args(["start", &closed_id])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["done", &closed_id])
        .current_dir(temp.path())
        .assert()
        .success();
    create_issue(&temp, "task", "SearchClosed Open");

    wk().args(["search", "SearchClosed", "--filter", "closed < 1d"])
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
fn search_filters_by_prefix() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PrefixSearch Alpha task");
    let _id2 = create_issue_with_prefix(&temp, "task", "PrefixSearch Beta task", "beta");

    // Extract prefix from id1
    let prefix1 = id1.split('-').next().unwrap();

    // Search with specific prefix (-p flag)
    wk().args(["search", "PrefixSearch", "-p", prefix1])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PrefixSearch Alpha task"))
        .stdout(predicate::str::contains("PrefixSearch Beta task").not());

    // Search with --prefix flag
    wk().args(["search", "PrefixSearch", "--prefix", "beta"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PrefixSearch Beta task"))
        .stdout(predicate::str::contains("PrefixSearch Alpha task").not());
}

#[test]
fn search_auto_filters_by_project_prefix() {
    let temp = init_temp();
    create_issue(&temp, "task", "AutoSearch Own task");
    let _id2 = create_issue_with_prefix(&temp, "task", "AutoSearch Other task", "beta");

    // Without -p flag, should only show issues matching configured prefix
    wk().args(["search", "AutoSearch"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AutoSearch Own task"))
        .stdout(predicate::str::contains("AutoSearch Other task").not());
}
