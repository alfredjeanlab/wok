// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Test infrastructure for command testing without filesystem setup.
//!
//! This module provides a `TestContext` that wraps an in-memory database
//! and a default config, enabling commands to be tested without requiring
//! actual `.wok/` directory setup.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::commands::testing::TestContext;
//!
//! #[test]
//! fn test_some_command() {
//!     let mut ctx = TestContext::new();
//!     ctx.create_issue("test-1", IssueType::Task, "My task");
//!
//!     // Test command logic using ctx.db and ctx.config
//! }
//! ```

use crate::config::Config;
use crate::db::Database;
use crate::models::{Action, Event, Issue, IssueType, Relation, Status};
use chrono::Utc;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test context providing in-memory database and default config for testing.
pub struct TestContext {
    pub db: Database,
    pub config: Config,
    pub work_dir: PathBuf,
    _temp_dir: TempDir, // Keep alive for duration of test
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    /// Create a new test context with in-memory database and default config.
    pub fn new() -> Self {
        Self::with_prefix("test")
    }

    /// Create a new test context with a custom prefix.
    pub fn with_prefix(prefix: &str) -> Self {
        let db = Database::open_in_memory().expect("Failed to create in-memory database");
        let config = Config::new(prefix.to_string()).expect("Failed to create config");
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let work_dir = temp_dir.path().to_path_buf();
        TestContext {
            db,
            config,
            work_dir,
            _temp_dir: temp_dir,
        }
    }

    /// Create an issue with the given parameters.
    pub fn create_issue(&mut self, id: &str, issue_type: IssueType, title: &str) -> &mut Self {
        self.create_issue_with_status(id, issue_type, title, Status::Todo)
    }

    /// Create an issue with a specific status.
    pub fn create_issue_with_status(
        &mut self,
        id: &str,
        issue_type: IssueType,
        title: &str,
        status: Status,
    ) -> &mut Self {
        let now = Utc::now();
        let issue = Issue {
            id: id.to_string(),
            issue_type,
            title: title.to_string(),
            description: None,
            status,
            assignee: None,
            created_at: now,
            updated_at: now,
            closed_at: None,
            last_status_hlc: None,
            last_title_hlc: None,
            last_type_hlc: None,
            last_description_hlc: None,
            last_assignee_hlc: None,
        };
        self.db
            .create_issue(&issue)
            .expect("Failed to create issue");

        // Log creation event
        let event = Event::new(id.to_string(), Action::Created);
        self.db.log_event(&event).expect("Failed to log event");

        self
    }

    /// Add a label to an issue.
    pub fn add_label(&mut self, id: &str, label: &str) -> &mut Self {
        self.db.add_label(id, label).expect("Failed to add label");

        let event =
            Event::new(id.to_string(), Action::Labeled).with_values(None, Some(label.to_string()));
        self.db.log_event(&event).expect("Failed to log event");

        self
    }

    /// Add a note to an issue.
    pub fn add_note(&mut self, id: &str, content: &str) -> &mut Self {
        let issue = self.db.get_issue(id).expect("Issue not found");
        self.db
            .add_note(id, issue.status, content)
            .expect("Failed to add note");

        let event =
            Event::new(id.to_string(), Action::Noted).with_values(None, Some(content.to_string()));
        self.db.log_event(&event).expect("Failed to log event");

        self
    }

    /// Add a dependency between issues.
    pub fn add_dependency(&mut self, from: &str, to: &str, relation: Relation) -> &mut Self {
        self.db
            .add_dependency(from, to, relation)
            .expect("Failed to add dependency");
        self
    }

    /// Add a blocking relationship (from blocks to).
    pub fn blocks(&mut self, blocker: &str, blocked: &str) -> &mut Self {
        self.add_dependency(blocker, blocked, Relation::Blocks)
    }

    /// Add a tracks relationship.
    pub fn tracks(&mut self, tracker: &str, tracked: &str) -> &mut Self {
        self.add_dependency(tracker, tracked, Relation::Tracks);
        self.add_dependency(tracked, tracker, Relation::TrackedBy)
    }

    /// Update an issue's status.
    pub fn set_status(&mut self, id: &str, status: Status) -> &mut Self {
        let issue = self.db.get_issue(id).expect("Issue not found");
        let old_status = issue.status;

        self.db
            .update_issue_status(id, status)
            .expect("Failed to update status");

        let action = match status {
            Status::InProgress => Action::Started,
            Status::Todo => Action::Stopped,
            Status::Done => Action::Done,
            Status::Closed => Action::Closed,
        };
        let event = Event::new(id.to_string(), action)
            .with_values(Some(old_status.to_string()), Some(status.to_string()));
        self.db.log_event(&event).expect("Failed to log event");

        self
    }

    // Workflow helpers for common test scenarios

    /// Start working on an issue (todo -> in_progress).
    pub fn start_issue(&mut self, id: &str) -> &mut Self {
        self.set_status(id, Status::InProgress)
    }

    /// Stop working on an issue (in_progress -> todo).
    pub fn stop_issue(&mut self, id: &str) -> &mut Self {
        self.set_status(id, Status::Todo)
    }

    /// Complete an issue (any -> done).
    pub fn complete_issue(&mut self, id: &str) -> &mut Self {
        self.set_status(id, Status::Done)
    }

    /// Close an issue without completing (any -> closed).
    pub fn close_issue(&mut self, id: &str) -> &mut Self {
        self.set_status(id, Status::Closed)
    }

