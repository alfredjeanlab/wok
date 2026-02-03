// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk list` command.
//! Converted from tests/specs/cli/unit/list.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

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

// =============================================================================
// Phase 1: Status Filtering Tests
// =============================================================================

#[test]
fn list_empty_database() {
    let temp = init_temp();
    wk().arg("list").current_dir(temp.path()).assert().success();
}

#[test]
fn list_shows_created_issues() {
    let temp = init_temp();
    create_issue(&temp, "task", "Task 1");
    create_issue(&temp, "task", "Task 2");

    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1"))
        .stdout(predicate::str::contains("Task 2"));
}

#[test]
fn list_default_shows_todo_and_in_progress() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "ListDefault Todo task");
    let id2 = create_issue(&temp, "task", "ListDefault Active task");
    let id3 = create_issue(&temp, "task", "ListDefault Done task");

    // Start id2
    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    // Start and complete id3
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

    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ListDefault Todo task"))
        .stdout(predicate::str::contains("ListDefault Active task"))
        .stdout(predicate::str::contains("ListDefault Done task").not());
}

#[test]
fn list_status_filter_todo() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "StatusFilter Todo");
    let id2 = create_issue(&temp, "task", "StatusFilter InProgress");
    let id3 = create_issue(&temp, "task", "StatusFilter Done");

    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();
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

    wk().arg("list")
        .arg("--status")
        .arg("todo")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("StatusFilter Todo"))
        .stdout(predicate::str::contains("StatusFilter InProgress").not())
        .stdout(predicate::str::contains("StatusFilter Done").not());
}

#[test]
fn list_status_filter_in_progress() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "StatusFilter2 Todo");
    let id2 = create_issue(&temp, "task", "StatusFilter2 InProgress");

    wk().arg("start")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--status")
        .arg("in_progress")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("StatusFilter2 InProgress"))
        .stdout(predicate::str::contains("StatusFilter2 Todo").not());
}

#[test]
fn list_status_filter_done() {
    let temp = init_temp();
    let _id1 = create_issue(&temp, "task", "StatusFilter3 Todo");
    let id2 = create_issue(&temp, "task", "StatusFilter3 Done");

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

    wk().arg("list")
        .arg("--status")
        .arg("done")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("StatusFilter3 Done"))
        .stdout(predicate::str::contains("StatusFilter3 Todo").not());
}

// =============================================================================
// Phase 2: Type, Label, and Blocked Filter Tests
// =============================================================================

#[test]
fn list_type_filter_feature() {
    let temp = init_temp();
    create_issue(&temp, "feature", "TypeFilter MyFeature");
    create_issue(&temp, "task", "TypeFilter MyTask");

    wk().arg("list")
        .arg("--type")
        .arg("feature")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeFilter MyFeature"))
        .stdout(predicate::str::contains("TypeFilter MyTask").not());
}

#[test]
fn list_type_filter_bug() {
    let temp = init_temp();
    create_issue(&temp, "bug", "TypeFilter2 MyBug");
    create_issue(&temp, "task", "TypeFilter2 MyTask");

    wk().arg("list")
        .arg("--type")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeFilter2 MyBug"));
}

#[test]
fn list_type_filter_chore() {
    let temp = init_temp();
    create_issue(&temp, "chore", "TypeFilter3 MyChore");
    create_issue(&temp, "task", "TypeFilter3 MyTask");

    wk().arg("list")
        .arg("--type")
        .arg("chore")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeFilter3 MyChore"));
}

#[test]
fn list_type_filter_idea() {
    let temp = init_temp();
    create_issue(&temp, "idea", "TypeFilter4 MyIdea");
    create_issue(&temp, "task", "TypeFilter4 MyTask");

    wk().arg("list")
        .arg("--type")
        .arg("idea")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeFilter4 MyIdea"))
        .stdout(predicate::str::contains("TypeFilter4 MyTask").not());
}

#[test]
fn list_type_short_flag() {
    let temp = init_temp();
    create_issue(&temp, "task", "TypeFilter5 MyTask");
    create_issue(&temp, "feature", "TypeFilter5 MyFeature");

    wk().arg("list")
        .arg("-t")
        .arg("task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeFilter5 MyTask"));
}

