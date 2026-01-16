// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::commands::testing::TestContext;
use crate::config::Config;
use crate::error::Error;
use crate::models::IssueType;
use tempfile::TempDir;

// Helper to create a test context and temp directory for config
struct ConfigTestContext {
    ctx: TestContext,
    temp_dir: TempDir,
}

impl ConfigTestContext {
    fn new(prefix: &str) -> Self {
        let ctx = TestContext::with_prefix(prefix);
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Save initial config
        ctx.config.save(temp_dir.path()).unwrap();

        ConfigTestContext { ctx, temp_dir }
    }

    fn work_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

// === Prefix Validation Tests ===

#[test]
fn test_rename_prefix_validates_new_prefix() {
    let test_ctx = ConfigTestContext::new("old");

    // Invalid: too short
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "a",
    );
    assert!(matches!(result, Err(Error::InvalidPrefix)));

    // Invalid: uppercase
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "ABC",
    );
    assert!(matches!(result, Err(Error::InvalidPrefix)));

    // Invalid: pure numbers
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "123",
    );
    assert!(matches!(result, Err(Error::InvalidPrefix)));

    // Invalid: contains dash
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "my-proj",
    );
    assert!(matches!(result, Err(Error::InvalidPrefix)));
}

#[test]
fn test_rename_prefix_validates_old_prefix() {
    let test_ctx = ConfigTestContext::new("old");

    // Invalid old prefix: too short
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "a",
        "new",
    );
    assert!(matches!(result, Err(Error::InvalidPrefix)));

    // Invalid old prefix: uppercase
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "ABC",
        "new",
    );
    assert!(matches!(result, Err(Error::InvalidPrefix)));
}

#[test]
fn test_rename_prefix_same_prefix_noop() {
    let test_ctx = ConfigTestContext::new("same");
    test_ctx
        .ctx
        .create_issue("same-a1b2", IssueType::Task, "Test issue");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "same",
        "same",
    );

    assert!(result.is_ok());
    // Issue should remain unchanged
    assert!(test_ctx.ctx.db.get_issue("same-a1b2").is_ok());
}

// === Issue ID Migration Tests ===

#[test]
fn test_rename_prefix_updates_issue_ids() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Issue 1");
    test_ctx
        .ctx
        .create_issue("old-c3d4", IssueType::Bug, "Issue 2");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    // Old IDs should not exist
    assert!(test_ctx.ctx.db.get_issue("old-a1b2").is_err());
    assert!(test_ctx.ctx.db.get_issue("old-c3d4").is_err());

    // New IDs should exist
    let issue1 = test_ctx.ctx.db.get_issue("new-a1b2").unwrap();
    assert_eq!(issue1.title, "Issue 1");

    let issue2 = test_ctx.ctx.db.get_issue("new-c3d4").unwrap();
    assert_eq!(issue2.title, "Issue 2");
}

#[test]
fn test_rename_prefix_updates_dependencies() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Blocker")
        .create_issue("old-c3d4", IssueType::Task, "Blocked")
        .blocks("old-a1b2", "old-c3d4");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    // Check that dependencies are updated
    let blockers = test_ctx.ctx.db.get_blockers("new-c3d4").unwrap();
    assert_eq!(blockers, vec!["new-a1b2"]);

    let blocking = test_ctx.ctx.db.get_blocking("new-a1b2").unwrap();
    assert_eq!(blocking, vec!["new-c3d4"]);
}

#[test]
fn test_rename_prefix_updates_labels() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Test issue")
        .add_label("old-a1b2", "urgent")
        .add_label("old-a1b2", "backend");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    let labels = test_ctx.ctx.db.get_labels("new-a1b2").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"backend".to_string()));
}

#[test]
fn test_rename_prefix_updates_notes() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Test issue")
        .add_note("old-a1b2", "This is a note");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    let notes = test_ctx.ctx.db.get_notes("new-a1b2").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "This is a note");
}

#[test]
fn test_rename_prefix_updates_events() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Test issue")
        .start_issue("old-a1b2");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    let events = test_ctx.ctx.db.get_events("new-a1b2").unwrap();
    // Should have creation and start events
    assert!(events.len() >= 2);
}

#[test]
fn test_rename_prefix_updates_config_file() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Test issue");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    // Reload config and check prefix
    let updated_config = Config::load(test_ctx.work_dir()).unwrap();
    assert_eq!(updated_config.prefix, "new");
}

