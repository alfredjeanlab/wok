// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk link` and `wk unlink` commands.
//! Converted from tests/specs/cli/unit/link.bats

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

// =============================================================================
// Provider Detection Tests
// =============================================================================

#[parameterized(
    github = {
        "https://github.com/org/repo/issues/123",
        "[github]",
        "github.com"
    },
    jira_atlassian = {
        "https://company.atlassian.net/browse/PE-5555",
        "[jira]",
        "PE-5555"
    },
    gitlab = {
        "https://gitlab.com/org/project/issues/456",
        "[gitlab]",
        "gitlab.com"
    },
    jira_shorthand = {
        "jira://PE-5555",
        "PE-5555",
        "PE-5555"
    },
    unknown = {
        "https://example.com/issue/123",
        "example.com",
        "example.com"
    },
)]
fn link_detects_provider_type_from_url(url: &str, expect1: &str, expect2: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkDetect Test");

    wk().args(["link", &id, url])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expect1))
        .stdout(predicate::str::contains(expect2));
}

#[test]
fn link_detects_confluence_not_jira() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkDetect Test confluence");

    wk().args([
        "link",
        &id,
        "https://company.atlassian.net/wiki/spaces/DOC/pages/123",
    ])
    .current_dir(temp.path())
    .assert()
    .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[confluence]"))
        .stdout(predicate::str::contains("[jira]").not());
}

// =============================================================================
// Reason Flag Tests
// =============================================================================

#[parameterized(
    tracks_long = { "--reason", "tracks", "(tracks)" },
    blocks_long = { "--reason", "blocks", "(blocks)" },
    tracks_short = { "-r", "tracks", "(tracks)" },
)]
fn link_reason_flags(flag: &str, value: &str, expected: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkReason Test");

    wk().args([
        "link",
        &id,
        "https://github.com/org/repo/issues/123",
        flag,
        value,
    ])
    .current_dir(temp.path())
    .assert()
    .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

// =============================================================================
// Import Validation Tests
// =============================================================================

#[test]
fn link_import_requires_known_provider() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkImport Test unknown");

    wk().args([
        "link",
        &id,
        "https://example.com/issue/123",
        "--reason",
        "import",
    ])
    .current_dir(temp.path())
    .assert()
    .failure()
    .stderr(predicate::str::contains("requires a known provider"));
}

#[test]
fn link_import_requires_detectable_id() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkImport Test no id");

    wk().args([
        "link",
        &id,
        "https://company.atlassian.net/wiki/spaces/DOC",
        "--reason",
        "import",
    ])
    .current_dir(temp.path())
    .assert()
    .failure()
    .stderr(predicate::str::contains("requires a detectable issue ID"));
}

#[parameterized(
    github = { "https://github.com/org/repo/issues/456" },
    jira = { "https://company.atlassian.net/browse/PE-1234" },
)]
fn link_import_succeeds_with_known_provider(url: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkImport Test");

    wk().args(["link", &id, url, "--reason", "import"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("(import)"));
}

// =============================================================================
// Show Links Section Tests
// =============================================================================

#[test]
fn show_displays_links_section_when_non_empty() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkShow Test task");

    wk().args(["link", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:"));
}

#[test]
fn show_hides_links_section_when_empty() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkShow Test empty");

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:").not());
}

#[test]
fn show_displays_multiple_links() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkShow Test multiple");

    wk().args(["link", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["link", &id, "jira://PE-5555"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("github.com"))
        .stdout(predicate::str::contains("PE-5555"));
}

// =============================================================================
// New --link Option Tests
// =============================================================================

#[test]
fn new_link_option_adds_link() {
    let temp = init_temp();

    let output = wk()
        .args([
            "new",
            "task",
            "LinkNew Linked task",
            "--link",
            "https://github.com/org/repo/issues/456",
            "-o",
            "id",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:"))
        .stdout(predicate::str::contains("github.com"));
}

#[test]
fn new_link_shorthand_flag() {
    let temp = init_temp();

    wk().args([
        "new",
        "task",
        "LinkNew Linked shorthand",
        "-l",
        "https://github.com/org/repo/issues/789",
    ])
    .current_dir(temp.path())
    .assert()
    .success();
}

#[test]
fn new_multiple_links() {
    let temp = init_temp();

    let output = wk()
        .args([
            "new",
            "task",
            "LinkNew Multi-linked",
            "--link",
            "https://github.com/org/repo/issues/1",
            "--link",
            "jira://PE-1234",
            "-o",
            "id",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("github.com"))
        .stdout(predicate::str::contains("PE-1234"));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn link_nonexistent_issue_fails() {
    let temp = init_temp();

    wk().args([
        "link",
        "test-nonexistent",
        "https://github.com/org/repo/issues/123",
    ])
    .current_dir(temp.path())
    .assert()
    .failure();
}

#[test]
fn link_invalid_reason_fails() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkError Test task");

    wk().args([
        "link",
        &id,
        "https://github.com/org/repo/issues/123",
        "--reason",
        "invalid",
    ])
    .current_dir(temp.path())
    .assert()
    .failure();
}

// =============================================================================
// JSON Output and Log Tests
// =============================================================================

#[test]
fn show_json_includes_links() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkJSON Test task");

    wk().args(["link", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id, "--output", "json"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"links\":"))
        .stdout(predicate::str::contains("\"link_type\":\"github\""));
}

#[test]
fn log_shows_linked_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "LinkLog Test task");

    wk().args(["link", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("linked"));
}

// =============================================================================
// Unlink Tests
// =============================================================================

#[test]
fn unlink_removes_link_from_issue() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Unlink Test");

    wk().args(["link", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:"));

    wk().args(["unlink", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed link"));

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:").not());
}

#[test]
fn unlink_nonexistent_url_succeeds_with_message() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Unlink Nonexistent Test");

    wk().args(["unlink", &id, "https://example.com/not-linked"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

#[test]
fn unlink_nonexistent_issue_fails() {
    let temp = init_temp();

    wk().args([
        "unlink",
        "test-nonexistent",
        "https://github.com/org/repo/issues/123",
    ])
    .current_dir(temp.path())
    .assert()
    .failure();
}

#[test]
fn unlink_removes_only_specified_link() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Unlink Multiple Test");

    wk().args(["link", &id, "https://github.com/org/repo/issues/1"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["link", &id, "https://github.com/org/repo/issues/2"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["unlink", &id, "https://github.com/org/repo/issues/1"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify only issues/2 remains in links (not issues/1)
    let output = wk()
        .args(["show", &id, "--output", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"url\":\"https://github.com/org/repo/issues/2\""));
    assert!(!stdout.contains("\"url\":\"https://github.com/org/repo/issues/1\""));
}

#[test]
fn log_shows_unlinked_event() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Unlink Log Test");

    wk().args(["link", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["unlink", &id, "https://github.com/org/repo/issues/123"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["log", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unlinked"));
}