#[test]
fn list_label_filter() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "task",
        "LabelFilter Labeled",
        &["--label", "project:auth"],
    );
    create_issue(&temp, "task", "LabelFilter Other");

    wk().arg("list")
        .arg("--label")
        .arg("project:auth")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("LabelFilter Labeled"))
        .stdout(predicate::str::contains("LabelFilter Other").not());
}

#[test]
fn list_blocked_filter() {
    let temp = init_temp();
    let blocker = create_issue(&temp, "task", "BlockFilter Blocker");
    let blocked = create_issue(&temp, "task", "BlockFilter Blocked");

    wk().arg("dep")
        .arg(&blocker)
        .arg("blocks")
        .arg(&blocked)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--blocked")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BlockFilter Blocker").not())
        .stdout(predicate::str::contains("BlockFilter Blocked"));
}

#[test]
fn list_default_shows_blocked() {
    let temp = init_temp();
    let blocker = create_issue(&temp, "task", "BlockFilter2 Blocker");
    let blocked = create_issue(&temp, "task", "BlockFilter2 Blocked");

    wk().arg("dep")
        .arg(&blocker)
        .arg("blocks")
        .arg(&blocked)
        .current_dir(temp.path())
        .assert()
        .success();

    // Default shows both blocked and unblocked
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BlockFilter2 Blocker"))
        .stdout(predicate::str::contains("BlockFilter2 Blocked"));
}

#[test]
fn list_no_blocked_count_footer() {
    let temp = init_temp();
    let blocker = create_issue(&temp, "task", "NoBlockCount Blocker");
    let blocked = create_issue(&temp, "task", "NoBlockCount Blocked");

    wk().arg("dep")
        .arg(&blocker)
        .arg("blocks")
        .arg(&blocked)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("blocked issues").not());
}

#[test]
fn list_combined_filters() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "feature",
        "Combined Feature",
        &["--label", "team:alpha"],
    );
    create_issue_with_opts(&temp, "task", "Combined Task", &["--label", "team:alpha"]);

    wk().arg("list")
        .arg("--type")
        .arg("feature")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Combined Feature"))
        .stdout(predicate::str::contains("Combined Task").not());
}

// =============================================================================
// Phase 3: JSON Output Tests
// =============================================================================

#[test]
fn list_output_json_valid() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "JSONList Task");
    wk().arg("label")
        .arg(&id)
        .arg("priority:high")
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .arg("list")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(json.as_array().is_some(), "Output should be an array");
}

#[test]
fn list_output_json_fields() {
    let temp = init_temp();
    create_issue(&temp, "task", "JSONFields Task");

    let output = wk()
        .arg("list")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let issues = json.as_array().unwrap();
    let issue = issues
        .iter()
        .find(|i| i["title"] == "JSONFields Task")
        .unwrap();

    assert!(issue.get("id").is_some());
    assert!(issue.get("issue_type").is_some());
    assert!(issue.get("status").is_some());
    assert!(issue.get("title").is_some());
    assert!(issue.get("labels").is_some());
}

#[test]
fn list_output_json_labels() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "JSONLabels Task");
    wk().arg("label")
        .arg(&id)
        .arg("priority:high")
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .arg("list")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let issues = json.as_array().unwrap();
    let issue = issues
        .iter()
        .find(|i| i["title"] == "JSONLabels Task")
        .unwrap();
    let labels = issue["labels"].as_array().unwrap();
    assert!(labels.iter().any(|l| l.as_str() == Some("priority:high")));
}

#[test]
fn list_output_json_short_flag() {
    let temp = init_temp();
    create_issue(&temp, "task", "JSONShort Task");

    let output = wk()
        .arg("list")
        .arg("-o")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
}

#[test]
fn list_output_json_type_filter() {
    let temp = init_temp();
    create_issue(&temp, "task", "JSONFilter Task");
    create_issue(&temp, "bug", "JSONFilter Bug");

    let output = wk()
        .arg("list")
        .arg("--type")
        .arg("bug")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let issues = json.as_array().unwrap();
    for issue in issues {
        assert_eq!(issue["issue_type"].as_str(), Some("bug"));
    }
}

