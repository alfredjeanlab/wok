// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for CLI flags.
//! Converted from tests/specs/cli/unit/flags.bats
//!
//! Tests verifying all documented --flags actually work based on REQUIREMENTS.md.

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

// =============================================================================
// Init Flags: --prefix and --path
// =============================================================================

#[test]
fn init_flags_prefix() {
    let temp = TempDir::new().unwrap();

    // --prefix creates project with custom prefix
    wk().arg("init")
        .arg("--prefix")
        .arg("myproj")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    let id = create_issue(&temp, "task", "Test task");
    assert!(id.starts_with("myproj-"), "Issue ID should start with 'myproj-', got: {}", id);
}

#[test]
fn init_flags_path() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("subdir")).unwrap();

    // --path creates project in specified path
    wk().arg("init")
        .arg("--path")
        .arg("subdir")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("subdir/.wok").exists());
}

// =============================================================================
// New Flags: --label and --note
// =============================================================================

#[test]
fn new_flags_label_adds_label() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Test task", &["--label", "mylabel"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("mylabel"));
}

#[test]
fn new_flags_multiple_labels() {
    let temp = init_temp();
    let id = create_issue_with_opts(
        &temp,
        "task",
        "Test task",
        &["--label", "label1", "--label", "label2"],
    );

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("label1"))
        .stdout(predicate::str::contains("label2"));
}

#[test]
fn new_flags_note_adds_initial_note() {
    let temp = init_temp();
    let id =
        create_issue_with_opts(&temp, "task", "Test task", &["--note", "Initial note content"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial note content"));
}

#[test]
fn new_flags_label_and_note_together() {
    let temp = init_temp();
    let id = create_issue_with_opts(
        &temp,
        "task",
        "Test task",
        &["--label", "mylabel", "--note", "My note"],
    );

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("mylabel"))
        .stdout(predicate::str::contains("My note"));
}

// =============================================================================
// Done/Close Flags: --reason records in log
// =============================================================================

#[test]
fn done_flag_reason_records_in_log() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test done");

    wk().arg("done")
        .arg(&id)
        .arg("--reason")
        .arg("Already fixed upstream")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("log")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Already fixed"));
}

#[test]
fn close_flag_reason_records_in_log() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test close");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("Duplicate issue")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("log")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Duplicate"));
}

// =============================================================================
// Reopen Flags: --reason records in log
// =============================================================================

#[test]
fn reopen_flag_reason_records_in_log() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test reopen");

    wk().arg("start").arg(&id).current_dir(temp.path()).assert().success();
    wk().arg("done").arg(&id).current_dir(temp.path()).assert().success();

    wk().arg("reopen")
        .arg(&id)
        .arg("--reason")
        .arg("Found regression")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("log")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("regression"));
}

// =============================================================================
// Edit Positional Args: title and type
// =============================================================================

#[test]
fn edit_title_changes_issue_title() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original title");

    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg("New title")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"));
}

#[test]
fn edit_type_changes_issue_type() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    wk().arg("edit").arg(&id).arg("type").arg("bug").current_dir(temp.path()).assert().success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[bug]"));
}

#[test]
fn edit_title_and_type_work_sequentially() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original");

    wk().arg("edit")
        .arg(&id)
        .arg("title")
        .arg("Changed")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("edit")
        .arg(&id)
        .arg("type")
        .arg("feature")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Changed"))
        .stdout(predicate::str::contains("[feature]"));
}

// =============================================================================
// List --status filters by status
// =============================================================================

#[parameterized(
    todo = { "todo", "StatusFlag Todo", "StatusFlag Started" },
    in_progress = { "in_progress", "StatusFlag Started", "StatusFlag Todo" },
    done = { "done", "StatusFlag Done", "StatusFlag Todo" },
    closed = { "closed", "StatusFlag Closed", "StatusFlag Todo" },
)]
fn list_status_filters_by_status(status: &str, expected: &str, not_expected: &str) {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "StatusFlag Todo");
    let id2 = create_issue(&temp, "task", "StatusFlag Started");
    let id3 = create_issue(&temp, "task", "StatusFlag Done");
    let id4 = create_issue(&temp, "task", "StatusFlag Closed");

    wk().arg("start").arg(&id2).current_dir(temp.path()).assert().success();
    wk().arg("start").arg(&id3).current_dir(temp.path()).assert().success();
    wk().arg("done").arg(&id3).current_dir(temp.path()).assert().success();
    wk().arg("close")
        .arg(&id4)
        .arg("--reason")
        .arg("Won't fix")
        .current_dir(temp.path())
        .assert()
        .success();

    // Suppress unused variable warnings
    let _ = id1;

    wk().arg("list")
        .arg("--status")
        .arg(status)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected))
        .stdout(predicate::str::contains(not_expected).not());
}

