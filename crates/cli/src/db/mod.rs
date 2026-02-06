// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! SQLite-backed database for issue storage.
//!
//! The [`Database`] struct wraps [`wk_core::Database`] and provides all data
//! access operations for issues, events, notes, labels, dependencies, and
//! external links. It converts between `wk_core::Issue` (with HLC fields)
//! and `wk_ipc::Issue` (with `closed_at`, no HLC) at the boundary.

use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;
use crate::models::{
    Dependency, Event, Issue, IssueType, Link, Note, PrefixInfo, Relation, Status,
};

/// SQLite database connection with issue tracker operations.
///
/// Wraps `wk_core::Database` and converts `wk_core::Issue` to `wk_ipc::Issue`
/// (which omits HLC fields and includes `closed_at`).
pub struct Database {
    inner: wk_core::Database,
}

impl Database {
    /// Open a database connection at the given path, creating and migrating if needed.
    pub fn open(path: &Path) -> Result<Self> {
        let inner = wk_core::Database::open(path)?;
        Ok(Database { inner })
    }

    /// Open an in-memory database (for testing and benchmarks).
    pub fn open_in_memory() -> Result<Self> {
        let inner = wk_core::Database::open_in_memory()?;
        Ok(Database { inner })
    }

    /// Access the underlying core database.
    pub fn core(&self) -> &wk_core::Database {
        &self.inner
    }

    // -- Issue operations (with type conversion) -------------------------------