    /// Reopen an issue (done/closed -> in_progress).
    pub fn reopen_issue(&mut self, id: &str) -> &mut Self {
        self.set_status(id, Status::InProgress)
    }

    /// Create an issue and immediately start it.
    pub fn create_and_start(&mut self, id: &str, issue_type: IssueType, title: &str) -> &mut Self {
        self.create_issue(id, issue_type, title).start_issue(id)
    }

    /// Create an issue and mark it as done.
    pub fn create_completed(&mut self, id: &str, issue_type: IssueType, title: &str) -> &mut Self {
        self.create_issue(id, issue_type, title)
            .start_issue(id)
            .complete_issue(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = TestContext::new();
        assert_eq!(ctx.config.prefix, "test");
    }

    #[test]
    fn test_context_with_custom_prefix() {
        let ctx = TestContext::with_prefix("myproj");
        assert_eq!(ctx.config.prefix, "myproj");
    }

    #[test]
    fn test_create_issue() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.id, "test-1");
        assert_eq!(issue.issue_type, IssueType::Task);
        assert_eq!(issue.title, "My task");
        assert_eq!(issue.status, Status::Todo);
    }

    #[test]
    fn test_create_issue_with_status() {
        let mut ctx = TestContext::new();
        ctx.create_issue_with_status("test-1", IssueType::Bug, "Fix bug", Status::InProgress);

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::InProgress);
    }

    #[test]
    fn test_add_label() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .add_label("test-1", "urgent");

        let labels = ctx.db.get_labels("test-1").unwrap();
        assert_eq!(labels, vec!["urgent"]);
    }

    #[test]
    fn test_add_note() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .add_note("test-1", "This is a note");

        let notes = ctx.db.get_notes("test-1").unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].content, "This is a note");
    }

    #[test]
    fn test_blocking_dependency() {
        let mut ctx = TestContext::new();
        ctx.create_issue("blocker", IssueType::Task, "Blocker task")
            .create_issue("blocked", IssueType::Task, "Blocked task")
            .blocks("blocker", "blocked");

        let blocked_ids = ctx.db.get_blocked_issue_ids().unwrap();
        assert!(blocked_ids.contains(&"blocked".to_string()));
        assert!(!blocked_ids.contains(&"blocker".to_string()));
    }

    #[test]
    fn test_tracks_relationship() {
        let mut ctx = TestContext::new();
        ctx.create_issue("tracker", IssueType::Feature, "Tracker feature")
            .create_issue("tracked", IssueType::Task, "Tracked task")
            .tracks("tracker", "tracked");

        let tracked = ctx.db.get_tracked("tracker").unwrap();
        assert_eq!(tracked, vec!["tracked"]);

        let tracking = ctx.db.get_tracking("tracked").unwrap();
        assert_eq!(tracking, vec!["tracker"]);
    }

    #[test]
    fn test_set_status() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .set_status("test-1", Status::InProgress);

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::InProgress);
    }

    #[test]
    fn test_chained_operations() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "Task 1")
            .create_issue("test-2", IssueType::Task, "Task 2")
            .add_label("test-1", "priority")
            .add_label("test-1", "backend")
            .blocks("test-1", "test-2")
            .set_status("test-1", Status::InProgress);

        let labels = ctx.db.get_labels("test-1").unwrap();
        assert_eq!(labels.len(), 2);

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::InProgress);

        let blocked = ctx.db.get_blocked_issue_ids().unwrap();
        assert!(blocked.contains(&"test-2".to_string()));
    }

    // Workflow helper tests

    #[test]
    fn test_start_issue() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .start_issue("test-1");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::InProgress);
    }

    #[test]
    fn test_stop_issue() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .start_issue("test-1")
            .stop_issue("test-1");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::Todo);
    }

    #[test]
    fn test_complete_issue() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .start_issue("test-1")
            .complete_issue("test-1");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::Done);
    }

    #[test]
    fn test_close_issue() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .close_issue("test-1");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::Closed);
    }

    #[test]
    fn test_reopen_issue() {
        let mut ctx = TestContext::new();
        ctx.create_issue("test-1", IssueType::Task, "My task")
            .start_issue("test-1")
            .complete_issue("test-1")
            .reopen_issue("test-1");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.status, Status::InProgress);
    }

    #[test]
    fn test_create_and_start() {
        let mut ctx = TestContext::new();
        ctx.create_and_start("test-1", IssueType::Task, "Active task");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.title, "Active task");
        assert_eq!(issue.status, Status::InProgress);
    }

    #[test]
    fn test_create_completed() {
        let mut ctx = TestContext::new();
        ctx.create_completed("test-1", IssueType::Bug, "Fixed bug");

        let issue = ctx.db.get_issue("test-1").unwrap();
        assert_eq!(issue.title, "Fixed bug");
        assert_eq!(issue.status, Status::Done);
    }

    #[test]
    fn test_workflow_sequence() {
        // Test a realistic workflow: create -> start -> complete
        let mut ctx = TestContext::new();
        ctx.create_issue("feature-1", IssueType::Task, "Add feature")
            .add_label("feature-1", "enhancement")
            .add_note("feature-1", "Starting implementation")
            .start_issue("feature-1")
            .add_note("feature-1", "Work in progress")
            .complete_issue("feature-1")
            .add_note("feature-1", "Completed implementation");

        let issue = ctx.db.get_issue("feature-1").unwrap();
        assert_eq!(issue.status, Status::Done);

        let notes = ctx.db.get_notes("feature-1").unwrap();
        assert_eq!(notes.len(), 3);
    }
}