#[test]
fn list_output_json_no_blocked_count() {
    let temp = init_temp();
    let blocker = create_issue(&temp, "task", "JSONBlock Blocker");
    let blocked = create_issue(&temp, "task", "JSONBlock Blocked");

    wk().arg("dep")
        .arg(&blocker)
        .arg("blocks")
        .arg(&blocked)
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .arg("list")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Output is a plain array - no wrapper object with metadata keys
    assert!(json.as_array().is_some(), "Output should be a plain array");
}

// =============================================================================
// Phase 4: Priority Sorting Tests
// =============================================================================

#[test]
fn list_sorts_by_priority_asc() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "SortList P3 task");
    wk().arg("label")
        .arg(&id1)
        .arg("priority:3")
        .current_dir(temp.path())
        .assert()
        .success();

    let id2 = create_issue(&temp, "task", "SortList P1 task");
    wk().arg("label")
        .arg(&id2)
        .arg("priority:1")
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let p1_pos = stdout.find("SortList P1 task").unwrap();
    let p3_pos = stdout.find("SortList P3 task").unwrap();
    assert!(p1_pos < p3_pos, "P1 should appear before P3");
}

#[test]
fn list_same_priority_newer_first() {
    let temp = init_temp();
    create_issue(&temp, "task", "SortList2 Older");
    std::thread::sleep(std::time::Duration::from_millis(100));
    create_issue(&temp, "task", "SortList2 Newer");

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let newer_pos = stdout.find("SortList2 Newer").unwrap();
    let older_pos = stdout.find("SortList2 Older").unwrap();
    assert!(newer_pos < older_pos, "Newer should appear before older");
}

#[test]
fn list_missing_priority_as_2() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "PrioList High");
    wk().arg("label")
        .arg(&id1)
        .arg("priority:1")
        .current_dir(temp.path())
        .assert()
        .success();

    create_issue(&temp, "task", "PrioList Default");

    let id3 = create_issue(&temp, "task", "PrioList Low");
    wk().arg("label")
        .arg(&id3)
        .arg("priority:3")
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let high_pos = stdout.find("PrioList High").unwrap();
    let default_pos = stdout.find("PrioList Default").unwrap();
    let low_pos = stdout.find("PrioList Low").unwrap();

    assert!(high_pos < default_pos, "High (p1) before default (p2)");
    assert!(default_pos < low_pos, "Default (p2) before low (p3)");
}

#[test]
fn list_prefers_priority_over_p() {
    let temp = init_temp();
    let id4 = create_issue(&temp, "task", "PrefList Dual");
    wk().arg("label")
        .arg(&id4)
        .arg("p:0")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("label")
        .arg(&id4)
        .arg("priority:4")
        .current_dir(temp.path())
        .assert()
        .success();

    create_issue(&temp, "task", "PrefList Default2");

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dual_pos = stdout.find("PrefList Dual").unwrap();
    let default2_pos = stdout.find("PrefList Default2").unwrap();

    // priority:4 should be used over p:0, so Default2 (p2) appears first
    assert!(
        default2_pos < dual_pos,
        "Default2 should appear before Dual"
    );
}

// =============================================================================
// Phase 5: Filter Expressions and Duration Parsing
// =============================================================================

#[test]
fn list_filter_age_less_than() {
    let temp = init_temp();
    create_issue(&temp, "task", "AgeFilter Old");
    std::thread::sleep(std::time::Duration::from_millis(500));
    create_issue(&temp, "task", "AgeFilter New");

    wk().arg("list")
        .arg("--filter")
        .arg("age < 400ms")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AgeFilter New"))
        .stdout(predicate::str::contains("AgeFilter Old").not());
}

#[test]
fn list_filter_age_gte() {
    let temp = init_temp();
    create_issue(&temp, "task", "AgeFilter2 Old");
    std::thread::sleep(std::time::Duration::from_millis(500));
    create_issue(&temp, "task", "AgeFilter2 New");

    wk().arg("list")
        .arg("--filter")
        .arg("age >= 400ms")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AgeFilter2 Old"))
        .stdout(predicate::str::contains("AgeFilter2 New").not());
}

