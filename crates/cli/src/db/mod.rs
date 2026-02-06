// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! SQLite-backed database for issue storage.
//!
//! The [`Database`] struct wraps [`wk_core::Database`] and provides all data access
//! operations for the CLI. Most operations delegate directly via `Deref`;
//! methods returning or accepting `Issue` convert between `wk_core::Issue`
//! (with HLC fields) and `wk_ipc::Issue` (without HLC, used by the CLI).

use std::path::Path;

use crate::error::Result;
use crate::models::{Issue, IssueType, Status};

// Re-export core db utilities for tests and internal use.
pub use wk_core::db::{parse_db, parse_timestamp};

/// SQLite database connection with issue tracker operations.
///
/// Wraps [`wk_core::Database`]. Non-Issue operations pass through via `Deref`;
/// Issue operations convert between core and IPC types at the boundary.
pub struct Database(pub wk_core::Database);

impl Database {
    /// Open a database connection at the given path, creating and migrating if needed.
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Database(wk_core::Database::open(path)?))
    }

    /// Open an in-memory database (for testing and benchmarks).
    pub fn open_in_memory() -> Result<Self> {
        Ok(Database(wk_core::Database::open_in_memory()?))
    }

    /// Access the underlying core database.
    pub fn core(&self) -> &wk_core::Database {
        &self.0
    }

    // -- Issue operations (with type conversion) ------------------------------

    /// Create a new issue.
    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        let core_issue: wk_core::Issue = issue.clone().into();
        self.0.create_issue(&core_issue)?;
        Ok(())
    }

    /// Get an issue by ID.
    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let core_issue = self.0.get_issue(id)?;
        Ok(core_issue.into())
    }

    /// List issues with optional filters.
    pub fn list_issues(
        &self,
        status: Option<Status>,
        issue_type: Option<IssueType>,
        label: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let core_issues = self.0.list_issues(status, issue_type, label)?;
        Ok(core_issues.into_iter().map(Into::into).collect())
    }

    /// Search issues by query string.
    pub fn search_issues(&self, query: &str) -> Result<Vec<Issue>> {
        let core_issues = self.0.search_issues(query)?;
        Ok(core_issues.into_iter().map(Into::into).collect())
    }

    /// Get all issues.
    pub fn get_all_issues(&self) -> Result<Vec<Issue>> {
        self.list_issues(None, None, None)
    }

    /// Resolve a partial ID to a full ID, converting error types.
    pub fn resolve_id(&self, partial_id: &str) -> Result<String> {
        Ok(self.0.resolve_id(partial_id)?)
    }

    /// Compute priority from label tags (forwarded from core).
    pub fn priority_from_tags(tags: &[String]) -> u8 {
        wk_core::Database::priority_from_tags(tags)
    }
}

impl std::ops::Deref for Database {
    type Target = wk_core::Database;

    fn deref(&self) -> &wk_core::Database {
        &self.0
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