#[test]
fn test_rename_prefix_does_not_update_config_for_different_prefix() {
    let test_ctx = ConfigTestContext::new("current");
    test_ctx
        .ctx
        .create_issue("other-a1b2", IssueType::Task, "Other prefix issue");

    // Rename "other" to "new" but config has "current" prefix
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "other",
        "new",
    );
    assert!(result.is_ok());

    // Config should remain unchanged (still "current")
    let updated_config = Config::load(test_ctx.work_dir()).unwrap();
    assert_eq!(updated_config.prefix, "current");

    // But the issue should be renamed
    assert!(test_ctx.ctx.db.get_issue("new-a1b2").is_ok());
}

#[test]
fn test_rename_prefix_handles_mixed_prefixes() {
    // Test that only matching prefixes are updated
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Old prefix issue");
    // Manually insert an issue with a different prefix
    test_ctx
        .ctx
        .create_issue("other-c3d4", IssueType::Task, "Other prefix issue");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    // Old prefix should be renamed
    assert!(test_ctx.ctx.db.get_issue("old-a1b2").is_err());
    assert!(test_ctx.ctx.db.get_issue("new-a1b2").is_ok());

    // Other prefix should remain unchanged
    assert!(test_ctx.ctx.db.get_issue("other-c3d4").is_ok());
}

#[test]
fn test_rename_prefix_with_tracks_relationship() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-feat", IssueType::Feature, "Feature")
        .create_issue("old-task", IssueType::Task, "Task")
        .tracks("old-feat", "old-task");

    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    // Check both directions of tracks/tracked-by
    let tracked = test_ctx.ctx.db.get_tracked("new-feat").unwrap();
    assert_eq!(tracked, vec!["new-task"]);

    let tracking = test_ctx.ctx.db.get_tracking("new-task").unwrap();
    assert_eq!(tracking, vec!["new-feat"]);
}

#[test]
fn test_rename_prefix_empty_database() {
    let test_ctx = ConfigTestContext::new("old");

    // Rename with no issues should succeed
    let result = run_rename_prefix(
        &test_ctx.ctx.db,
        &test_ctx.ctx.config,
        test_ctx.work_dir(),
        "old",
        "new",
    );
    assert!(result.is_ok());

    // Config should still be updated
    let updated_config = Config::load(test_ctx.work_dir()).unwrap();
    assert_eq!(updated_config.prefix, "new");
}

// === Direct rename_all_issue_ids Tests ===

#[test]
fn test_rename_all_issue_ids_transaction() {
    let test_ctx = ConfigTestContext::new("old");
    test_ctx
        .ctx
        .create_issue("old-a1b2", IssueType::Task, "Issue 1")
        .create_issue("old-c3d4", IssueType::Task, "Issue 2")
        .blocks("old-a1b2", "old-c3d4")
        .add_label("old-a1b2", "test")
        .add_note("old-c3d4", "note");

    let result = rename_all_issue_ids(&test_ctx.ctx.db, "old", "new");
    assert!(result.is_ok());

    // Verify all tables were updated
    assert!(test_ctx.ctx.db.get_issue("new-a1b2").is_ok());
    assert!(test_ctx.ctx.db.get_issue("new-c3d4").is_ok());

    let blockers = test_ctx.ctx.db.get_blockers("new-c3d4").unwrap();
    assert_eq!(blockers, vec!["new-a1b2"]);

    let labels = test_ctx.ctx.db.get_labels("new-a1b2").unwrap();
    assert!(labels.contains(&"test".to_string()));

    let notes = test_ctx.ctx.db.get_notes("new-c3d4").unwrap();
    assert!(!notes.is_empty());
}

#[test]
fn test_rename_prefix_valid_prefixes() {
    // Test various valid prefixes
    let valid_prefixes = vec!["ab", "abc", "project", "v0", "proj2", "myproject123"];

    for new_prefix in valid_prefixes {
        let test_ctx = ConfigTestContext::new("old");
        test_ctx
            .ctx
            .create_issue("old-a1b2", IssueType::Task, "Test");

        let result = run_rename_prefix(
            &test_ctx.ctx.db,
            &test_ctx.ctx.config,
            test_ctx.work_dir(),
            "old",
            new_prefix,
        );
        assert!(
            result.is_ok(),
            "Prefix '{}' should be valid but got error",
            new_prefix
        );

        let expected_id = format!("{}-a1b2", new_prefix);
        assert!(
            test_ctx.ctx.db.get_issue(&expected_id).is_ok(),
            "Issue should have new ID: {}",
            expected_id
        );
    }
}