// =============================================================================
// List --type filters by type (including short flag)
// =============================================================================

#[parameterized(
    feature = { "feature", "TypeFlag My feature", "TypeFlag My task" },
    task = { "task", "TypeFlag My task", "TypeFlag My bug" },
    bug = { "bug", "TypeFlag My bug", "TypeFlag My task" },
)]
fn list_type_filters_by_type(type_filter: &str, expected: &str, not_expected: &str) {
    let temp = init_temp();
    create_issue(&temp, "task", "TypeFlag My task");
    create_issue(&temp, "feature", "TypeFlag My feature");
    create_issue(&temp, "bug", "TypeFlag My bug");

    wk().arg("list")
        .arg("--type")
        .arg(type_filter)
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected))
        .stdout(predicate::str::contains(not_expected).not());
}

#[test]
fn list_type_short_flag_t_works() {
    let temp = init_temp();
    create_issue(&temp, "task", "TypeShort My task");
    create_issue(&temp, "bug", "TypeShort My bug");

    wk().arg("list")
        .arg("-t")
        .arg("bug")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeShort My bug"))
        .stdout(predicate::str::contains("TypeShort My task").not());
}

#[test]
fn list_type_comma_separated() {
    let temp = init_temp();
    create_issue(&temp, "task", "TypeComma My task");
    create_issue(&temp, "feature", "TypeComma My feature");
    create_issue(&temp, "bug", "TypeComma My bug");

    wk().arg("list")
        .arg("-t")
        .arg("bug,task")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeComma My task"))
        .stdout(predicate::str::contains("TypeComma My bug"))
        .stdout(predicate::str::contains("TypeComma My feature").not());
}

// =============================================================================
// List --label, --all, --blocked filters
// =============================================================================

#[test]
fn list_label_filters_by_label() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "task", "LabelFlag Labeled task", &["--label", "findme"]);
    create_issue(&temp, "task", "LabelFlag Unlabeled task");

    wk().arg("list")
        .arg("--label")
        .arg("findme")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("LabelFlag Labeled task"))
        .stdout(predicate::str::contains("LabelFlag Unlabeled task").not());
}

#[test]
fn list_all_includes_blocked_issues() {
    let temp = init_temp();
    let blocker = create_issue(&temp, "task", "BlockFlag Blocker");
    let blocked = create_issue(&temp, "task", "BlockFlag Blocked");

    wk().arg("dep")
        .arg(&blocker)
        .arg("blocks")
        .arg(&blocked)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BlockFlag Blocker"))
        .stdout(predicate::str::contains("BlockFlag Blocked"));
}

#[test]
fn list_blocked_shows_only_blocked_issues() {
    let temp = init_temp();
    let blocker = create_issue(&temp, "task", "BlockOnly Blocker");
    let blocked = create_issue(&temp, "task", "BlockOnly Blocked");

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
        .stdout(predicate::str::contains("BlockOnly Blocked"))
        .stdout(predicate::str::contains("BlockOnly Blocker").not());
}

#[test]
fn list_blocked_short_flag_b_not_supported() {
    let temp = init_temp();

    wk().arg("list").arg("-b").current_dir(temp.path()).assert().failure();
}

#[test]
fn list_default_shows_blocked_issues() {
    let temp = init_temp();
    let blocker = create_issue(&temp, "task", "DefaultBlk Blocker");
    let blocked = create_issue(&temp, "task", "DefaultBlk Blocked");

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
        .stdout(predicate::str::contains("DefaultBlk Blocker"))
        .stdout(predicate::str::contains("DefaultBlk Blocked"));
}

// =============================================================================
// List combined filters: --status --type --label
// =============================================================================

#[test]
fn list_combined_status_and_type() {
    let temp = init_temp();
    create_issue(&temp, "task", "CombFlag Todo task");
    let bug_id = create_issue(&temp, "bug", "CombFlag Todo bug");

    wk().arg("start").arg(&bug_id).current_dir(temp.path()).assert().success();

    wk().arg("list")
        .arg("--status")
        .arg("in_progress")
        .arg("--type")
        .arg("bug")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("CombFlag Todo bug"))
        .stdout(predicate::str::contains("CombFlag Todo task").not());
}

#[test]
fn list_combined_label_and_status() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "task", "CombLabel Tagged todo", &["--label", "mylabel"]);
    let started_id =
        create_issue_with_opts(&temp, "task", "CombLabel Tagged started", &["--label", "mylabel"]);

    wk().arg("start").arg(&started_id).current_dir(temp.path()).assert().success();

    wk().arg("list")
        .arg("--label")
        .arg("mylabel")
        .arg("--status")
        .arg("in_progress")
        .arg("--all")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("CombLabel Tagged started"))
        .stdout(predicate::str::contains("CombLabel Tagged todo").not());
}