#[test]
fn list_filter_short_flag() {
    let temp = init_temp();
    create_issue(&temp, "task", "FilterShort Task");

    wk().arg("list")
        .arg("-q")
        .arg("age < 1h")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn list_filter_invalid_field() {
    let temp = init_temp();

    wk().arg("list")
        .arg("--filter")
        .arg("invalid < 3d")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown filter field"));
}

#[test]
fn list_filter_invalid_operator() {
    let temp = init_temp();

    wk().arg("list")
        .arg("--filter")
        .arg("age << 3d")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid filter operator"));
}

#[test]
fn list_filter_invalid_duration() {
    let temp = init_temp();

    wk().arg("list")
        .arg("--filter")
        .arg("age < 3x")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid duration"));
}

#[test]
fn list_filter_multiple() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "task",
        "MultiFilter Task",
        &["--label", "team:alpha"],
    );

    wk().arg("list")
        .arg("--filter")
        .arg("age < 1h")
        .arg("--filter")
        .arg("updated < 1h")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("MultiFilter"));
}

#[test]
fn list_filter_combined_with_flags() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "task",
        "MultiFilter2 Task",
        &["--label", "team:alpha"],
    );
    create_issue_with_opts(&temp, "bug", "MultiFilter2 Bug", &["--label", "team:alpha"]);

    wk().arg("list")
        .arg("--filter")
        .arg("age < 1h")
        .arg("--type")
        .arg("task")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("MultiFilter2 Task"))
        .stdout(predicate::str::contains("MultiFilter2 Bug").not());
}

#[test]
fn list_filter_closed() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ClosedFilter Issue");
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

    // Without filter, done hidden
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ClosedFilter Issue").not());

    // With closed filter, shown
    wk().arg("list")
        .arg("--filter")
        .arg("closed < 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ClosedFilter Issue"));
}

#[test]
fn list_filter_closed_includes_both() {
    let temp = init_temp();

    let done_id = create_issue(&temp, "task", "ClosedStatus Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let closed_id = create_issue(&temp, "task", "ClosedStatus Closed");
    wk().arg("close")
        .arg(&closed_id)
        .arg("--reason")
        .arg("duplicate")
        .current_dir(temp.path())
        .assert()
        .success();

    create_issue(&temp, "task", "ClosedStatus Open");

    wk().arg("list")
        .arg("--filter")
        .arg("closed < 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ClosedStatus Done"))
        .stdout(predicate::str::contains("ClosedStatus Closed"))
        .stdout(predicate::str::contains("ClosedStatus Open").not());
}

