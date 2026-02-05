// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for filtering behavior across `wk list` and `wk ready` commands.
//! Converted from tests/specs/cli/integration/filtering.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::common::*;
use yare::parameterized;

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
// Default List Behavior
// =============================================================================

#[test]
fn default_list_shows_all_open_issues_including_blocked() {
    let temp = init_temp();
    let t1 = create_issue(&temp, "task", "FilterDefault Ready task");
    let t2 = create_issue(&temp, "task", "FilterDefault Blocked task");

    wk().arg("dep")
        .arg(&t1)
        .arg("blocks")
        .arg(&t2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("FilterDefault Ready task"))
        .stdout(predicate::str::contains("FilterDefault Blocked task"));
}

// =============================================================================
// Status Filter Tests
// =============================================================================

#[test]
fn status_filter_works_alone() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "StatusFilterTest");

    wk().arg("start")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--status")
        .arg("in_progress")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("StatusFilterTest"));

    wk().arg("list")
        .arg("--status")
        .arg("todo")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("StatusFilterTest").not());
}

#[test]
fn closed_items_only_appear_with_status_closed() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ClosedItemTest");

    wk().arg("close")
        .arg(&id)
        .arg("--reason")
        .arg("done")
        .current_dir(temp.path())
        .assert()
        .success();

    // Should not appear in default list
    wk().arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ClosedItemTest").not());

    // Should appear with --status closed
    wk().arg("list")
        .arg("--status")
        .arg("closed")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ClosedItemTest"));
}

// =============================================================================
// Type Filter Tests (Parameterized)
// =============================================================================

#[parameterized(
    task = { "task", "Task" },
    bug = { "bug", "Bug" },
    feature = { "feature", "Feature" },
    chore = { "chore", "Chore" },
)]
fn type_filter_works_alone(filter_type: &str, label: &str) {
    let temp = init_temp();
    create_issue(&temp, "task", "TypeFilter Task item");
    create_issue(&temp, "bug", "TypeFilter Bug item");
    create_issue(&temp, "feature", "TypeFilter Feature item");
    create_issue(&temp, "chore", "TypeFilter Chore item");

    let expected = format!("TypeFilter {} item", label);

    wk().arg("list")
        .arg("--type")
        .arg(filter_type)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&expected));

    // Verify others are excluded
    for (other_type, other_label) in [
        ("task", "Task"),
        ("bug", "Bug"),
        ("feature", "Feature"),
        ("chore", "Chore"),
    ] {
        if other_type != filter_type {
            let not_expected = format!("TypeFilter {} item", other_label);
            wk().arg("list")
                .arg("--type")
                .arg(filter_type)
                .current_dir(temp.path())
                .assert()
                .stdout(predicate::str::contains(&not_expected).not());
        }
    }
}

#[parameterized(
    bug = { "bug" },
    chore = { "chore" },
    task = { "task" },
)]
fn type_filter_with_short_flag(filter_type: &str) {
    let temp = init_temp();
    create_issue(&temp, "task", "TypeShort Task item");
    create_issue(&temp, "bug", "TypeShort Bug item");
    create_issue(&temp, "chore", "TypeShort Chore item");

    wk().arg("list")
        .arg("-t")
        .arg(filter_type)
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Label Filter Tests
// =============================================================================

#[test]
fn label_filter_works_alone() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "task",
        "LabelFilter Labeled",
        &["--label", "team:alpha"],
    );
    create_issue(&temp, "task", "LabelFilter Other");

    wk().arg("list")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("LabelFilter Labeled"))
        .stdout(predicate::str::contains("LabelFilter Other").not());
}

#[test]
fn multiple_labels_filter() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "task",
        "MultiLabel Has both",
        &["--label", "a", "--label", "b"],
    );
    create_issue_with_opts(&temp, "task", "MultiLabel Has one", &["--label", "a"]);

    wk().arg("list")
        .arg("--label")
        .arg("a")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("MultiLabel Has both"))
        .stdout(predicate::str::contains("MultiLabel Has one"));
}

// =============================================================================
// Combined Filter Tests
// =============================================================================

#[test]
fn status_plus_type_combined() {
    let temp = init_temp();
    let t = create_issue(&temp, "task", "Combined Active task");
    let b = create_issue(&temp, "bug", "Combined Active bug");

    wk().arg("start")
        .arg(&t)
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("start")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    create_issue(&temp, "task", "Combined Todo task");

    wk().arg("list")
        .arg("--status")
        .arg("in_progress")
        .arg("--type")
        .arg("task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Combined Active task"))
        .stdout(predicate::str::contains("Combined Active bug").not())
        .stdout(predicate::str::contains("Combined Todo task").not());
}

#[test]
fn status_plus_label_combined() {
    let temp = init_temp();
    let t = create_issue_with_opts(
        &temp,
        "task",
        "StatusLabel Labeled active",
        &["--label", "important"],
    );

    wk().arg("start")
        .arg(&t)
        .current_dir(temp.path())
        .assert()
        .success();

    create_issue_with_opts(
        &temp,
        "task",
        "StatusLabel Labeled todo",
        &["--label", "important"],
    );

    wk().arg("list")
        .arg("--status")
        .arg("in_progress")
        .arg("--label")
        .arg("important")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("StatusLabel Labeled active"))
        .stdout(predicate::str::contains("StatusLabel Labeled todo").not());
}