// =============================================================================
// Log --limit limits output
// =============================================================================

#[test]
fn log_limit_limits_output() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogLimit task");

    wk().arg("note").arg(&id).arg("Note 1").current_dir(temp.path()).assert().success();
    wk().arg("note").arg(&id).arg("Note 2").current_dir(temp.path()).assert().success();
    wk().arg("note").arg(&id).arg("Note 3").current_dir(temp.path()).assert().success();
    wk().arg("start").arg(&id).current_dir(temp.path()).assert().success();

    wk().arg("log").arg(&id).arg("--limit").arg("2").current_dir(temp.path()).assert().success();
}

#[test]
fn log_limit_1_shows_most_recent() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LogLimit1 task");

    wk().arg("note").arg(&id).arg("Note 1").current_dir(temp.path()).assert().success();
    wk().arg("start").arg(&id).current_dir(temp.path()).assert().success();

    wk().arg("log")
        .arg(&id)
        .arg("--limit")
        .arg("1")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("started"));
}

// =============================================================================
// Ready filters: --type, --label, and combined
// =============================================================================

#[test]
fn ready_type_filters_items() {
    let temp = init_temp();
    create_issue(&temp, "task", "ReadyFlag Ready task");
    create_issue(&temp, "bug", "ReadyFlag Ready bug");

    wk().arg("ready")
        .arg("--type")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadyFlag Ready bug"))
        .stdout(predicate::str::contains("ReadyFlag Ready task").not());
}

#[test]
fn ready_type_short_flag_t_works() {
    let temp = init_temp();
    create_issue(&temp, "task", "ReadyShort Ready task");
    create_issue(&temp, "bug", "ReadyShort Ready bug");

    wk().arg("ready")
        .arg("-t")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadyShort Ready bug"))
        .stdout(predicate::str::contains("ReadyShort Ready task").not());
}

#[test]
fn ready_label_filters_items() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "task", "ReadyLabel Labeled task", &["--label", "urgent"]);
    create_issue(&temp, "task", "ReadyLabel Unlabeled task");

    wk().arg("ready")
        .arg("--label")
        .arg("urgent")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadyLabel Labeled task"))
        .stdout(predicate::str::contains("ReadyLabel Unlabeled task").not());
}

#[test]
fn ready_type_and_label_combined() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "bug", "ReadyComb Labeled bug", &["--label", "team:alpha"]);
    create_issue_with_opts(
        &temp,
        "task",
        "ReadyComb Other labeled task",
        &["--label", "team:alpha"],
    );
    let blocker =
        create_issue_with_opts(&temp, "bug", "ReadyComb Blocker bug", &["--label", "team:alpha"]);
    let blocked =
        create_issue_with_opts(&temp, "bug", "ReadyComb Blocked bug", &["--label", "team:alpha"]);

    wk().arg("dep")
        .arg(&blocker)
        .arg("blocks")
        .arg(&blocked)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("ready")
        .arg("--type")
        .arg("bug")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadyComb Labeled bug"))
        .stdout(predicate::str::contains("ReadyComb Blocker bug"))
        .stdout(predicate::str::contains("ReadyComb Other labeled task").not())
        .stdout(predicate::str::contains("ReadyComb Blocked bug").not());
}

// =============================================================================
// Ready rejects --all and --blocked flags
// =============================================================================

#[test]
fn ready_rejects_all_flag() {
    let temp = init_temp();

    wk().arg("ready").arg("--all").current_dir(temp.path()).assert().failure();
}

#[test]
fn ready_rejects_blocked_flag() {
    let temp = init_temp();

    wk().arg("ready").arg("--blocked").current_dir(temp.path()).assert().failure();
}

// =============================================================================
// -C flag: runs command in specified directory
// =============================================================================

#[test]
fn c_flag_runs_command_in_specified_directory() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("other-project")).unwrap();

    wk().arg("init")
        .arg("--path")
        .arg("other-project")
        .arg("--prefix")
        .arg("othr")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("-C")
        .arg("other-project")
        .arg("new")
        .arg("task")
        .arg("Remote task")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("-C")
        .arg("other-project")
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote task"));
}

#[test]
fn c_flag_error_on_nonexistent_directory() {
    let temp = TempDir::new().unwrap();

    wk().arg("-C")
        .arg("/nonexistent/path")
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot change to directory"));
}

#[test]
fn c_flag_works_with_init() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("newproj")).unwrap();

    wk().arg("-C")
        .arg("newproj")
        .arg("init")
        .arg("--prefix")
        .arg("np")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("newproj/.wok").exists());
}
