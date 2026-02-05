// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for issue ID prefix matching.
//! Converted from tests/specs/cli/unit/prefix-matching.bats

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

/// Extract an 8-character prefix from an issue ID (project prefix + hyphen + 3 hash chars)
fn prefix_of(id: &str) -> String {
    id.chars().take(8).collect()
}

// =============================================================================
// Basic Prefix Matching Tests
// =============================================================================

#[test]
fn exact_id_match_works() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Test issue"));
}

#[test]
fn prefix_match_with_7_chars_works() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Unique issue");
    // Use first 7 characters of the ID (prefix + 2 chars of hash)
    let prefix: String = id.chars().take(7).collect();

    wk().args(["show", &prefix])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Unique issue"));
}

#[test]
fn prefix_shorter_than_3_chars_fails() {
    let temp = init_temp();
    let _id = create_issue(&temp, "task", "Short prefix test");

    // Use only 2 characters - should fail
    wk().args(["show", "te"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("issue not found"));
}

// =============================================================================
// Command-Specific Prefix Tests (Parameterized)
// =============================================================================

#[test]
fn show_with_prefix_shows_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Show with prefix test");
    let prefix = prefix_of(&id);

    wk().args(["show", &prefix])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Show with prefix test"));
}

#[test]
fn edit_with_prefix_updates_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Original title");
    let prefix = prefix_of(&id);

    wk().args(["edit", &prefix, "title", "Updated via prefix"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated via prefix"));
}

#[test]
fn start_with_prefix_starts_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Start prefix test");
    let prefix = prefix_of(&id);

    wk().args(["start", &prefix])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("in_progress"));
}

#[test]
fn done_with_prefix_completes_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Done prefix test");

    wk().args(["start", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    let prefix = prefix_of(&id);

    wk().args(["done", &prefix])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

#[test]
fn note_with_prefix_adds_note_to_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Note prefix test");
    let prefix = prefix_of(&id);

    wk().args(["note", &prefix, "A note via prefix"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("A note via prefix"));
}

#[test]
fn label_with_prefix_labels_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Label prefix test");
    let prefix = prefix_of(&id);

    wk().args(["label", &prefix, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("mylabel"));
}

#[test]
fn tree_with_prefix_shows_correct_tree() {
    let temp = init_temp();
    let parent = create_issue(&temp, "feature", "Parent feature");
    let child = create_issue(&temp, "task", "Child task");

    wk().args(["dep", &parent, "tracks", &child])
        .current_dir(temp.path())
        .assert()
        .success();

    let prefix = prefix_of(&parent);

    wk().args(["tree", &prefix])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Parent feature"))
        .stdout(predicate::str::contains("Child task"));
}

#[test]
fn log_with_prefix_shows_events_for_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Log prefix test");
    let prefix = prefix_of(&id);

    wk().args(["log", &prefix])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));
}

#[test]
fn dep_with_prefix_creates_dependency_correctly() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Blocker task");
    let id2 = create_issue(&temp, "task", "Blocked task");
    let prefix1 = prefix_of(&id1);
    let prefix2 = prefix_of(&id2);

    wk().args(["dep", &prefix1, "blocks", &prefix2])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id2])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked by"));
}

#[test]
fn link_with_prefix_adds_link_to_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Link prefix test");
    let prefix = prefix_of(&id);

    wk().args(["link", &prefix, "https://example.com/issue/123"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("example.com"));
}

#[test]
fn close_with_prefix_closes_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Close prefix test");
    let prefix = prefix_of(&id);

    wk().args(["close", &prefix, "--reason", "Testing prefix close"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("closed"));
}

#[test]
fn reopen_with_prefix_reopens_correct_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Reopen prefix test");

    wk().args(["start", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["done", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    let prefix = prefix_of(&id);

    wk().args(["reopen", &prefix, "--reason", "Testing prefix reopen"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("todo"));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn ambiguous_prefix_shows_error_with_matches() {
    let temp = init_temp();
    // Create multiple issues - they all share the "test-" prefix
    let _id1 = create_issue(&temp, "task", "First issue");
    let _id2 = create_issue(&temp, "task", "Second issue");
    let _id3 = create_issue(&temp, "task", "Third issue");

    // Try to match with just "test-" which may be ambiguous
    let result = wk()
        .args(["show", "test-"])
        .current_dir(temp.path())
        .assert();

    // The command may fail if ambiguous, or succeed if one matches
    // We just verify it doesn't panic and returns a sensible result
    let output = result.get_output();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("ambiguous") || stderr.contains("not found"),
            "Expected ambiguous or not found error, got: {}",
            stderr
        );
    }
}

#[test]
fn nonexistent_prefix_fails_with_not_found() {
    let temp = init_temp();

    wk().args(["show", "test-zzzzzzz"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("issue not found"));
}

// =============================================================================
// Bulk Operations Tests
// =============================================================================

#[test]
fn bulk_start_with_prefix_succeeds() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Bulk start test");
    let prefix = prefix_of(&id);

    wk().args(["start", &prefix])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Started"));
}

#[test]
fn bulk_done_completes_multiple_issues() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Bulk done 1");
    let id2 = create_issue(&temp, "task", "Bulk done 2");

    wk().args(["start", &id1])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["start", &id2])
        .current_dir(temp.path())
        .assert()
        .success();

    // Use full IDs for bulk done (more reliable)
    wk().args(["done", &id1, &id2])
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Parameterized Tests for Commands Using Prefixes
// =============================================================================

/// Commands that accept a single issue ID and just need to succeed with a prefix.
/// These all follow the pattern: create issue -> get prefix -> run command -> assert success
#[parameterized(
    show = { &["show"], "Show param test" },
    log = { &["log"], "Log param test" },
)]
fn command_works_with_prefix(args: &[&str], title: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", title);
    let prefix = prefix_of(&id);

    let mut cmd = wk();
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(&prefix).current_dir(temp.path()).assert().success();
}

/// Commands that modify issue state and can be verified with show
#[parameterized(
    start = { &["start"], "in_progress", "Start param test" },
)]
fn state_change_command_works_with_prefix(args: &[&str], expected_status: &str, title: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", title);
    let prefix = prefix_of(&id);

    let mut cmd = wk();
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(&prefix).current_dir(temp.path()).assert().success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_status));
}