#[test]
fn list_filter_completed() {
    let temp = init_temp();

    let done_id = create_issue(&temp, "task", "CompletedFilter Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let closed_id = create_issue(&temp, "task", "CompletedFilter Cancelled");
    wk().arg("close")
        .arg(&closed_id)
        .arg("--reason")
        .arg("wontfix")
        .current_dir(temp.path())
        .assert()
        .success();

    create_issue(&temp, "task", "CompletedFilter Open");

    wk().arg("list")
        .arg("--filter")
        .arg("completed < 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("CompletedFilter Done"))
        .stdout(predicate::str::contains("CompletedFilter Cancelled").not())
        .stdout(predicate::str::contains("CompletedFilter Open").not());
}

#[test]
fn list_filter_completed_synonym_done() {
    let temp = init_temp();

    let done_id = create_issue(&temp, "task", "CompletedSynonym Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let closed_id = create_issue(&temp, "task", "CompletedSynonym Cancelled");
    wk().arg("close")
        .arg(&closed_id)
        .arg("--reason")
        .arg("wontfix")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--filter")
        .arg("done < 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("CompletedSynonym Done"))
        .stdout(predicate::str::contains("CompletedSynonym Cancelled").not());
}

#[test]
fn list_filter_skipped() {
    let temp = init_temp();

    let done_id = create_issue(&temp, "task", "SkippedFilter Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let closed_id = create_issue(&temp, "task", "SkippedFilter Cancelled");
    wk().arg("close")
        .arg(&closed_id)
        .arg("--reason")
        .arg("wontfix")
        .current_dir(temp.path())
        .assert()
        .success();

    create_issue(&temp, "task", "SkippedFilter Open");

    wk().arg("list")
        .arg("--filter")
        .arg("skipped < 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SkippedFilter Cancelled"))
        .stdout(predicate::str::contains("SkippedFilter Done").not())
        .stdout(predicate::str::contains("SkippedFilter Open").not());
}

#[test]
fn list_filter_skipped_synonym_cancelled() {
    let temp = init_temp();

    let closed_id = create_issue(&temp, "task", "SkippedSynonym Cancelled");
    wk().arg("close")
        .arg(&closed_id)
        .arg("--reason")
        .arg("wontfix")
        .current_dir(temp.path())
        .assert()
        .success();

    let done_id = create_issue(&temp, "task", "SkippedSynonym Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--filter")
        .arg("cancelled < 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SkippedSynonym Cancelled"))
        .stdout(predicate::str::contains("SkippedSynonym Done").not());
}

#[test]
fn list_filter_bare_closed() {
    let temp = init_temp();

    create_issue(&temp, "task", "BareFilter Open");

    let done_id = create_issue(&temp, "task", "BareFilter Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let skipped_id = create_issue(&temp, "task", "BareFilter Skipped");
    wk().arg("close")
        .arg(&skipped_id)
        .arg("--reason")
        .arg("wontfix")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--filter")
        .arg("closed")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BareFilter Done"))
        .stdout(predicate::str::contains("BareFilter Skipped"))
        .stdout(predicate::str::contains("BareFilter Open").not());
}

#[test]
fn list_filter_bare_completed() {
    let temp = init_temp();

    create_issue(&temp, "task", "BareCompleted Open");

    let done_id = create_issue(&temp, "task", "BareCompleted Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let skipped_id = create_issue(&temp, "task", "BareCompleted Skipped");
    wk().arg("close")
        .arg(&skipped_id)
        .arg("--reason")
        .arg("wontfix")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--filter")
        .arg("completed")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BareCompleted Done"))
        .stdout(predicate::str::contains("BareCompleted Skipped").not())
        .stdout(predicate::str::contains("BareCompleted Open").not());
}

#[test]
fn list_filter_bare_skipped() {
    let temp = init_temp();

    create_issue(&temp, "task", "BareSkipped Open");

    let done_id = create_issue(&temp, "task", "BareSkipped Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let skipped_id = create_issue(&temp, "task", "BareSkipped Skipped");
    wk().arg("close")
        .arg(&skipped_id)
        .arg("--reason")
        .arg("wontfix")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--filter")
        .arg("skipped")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BareSkipped Skipped"))
        .stdout(predicate::str::contains("BareSkipped Done").not())
        .stdout(predicate::str::contains("BareSkipped Open").not());
}

#[test]
fn list_filter_bare_aliases() {
    let temp = init_temp();

    let done_id = create_issue(&temp, "task", "AliasFilter Done");
    wk().arg("start")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("done")
        .arg(&done_id)
        .current_dir(temp.path())
        .assert()
        .success();

    let skipped_id = create_issue(&temp, "task", "AliasFilter Skipped");
    wk().arg("close")
        .arg(&skipped_id)
        .arg("--reason")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();

    // "done" is alias for "completed"
    wk().arg("list")
        .arg("--filter")
        .arg("done")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AliasFilter Done"));

    // "cancelled" is alias for "skipped"
    wk().arg("list")
        .arg("--filter")
        .arg("cancelled")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AliasFilter Skipped"))
        .stdout(predicate::str::contains("AliasFilter Done").not());
}

#[test]
fn list_filter_bare_age_fails() {
    let temp = init_temp();

    wk().arg("list")
        .arg("--filter")
        .arg("age")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires operator"));
}

#[test]
fn list_filter_bare_updated_fails() {
    let temp = init_temp();

    wk().arg("list")
        .arg("--filter")
        .arg("updated")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires operator"));
}

#[test]
fn list_filter_accepts_now() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "NowFilter Issue");
    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();

    // closed < now should match (closed before current time)
    wk().arg("list")
        .arg("--filter")
        .arg("closed < now")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("NowFilter Issue"));

    // closed > now should not match (nothing closed in the future)
    wk().arg("list")
        .arg("--filter")
        .arg("closed > now")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("NowFilter Issue").not());
}

#[test]
fn list_filter_word_lt() {
    let temp = init_temp();
    create_issue(&temp, "task", "WordOp Old");
    std::thread::sleep(std::time::Duration::from_millis(500));
    create_issue(&temp, "task", "WordOp New");

    wk().arg("list")
        .arg("--filter")
        .arg("age lt 400ms")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("WordOp New"))
        .stdout(predicate::str::contains("WordOp Old").not());
}

#[test]
fn list_filter_word_gte() {
    let temp = init_temp();
    create_issue(&temp, "task", "WordOp2 Old");
    std::thread::sleep(std::time::Duration::from_millis(500));
    create_issue(&temp, "task", "WordOp2 New");

    wk().arg("list")
        .arg("--filter")
        .arg("age gte 400ms")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("WordOp2 Old"))
        .stdout(predicate::str::contains("WordOp2 New").not());
}