    /// Create a new issue.
    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        let core_issue: wk_core::Issue = issue.clone().into();
        self.inner.create_issue(&core_issue)?;
        Ok(())
    }

    /// Get an issue by ID.
    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let core_issue = self.inner.get_issue(id)?;
        Ok(core_issue.into())
    }

    /// Check if an issue exists.
    pub fn issue_exists(&self, id: &str) -> Result<bool> {
        Ok(self.inner.issue_exists(id)?)
    }

    /// Resolve a potentially partial issue ID to a full ID.
    pub fn resolve_id(&self, partial_id: &str) -> Result<String> {
        Ok(self.inner.resolve_id(partial_id)?)
    }

    /// Update issue status.
    pub fn update_issue_status(&self, id: &str, status: Status) -> Result<()> {
        self.inner.update_issue_status(id, status)?;
        Ok(())
    }

    /// Update issue title.
    pub fn update_issue_title(&self, id: &str, title: &str) -> Result<()> {
        self.inner.update_issue_title(id, title)?;
        Ok(())
    }

    /// Update issue description.
    pub fn update_issue_description(&self, id: &str, description: &str) -> Result<()> {
        self.inner.update_issue_description(id, description)?;
        Ok(())
    }

    /// Update issue type.
    pub fn update_issue_type(&self, id: &str, issue_type: IssueType) -> Result<()> {
        self.inner.update_issue_type(id, issue_type)?;
        Ok(())
    }

    /// Set issue assignee.
    pub fn set_assignee(&self, id: &str, assignee: &str) -> Result<()> {
        self.inner.set_assignee(id, assignee)?;
        Ok(())
    }

    /// Clear issue assignee.
    pub fn clear_assignee(&self, id: &str) -> Result<()> {
        self.inner.clear_assignee(id)?;
        Ok(())
    }

    /// List issues with optional filters.
    pub fn list_issues(
        &self,
        status: Option<Status>,
        issue_type: Option<IssueType>,
        label: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let core_issues = self.inner.list_issues(status, issue_type, label)?;
        Ok(core_issues.into_iter().map(Into::into).collect())
    }

    /// Search issues by query string.
    pub fn search_issues(&self, query: &str) -> Result<Vec<Issue>> {
        let core_issues = self.inner.search_issues(query)?;
        Ok(core_issues.into_iter().map(Into::into).collect())
    }

    /// Get all issues.
    pub fn get_all_issues(&self) -> Result<Vec<Issue>> {
        self.list_issues(None, None, None)
    }

    /// Get IDs of blocked issues.
    pub fn get_blocked_issue_ids(&self) -> Result<Vec<String>> {
        Ok(self.inner.get_blocked_issue_ids()?)
    }

    /// Extract priority from label list.
    pub fn priority_from_tags(tags: &[String]) -> u8 {
        wk_core::Database::priority_from_tags(tags)
    }

    // -- Event operations ------------------------------------------------------

    /// Log an event.
    pub fn log_event(&self, event: &Event) -> Result<i64> {
        Ok(self.inner.log_event(event)?)
    }

    /// Get all events for an issue.
    pub fn get_events(&self, issue_id: &str) -> Result<Vec<Event>> {
        Ok(self.inner.get_events(issue_id)?)
    }

    /// Get recent events across all issues.
    pub fn get_recent_events(&self, limit: usize) -> Result<Vec<Event>> {
        Ok(self.inner.get_recent_events(limit)?)
    }

    // -- Note operations -------------------------------------------------------

    /// Add a note to an issue.
    pub fn add_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
        Ok(self.inner.add_note(issue_id, status, content)?)
    }

    /// Get all notes for an issue.
    pub fn get_notes(&self, issue_id: &str) -> Result<Vec<Note>> {
        Ok(self.inner.get_notes(issue_id)?)
    }

    /// Replace the most recent note for an issue.
    pub fn replace_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
        Ok(self.inner.replace_note(issue_id, status, content)?)
    }

    /// Get notes grouped by status.
    pub fn get_notes_by_status(&self, issue_id: &str) -> Result<Vec<(Status, Vec<Note>)>> {
        Ok(self.inner.get_notes_by_status(issue_id)?)
    }

    // -- Label operations ------------------------------------------------------

    /// Add a label to an issue.
    pub fn add_label(&self, issue_id: &str, label: &str) -> Result<()> {
        self.inner.add_label(issue_id, label)?;
        Ok(())
    }

    /// Remove a label from an issue.
    pub fn remove_label(&self, issue_id: &str, label: &str) -> Result<bool> {
        Ok(self.inner.remove_label(issue_id, label)?)
    }

    /// Get all labels for an issue.
    pub fn get_labels(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.inner.get_labels(issue_id)?)
    }

    /// Get labels for multiple issues.
    pub fn get_labels_batch(&self, issue_ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
        Ok(self.inner.get_labels_batch(issue_ids)?)
    }

    // -- Dependency operations -------------------------------------------------

    /// Add a dependency between two issues.
    pub fn add_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        self.inner.add_dependency(from_id, to_id, relation)?;
        Ok(())
    }

    /// Remove a dependency between two issues.
    pub fn remove_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        self.inner.remove_dependency(from_id, to_id, relation)?;
        Ok(())
    }

    /// Get all dependencies from an issue.
    pub fn get_deps_from(&self, from_id: &str) -> Result<Vec<Dependency>> {
        Ok(self.inner.get_deps_from(from_id)?)
    }

    /// Get issues that directly block the given issue.
    pub fn get_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.inner.get_blockers(issue_id)?)
    }

    /// Get all issues that transitively block the given issue.
    pub fn get_transitive_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.inner.get_transitive_blockers(issue_id)?)
    }

    /// Get issues that this issue blocks.
    pub fn get_blocking(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.inner.get_blocking(issue_id)?)
    }

    /// Get tracking issues.
    pub fn get_tracking(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.inner.get_tracking(issue_id)?)
    }

    /// Get tracked issues.
    pub fn get_tracked(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.inner.get_tracked(issue_id)?)
    }

    // -- Link operations -------------------------------------------------------

    /// Add an external link to an issue.
    pub fn add_link(&self, link: &Link) -> Result<i64> {
        Ok(self.inner.add_link(link)?)
    }

    /// Get all external links for an issue.
    pub fn get_links(&self, issue_id: &str) -> Result<Vec<Link>> {
        Ok(self.inner.get_links(issue_id)?)
    }

    /// Get a specific link by issue ID and URL.
    pub fn get_link_by_url(&self, issue_id: &str, url: &str) -> Result<Option<Link>> {
        Ok(self.inner.get_link_by_url(issue_id, url)?)
    }

    /// Remove an external link by its ID.
    pub fn remove_link(&self, link_id: i64) -> Result<()> {
        self.inner.remove_link(link_id)?;
        Ok(())
    }

    /// Remove all links for an issue.
    pub fn remove_all_links(&self, issue_id: &str) -> Result<()> {
        self.inner.remove_all_links(issue_id)?;
        Ok(())
    }

    // -- Prefix operations -----------------------------------------------------

    /// Ensure a prefix exists in the prefixes table.
    pub fn ensure_prefix(&self, prefix: &str) -> Result<()> {
        self.inner.ensure_prefix(prefix)?;
        Ok(())
    }

    /// Increment the issue count for a prefix.
    pub fn increment_prefix_count(&self, prefix: &str) -> Result<()> {
        self.inner.increment_prefix_count(prefix)?;
        Ok(())
    }

    /// Decrement the issue count for a prefix.
    pub fn decrement_prefix_count(&self, prefix: &str) -> Result<()> {
        self.inner.decrement_prefix_count(prefix)?;
        Ok(())
    }

    /// List all prefixes with their issue counts.
    pub fn list_prefixes(&self) -> Result<Vec<PrefixInfo>> {
        Ok(self.inner.list_prefixes()?)
    }

    /// Rename a prefix.
    pub fn rename_prefix(&self, old: &str, new: &str) -> Result<()> {
        self.inner.rename_prefix(old, new)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
