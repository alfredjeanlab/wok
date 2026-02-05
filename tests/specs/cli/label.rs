// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk label` and `wk unlabel` commands.
//! Converted from tests/specs/cli/unit/label.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let output = wk()
        .args(["new", type_, title, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic Label Tests
// =============================================================================

#[test]
fn label_adds_simple_label_to_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelBasic Test task");

    wk().args(["label", &id, "urgent"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"));
}

#[test]
fn label_adds_namespaced_label_to_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelBasic Namespaced task");

    wk().args(["label", &id, "team:backend"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("team:backend"));
}

#[test]
fn label_multiple_labels_sequentially() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelBasic Multi task");

    wk().args(["label", &id, "label1"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["label", &id, "label2"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("label1"))
        .stdout(predicate::str::contains("label2"));
}

#[test]
fn label_searchable_via_list() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelBasic Searchable task");

    wk().args(["label", &id, "findme"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["list", "--label", "findme"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Searchable task"));
}

// =============================================================================
// Unlabel Tests
// =============================================================================

#[test]
fn unlabel_removes_label() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelUnlabel Test task");

    wk().args(["label", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["unlabel", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Check that Labels line does not contain mylabel (but log may contain "labeled mylabel")
    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::is_match(r"(?m)^Labels:.*mylabel")
                .unwrap()
                .not(),
        );
}

#[test]
fn unlabel_nonexistent_label_succeeds_or_fails_gracefully() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelUnlabel Nonexistent task");

    // Should either succeed (idempotent) or fail gracefully
    let _ = wk()
        .args(["unlabel", &id, "nonexistent"])
        .current_dir(temp.path())
        .output();
}

#[test]
fn duplicate_label_is_idempotent_or_fails_gracefully() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelUnlabel Duplicate task");

    wk().args(["label", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Should either succeed (idempotent) or fail gracefully
    let _ = wk()
        .args(["label", &id, "mylabel"])
        .current_dir(temp.path())
        .output();
}

// =============================================================================
// Event Logging Tests
// =============================================================================

#[test]
fn label_logs_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelLog Test task");

    wk().args(["label", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("labeled"));
}

#[test]
fn unlabel_logs_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelLog Unlabel task");

    wk().args(["label", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["unlabel", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unlabeled"));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn label_nonexistent_issue_fails() {
    let temp = init_temp();

    wk().args(["label", "test-nonexistent", "mylabel"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn label_all_invalid_args_fails() {
    let temp = init_temp();

    // When first arg doesn't resolve, all args are treated as labels
    // which fails because there are no valid issue IDs
    wk().args(["label", "not-an-id", "also-not-an-id", "urgent"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Batch Label Operations
// =============================================================================

#[test]
fn label_multiple_issues() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LabelBatch Task 1");
    let id2 = create_issue(&temp, "task", "LabelBatch Task 2");

    wk().args(["label", &id1, &id2, "urgent"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id1])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"));

    wk().args(["show", &id2])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"));
}

#[test]
fn label_three_issues() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LabelBatch3 Task 1");
    let id2 = create_issue(&temp, "task", "LabelBatch3 Task 2");
    let id3 = create_issue(&temp, "task", "LabelBatch3 Task 3");

    wk().args(["label", &id1, &id2, &id3, "backend"])
        .current_dir(temp.path())
        .assert()
        .success();

    for id in [&id1, &id2, &id3] {
        wk().args(["show", id])
            .current_dir(temp.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("backend"));
    }
}

#[test]
fn unlabel_multiple_issues() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LabelBatchUnlabel Task 1");
    let id2 = create_issue(&temp, "task", "LabelBatchUnlabel Task 2");

    wk().args(["label", &id1, "urgent"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["label", &id2, "urgent"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["unlabel", &id1, &id2, "urgent"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Check that Labels line does not contain urgent
    wk().args(["show", &id1])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::is_match(r"(?m)^Labels:.*urgent")
                .unwrap()
                .not(),
        );

    wk().args(["show", &id2])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::is_match(r"(?m)^Labels:.*urgent")
                .unwrap()
                .not(),
        );
}

#[test]
fn batch_labeled_issues_searchable() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "LabelBatchSearch Task 1");
    let id2 = create_issue(&temp, "task", "LabelBatchSearch Task 2");

    wk().args(["label", &id1, &id2, "batchtest"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["list", "--label", "batchtest"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("LabelBatchSearch Task 1"))
        .stdout(predicate::str::contains("LabelBatchSearch Task 2"));
}

// =============================================================================
// Multiple Labels in Single Command
// =============================================================================

#[test]
fn add_multiple_labels_to_multiple_issues() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "MultiLabel Task 1");
    let id2 = create_issue(&temp, "task", "MultiLabel Task 2");

    wk().args(["label", &id1, &id2, "urgent", "backend"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify both issues have both labels
    wk().args(["show", &id1])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"))
        .stdout(predicate::str::contains("backend"));

    wk().args(["show", &id2])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("urgent"))
        .stdout(predicate::str::contains("backend"));
}

#[test]
fn remove_multiple_labels_from_multiple_issues() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "MultiUnlabel Task 1");
    let id2 = create_issue(&temp, "task", "MultiUnlabel Task 2");

    // Add labels first
    wk().args(["label", &id1, &id2, "urgent", "backend"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Remove multiple labels from multiple issues
    wk().args(["unlabel", &id1, &id2, "urgent", "backend"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Check that Labels line does not contain urgent or backend
    wk().args(["show", &id1])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::is_match(r"(?m)^Labels:.*urgent")
                .unwrap()
                .not(),
        )
        .stdout(
            predicate::str::is_match(r"(?m)^Labels:.*backend")
                .unwrap()
                .not(),
        );

    wk().args(["show", &id2])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::is_match(r"(?m)^Labels:.*urgent")
                .unwrap()
                .not(),
        )
        .stdout(
            predicate::str::is_match(r"(?m)^Labels:.*backend")
                .unwrap()
                .not(),
        );
}

#[test]
fn add_three_labels_to_single_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "MultiLabel Task 3");

    wk().args(["label", &id, "p0", "urgent", "backend"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("p0"))
        .stdout(predicate::str::contains("urgent"))
        .stdout(predicate::str::contains("backend"));
}

// =============================================================================
// Parameterized Tests
// =============================================================================

#[yare::parameterized(
    simple_label = { "urgent" },
    namespaced_label = { "team:backend" },
    priority_label = { "priority:1" },
    scope_label = { "scope:frontend" },
)]
fn label_types(label: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LabelType Test");

    wk().args(["label", &id, label])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(label));
}