#[test]
fn list_filter_word_gt() {
    let temp = init_temp();
    create_issue(&temp, "task", "WordOp3 Old");
    std::thread::sleep(std::time::Duration::from_millis(500));
    create_issue(&temp, "task", "WordOp3 New");

    wk().arg("list")
        .arg("--filter")
        .arg("age gt 300ms")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("WordOp3 Old"));
}

#[test]
fn list_filter_word_lte() {
    let temp = init_temp();
    create_issue(&temp, "task", "WordOp4 Old");
    create_issue(&temp, "task", "WordOp4 New");

    wk().arg("list")
        .arg("--filter")
        .arg("age lte 1d")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("WordOp4 New"))
        .stdout(predicate::str::contains("WordOp4 Old"));
}

#[test]
fn list_filter_word_case_insensitive() {
    let temp = init_temp();
    create_issue(&temp, "task", "WordCase Task");

    wk().arg("list")
        .arg("--filter")
        .arg("age LT 1d")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--filter")
        .arg("age GT 0ms")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Phase 6: Limit and IDs Output Tests
// =============================================================================

#[test]
fn list_defaults_to_100_results() {
    let temp = init_temp();

    // Create more than 100 issues
    for i in 1..=105 {
        create_issue(&temp, "task", &format!("DefaultLimit Issue {}", i));
    }

    let output = wk().arg("list").current_dir(temp.path()).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("- [").count();
    assert!(count <= 100, "Should return at most 100, got {}", count);
}

#[test]
fn list_limit_truncates() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "task", "Limit 1", &["--label", "limit-tag"]);
    create_issue_with_opts(&temp, "task", "Limit 2", &["--label", "limit-tag"]);
    create_issue_with_opts(&temp, "task", "Limit 3", &["--label", "limit-tag"]);

    let output = wk()
        .arg("list")
        .arg("--label")
        .arg("limit-tag")
        .arg("--limit")
        .arg("2")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("Limit").count();
    assert_eq!(count, 2, "Should return exactly 2");
}

#[test]
fn list_limit_short_flag() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "task", "LimitShort 1", &["--label", "limit-short"]);
    create_issue_with_opts(&temp, "task", "LimitShort 2", &["--label", "limit-short"]);
    create_issue_with_opts(&temp, "task", "LimitShort 3", &["--label", "limit-short"]);

    let output = wk()
        .arg("list")
        .arg("--label")
        .arg("limit-short")
        .arg("-n")
        .arg("1")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("LimitShort").count();
    assert_eq!(count, 1, "Should return exactly 1");
}

#[test]
fn list_limit_0_unlimited() {
    let temp = init_temp();

    // Create 15 issues with a unique label
    for i in 1..=15 {
        create_issue_with_opts(
            &temp,
            "task",
            &format!("UnlimitedTest Issue {}", i),
            &["--label", "test:unlimited"],
        );
    }

    let output = wk()
        .arg("list")
        .arg("--label")
        .arg("test:unlimited")
        .arg("--limit")
        .arg("0")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("- [").count();
    assert_eq!(count, 15, "Should return all 15 issues");
}