#[test]
fn type_plus_label_combined() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "bug",
        "TypeLabel Labeled bug",
        &["--label", "team:alpha"],
    );
    create_issue_with_opts(
        &temp,
        "task",
        "TypeLabel Labeled task",
        &["--label", "team:alpha"],
    );
    create_issue(&temp, "bug", "TypeLabel Other bug");

    wk().arg("list")
        .arg("--type")
        .arg("bug")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeLabel Labeled bug"))
        .stdout(predicate::str::contains("TypeLabel Labeled task").not())
        .stdout(predicate::str::contains("TypeLabel Other bug").not());
}

#[test]
fn chore_plus_label_combined_filter() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "chore",
        "ChoreLabel Labeled chore",
        &["--label", "refactor"],
    );
    create_issue_with_opts(
        &temp,
        "task",
        "ChoreLabel Labeled task",
        &["--label", "refactor"],
    );
    create_issue(&temp, "chore", "ChoreLabel Other chore");

    wk().arg("list")
        .arg("--type")
        .arg("chore")
        .arg("--label")
        .arg("refactor")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ChoreLabel Labeled chore"))
        .stdout(predicate::str::contains("ChoreLabel Labeled task").not())
        .stdout(predicate::str::contains("ChoreLabel Other chore").not());
}

#[test]
fn all_three_filters_combined() {
    let temp = init_temp();
    let t = create_issue_with_opts(
        &temp,
        "task",
        "TripleFilter Match",
        &["--label", "team:alpha"],
    );

    wk().arg("start")
        .arg(&t)
        .current_dir(temp.path())
        .assert()
        .success();

    let b = create_issue_with_opts(
        &temp,
        "bug",
        "TripleFilter No match",
        &["--label", "team:alpha"],
    );
    wk().arg("start")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--status")
        .arg("in_progress")
        .arg("--type")
        .arg("task")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TripleFilter Match"))
        .stdout(predicate::str::contains("TripleFilter No match").not());
}

// =============================================================================
// --all and --blocked with Filters
// =============================================================================

#[test]
fn all_with_filters() {
    let temp = init_temp();
    let a = create_issue_with_opts(
        &temp,
        "task",
        "AllFilter Blocker",
        &["--label", "team:alpha"],
    );
    let b = create_issue_with_opts(
        &temp,
        "task",
        "AllFilter Blocked",
        &["--label", "team:alpha"],
    );
    create_issue_with_opts(
        &temp,
        "task",
        "AllFilter Other blocked",
        &["--label", "team:beta"],
    );

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--all")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AllFilter Blocker"))
        .stdout(predicate::str::contains("AllFilter Blocked"))
        .stdout(predicate::str::contains("AllFilter Other blocked").not());
}

#[test]
fn blocked_with_filters() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "BlockedFilter Blocker");
    let b = create_issue(&temp, "bug", "BlockedFilter Blocked bug");
    let c = create_issue(&temp, "task", "BlockedFilter Blocked task");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("list")
        .arg("--blocked")
        .arg("--type")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BlockedFilter Blocked bug"))
        .stdout(predicate::str::contains("BlockedFilter Blocked task").not());
}

// =============================================================================
// Empty Result Set
// =============================================================================

#[test]
fn empty_result_set_handled_gracefully() {
    let temp = init_temp();
    create_issue(&temp, "task", "EmptyResult Task");

    // Filtering for a type with no results should succeed
    wk().arg("list")
        .arg("--type")
        .arg("feature")
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Ready Command Filter Tests
// =============================================================================

#[test]
fn ready_type_plus_label_combined() {
    let temp = init_temp();
    create_issue_with_opts(
        &temp,
        "bug",
        "ReadyFilter Labeled bug",
        &["--label", "team:alpha"],
    );
    create_issue_with_opts(
        &temp,
        "task",
        "ReadyFilter Labeled task",
        &["--label", "team:alpha"],
    );

    wk().arg("ready")
        .arg("--type")
        .arg("bug")
        .arg("--label")
        .arg("team:alpha")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadyFilter Labeled bug"))
        .stdout(predicate::str::contains("ReadyFilter Labeled task").not());
}

#[test]
fn ready_excludes_blocked_even_with_filters() {
    let temp = init_temp();
    let a = create_issue_with_opts(
        &temp,
        "bug",
        "ReadyBlocked Blocker bug",
        &["--label", "team:alpha"],
    );
    let b = create_issue_with_opts(
        &temp,
        "bug",
        "ReadyBlocked Blocked bug",
        &["--label", "team:alpha"],
    );

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
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
        .stdout(predicate::str::contains("ReadyBlocked Blocker bug"))
        .stdout(predicate::str::contains("ReadyBlocked Blocked bug").not());
}

#[test]
fn ready_with_short_flag_for_type() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "bug", "ReadyShort A bug");
    wk().arg("label")
        .arg(&id1)
        .arg("test:ready-shortflag")
        .current_dir(temp.path())
        .assert()
        .success();

    let id2 = create_issue(&temp, "task", "ReadyShort A task");
    wk().arg("label")
        .arg(&id2)
        .arg("test:ready-shortflag")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("ready")
        .arg("-t")
        .arg("bug")
        .arg("--label")
        .arg("test:ready-shortflag")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadyShort A bug"))
        .stdout(predicate::str::contains("ReadyShort A task").not());
}

#[test]
fn chore_in_ready_command() {
    let temp = init_temp();
    let a = create_issue(&temp, "chore", "ReadyChore Blocker chore");
    let b = create_issue(&temp, "chore", "ReadyChore Blocked chore");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("ready")
        .arg("--type")
        .arg("chore")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadyChore Blocker chore"))
        .stdout(predicate::str::contains("ReadyChore Blocked chore").not());
}
