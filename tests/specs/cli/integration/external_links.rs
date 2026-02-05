// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for external links integration tests.
//! Converted from tests/specs/cli/integration/external_links.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use yare::parameterized;

fn wk() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("wok").unwrap()
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

// =============================================================================
// Export/import roundtrip
// =============================================================================

#[test]
fn export_includes_links() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Task with link");

    wk().arg("link")
        .arg(&id)
        .arg("https://github.com/org/repo/issues/789")
        .current_dir(temp.path())
        .assert()
        .success();

    let export_path = temp.path().join("export.jsonl");
    wk().arg("export")
        .arg(&export_path)
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("links"), "Export should contain links");
    assert!(
        content.contains("github.com"),
        "Export should contain github.com"
    );
}

#[test]
fn export_includes_link_metadata() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Task with metadata");

    wk().arg("link")
        .arg(&id)
        .arg("https://github.com/org/repo/issues/123")
        .arg("--reason")
        .arg("tracks")
        .current_dir(temp.path())
        .assert()
        .success();

    let export_path = temp.path().join("export2.jsonl");
    wk().arg("export")
        .arg(&export_path)
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(&export_path).unwrap();
    // Find the line containing our issue ID
    let issue_line = content
        .lines()
        .find(|line| line.contains(&id))
        .expect("Export should contain issue");
    assert!(
        issue_line.contains(r#""link_type":"github""#),
        "Export should contain link_type github"
    );
    assert!(
        issue_line.contains(r#""rel":"tracks""#),
        "Export should contain rel tracks"
    );
}

#[test]
fn import_wk_format_with_links() {
    let temp = init_temp();

    // Create and export issue with links
    let id = create_issue(&temp, "task", "Exportable task");
    wk().arg("link")
        .arg(&id)
        .arg("jira://TEST-123")
        .current_dir(temp.path())
        .assert()
        .success();

    let export_path = temp.path().join("with_links.jsonl");
    wk().arg("export")
        .arg(&export_path)
        .current_dir(temp.path())
        .assert()
        .success();

    // Import to a new project in a subdirectory
    let import_dir = temp.path().join("import_test");
    fs::create_dir(&import_dir).unwrap();

    wk().arg("init")
        .arg("--prefix")
        .arg("tst")
        .arg("--private")
        .current_dir(&import_dir)
        .assert()
        .success();

    wk().arg("import")
        .arg(&export_path)
        .current_dir(&import_dir)
        .assert()
        .success();

    // Verify link was preserved
    wk().arg("show")
        .arg(&id)
        .current_dir(&import_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:"))
        .stdout(predicate::str::contains("TEST-123"));
}

#[test]
fn import_bd_format_does_not_include_links() {
    let temp = init_temp();

    // Create bd-format JSONL (beads format doesn't have external links)
    let bd_file = temp.path().join("bd_issues.jsonl");
    fs::write(
        &bd_file,
        r#"{"id":"bd-1234","title":"Imported from bd","status":"open","issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
    )
    .unwrap();

    wk().arg("import")
        .arg("--format")
        .arg("bd")
        .arg(&bd_file)
        .current_dir(temp.path())
        .assert()
        .success();

    // No links section since beads format doesn't support them
    wk().arg("show")
        .arg("bd-1234")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:").not());
}

// =============================================================================
// Event naming verification
// =============================================================================

#[test]
fn dep_events_use_related_not_linked() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Blocker");
    let id2 = create_issue(&temp, "task", "Blocked");

    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("log")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("related"))
        .stdout(predicate::str::contains("linked").not());
}

#[test]
fn undep_events_use_unrelated() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "Blocker");
    let id2 = create_issue(&temp, "task", "Blocked");

    wk().arg("dep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("undep")
        .arg(&id1)
        .arg("blocks")
        .arg(&id2)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("log")
        .arg(&id1)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unrelated"))
        .stdout(predicate::str::contains("unlinked").not());
}

#[test]
fn link_events_use_linked() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Task");

    wk().arg("link")
        .arg(&id)
        .arg("https://github.com/org/repo/issues/1")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("log")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("linked"));
}

// =============================================================================
// Multiple links on single issue
// =============================================================================

#[test]
fn issue_can_have_multiple_links() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Multi-linked issue");

    wk().arg("link")
        .arg(&id)
        .arg("https://github.com/org/repo/issues/1")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("link")
        .arg(&id)
        .arg("https://github.com/org/repo/issues/2")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("link")
        .arg(&id)
        .arg("jira://PE-1234")
        .current_dir(temp.path())
        .assert()
        .success();

    // Should show all three links
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Links:"))
        .stdout(predicate::str::contains("issues/1"))
        .stdout(predicate::str::contains("issues/2"))
        .stdout(predicate::str::contains("PE-1234"));
}

// =============================================================================
// Links with combined features
// =============================================================================

#[test]
fn new_with_link_and_label_creates_both() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Labeled linked task")
        .arg("--label")
        .arg("important")
        .arg("--link")
        .arg("https://github.com/org/repo/issues/100")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Labels: important"))
        .stdout(predicate::str::contains("Links:"))
        .stdout(predicate::str::contains("github.com"));
}

#[test]
fn new_with_link_and_note_creates_both() {
    let temp = init_temp();

    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Noted linked task")
        .arg("--note")
        .arg("Initial note")
        .arg("--link")
        .arg("https://github.com/org/repo/issues/200")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Description:"))
        .stdout(predicate::str::contains("Initial note"))
        .stdout(predicate::str::contains("Links:"));
}

// =============================================================================
// Confluence vs Jira detection
// =============================================================================

#[parameterized(
    confluence_wiki_url = { "https://company.atlassian.net/wiki/spaces/TEAM/pages/456789", "[confluence]", "[jira]" },
    jira_browse_url = { "https://company.atlassian.net/browse/PROJ-123", "[jira]", "[confluence]" },
)]
fn link_type_detection(url: &str, expected: &str, not_expected: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Link type test");

    wk().arg("link")
        .arg(&id)
        .arg(url)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected))
        .stdout(predicate::str::contains(not_expected).not());
}

#[test]
fn jira_browse_url_shows_issue_key() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Jira test");

    wk().arg("link")
        .arg(&id)
        .arg("https://company.atlassian.net/browse/PROJ-123")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PROJ-123"));
}