#[test]
fn list_explicit_limit_overrides() {
    let temp = init_temp();

    // Create 50 issues with a unique label
    for i in 1..=50 {
        create_issue_with_opts(
            &temp,
            "task",
            &format!("ExplicitLimit Issue {}", i),
            &["--label", "test:explicit"],
        );
    }

    let output = wk()
        .arg("list")
        .arg("--label")
        .arg("test:explicit")
        .arg("--limit")
        .arg("20")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("- [").count();
    assert_eq!(count, 20, "Should return exactly 20");
}

#[test]
fn list_json_metadata_filters_applied() {
    let temp = init_temp();
    create_issue(&temp, "task", "JSONMeta Issue");

    let output = wk()
        .arg("list")
        .arg("--filter")
        .arg("age < 1d")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Output is a plain array, no metadata wrapper
    assert!(json.as_array().is_some(), "Output should be a plain array");
    assert!(
        json.as_array().unwrap().len() == 1,
        "Should have one issue matching filter"
    );
}

#[test]
fn list_json_metadata_limit() {
    let temp = init_temp();
    create_issue(&temp, "task", "JSONLimit Issue");

    let output = wk()
        .arg("list")
        .arg("--limit")
        .arg("10")
        .arg("--output")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Output is a plain array, limit is applied but not in output
    assert!(json.as_array().is_some(), "Output should be a plain array");
}

#[test]
fn list_output_ids_space_separated() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "IDFormat Issue 1");
    let id2 = create_issue(&temp, "task", "IDFormat Issue 2");

    let output = wk()
        .arg("list")
        .arg("--output")
        .arg("ids")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&id1));
    assert!(stdout.contains(&id2));
    assert!(!stdout.contains("task"));
    assert!(!stdout.contains("todo"));
    assert!(!stdout.contains("IDFormat"));

    // Single line
    let line_count = stdout.trim().lines().count();
    assert_eq!(line_count, 1);
}

#[test]
fn list_output_ids_no_metadata() {
    let temp = init_temp();
    create_issue(&temp, "task", "IDNoMeta Issue");

    let output = wk()
        .arg("list")
        .arg("--output")
        .arg("ids")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("task"));
    assert!(!stdout.contains("todo"));
    assert!(!stdout.contains("IDNoMeta"));
}

#[test]
fn list_output_ids_with_filters() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "FilterID Task");
    create_issue(&temp, "bug", "FilterID Bug");

    let output = wk()
        .arg("list")
        .arg("--type")
        .arg("task")
        .arg("--output")
        .arg("ids")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&id));
    assert!(!stdout.contains("FilterID Bug"));
}

#[test]
fn list_output_ids_respects_limit() {
    let temp = init_temp();

    for i in 1..=15 {
        create_issue_with_opts(
            &temp,
            "task",
            &format!("LimitID Issue {}", i),
            &["--label", "test:limit-ids"],
        );
    }

    let output = wk()
        .arg("list")
        .arg("--label")
        .arg("test:limit-ids")
        .arg("--output")
        .arg("ids")
        .arg("--limit")
        .arg("10")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.split_whitespace().count();
    assert_eq!(count, 10, "Should return exactly 10 IDs");
}

#[test]
fn list_output_ids_short_flag() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ShortFlagID Issue");

    let output = wk()
        .arg("list")
        .arg("-o")
        .arg("ids")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&id));
}

#[test]
fn list_output_ids_clean_format() {
    let temp = init_temp();
    create_issue(&temp, "task", "Pipe Test Issue");

    let output = wk()
        .arg("list")
        .arg("--output")
        .arg("ids")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output should be space-separated IDs (alphanumeric with hyphens only)
    let re = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
    for word in stdout.split_whitespace() {
        assert!(
            re.is_match(word),
            "ID format should be alphanumeric with hyphens: {}",
            word
        );
    }
}
