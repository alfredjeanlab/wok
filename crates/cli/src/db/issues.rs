// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::error::{Error, Result};
use crate::models::{Issue, IssueType, Status};

use super::Database;

impl Database {
    /// Create a new issue
    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        let core_issue: wk_core::Issue = issue.clone().into();
        self.0.create_issue(&core_issue)?;
        Ok(())
    }

    /// Get an issue by ID
    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let core_issue = self.0.get_issue(id)?;
        Ok(core_issue.into())
    }

    /// Check if an issue exists
    pub fn issue_exists(&self, id: &str) -> Result<bool> {
        Ok(self.0.issue_exists(id)?)
    }

    /// Resolve a potentially partial issue ID to a full ID.
    pub fn resolve_id(&self, partial_id: &str) -> Result<String> {
        self.0.resolve_id(partial_id).map_err(|e| match e {
            wk_core::Error::AmbiguousId { prefix, matches } => {
                Error::AmbiguousId { prefix, matches }
            }
            other => other.into(),
        })
    }

    /// Update issue status
    ///
    /// Sets `closed_at` to now when transitioning to a terminal state (done/closed),
    /// and clears it when transitioning to an active state (todo/in_progress).
    pub fn update_issue_status(&self, id: &str, status: Status) -> Result<()> {
        let now = chrono::Utc::now();
        let closed_at = if status.is_terminal() {
            Some(now.to_rfc3339())
        } else {
            None
        };

        self.0.conn.execute(
            "UPDATE issues SET status = ?1, updated_at = ?2, closed_at = ?3 WHERE id = ?4",
            rusqlite::params![status.as_str(), now.to_rfc3339(), closed_at, id],
        )?;
        if self.0.conn.changes() == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue title
    pub fn update_issue_title(&self, id: &str, title: &str) -> Result<()> {
        self.0.conn.execute(
            "UPDATE issues SET title = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![title, chrono::Utc::now().to_rfc3339(), id],
        )?;
        if self.0.conn.changes() == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue description
    pub fn update_issue_description(&self, id: &str, description: &str) -> Result<()> {
        self.0.conn.execute(
            "UPDATE issues SET description = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![description, chrono::Utc::now().to_rfc3339(), id],
        )?;
        if self.0.conn.changes() == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue type
    pub fn update_issue_type(&self, id: &str, issue_type: IssueType) -> Result<()> {
        self.0.conn.execute(
            "UPDATE issues SET type = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![issue_type.as_str(), chrono::Utc::now().to_rfc3339(), id],
        )?;
        if self.0.conn.changes() == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Set issue assignee
    pub fn set_assignee(&self, id: &str, assignee: &str) -> Result<()> {
        self.0.conn.execute(
            "UPDATE issues SET assignee = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![assignee, chrono::Utc::now().to_rfc3339(), id],
        )?;
        if self.0.conn.changes() == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Clear issue assignee
    pub fn clear_assignee(&self, id: &str) -> Result<()> {
        self.0.conn.execute(
            "UPDATE issues SET assignee = NULL, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now().to_rfc3339(), id],
        )?;
        if self.0.conn.changes() == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// List issues with optional filters
    pub fn list_issues(
        &self,
        status: Option<Status>,
        issue_type: Option<IssueType>,
        label: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let issues = self.0.list_issues(status, issue_type, label)?;
        Ok(issues.into_iter().map(|i| i.into()).collect())
    }

    /// Search issues by query string
    pub fn search_issues(&self, query: &str) -> Result<Vec<Issue>> {
        let issues = self.0.search_issues(query)?;
        Ok(issues.into_iter().map(|i| i.into()).collect())
    }

    /// Get IDs of blocked issues
    pub fn get_blocked_issue_ids(&self) -> Result<Vec<String>> {
        Ok(self.0.get_blocked_issue_ids()?)
    }

    /// Get all issues for export
    pub fn get_all_issues(&self) -> Result<Vec<Issue>> {
        self.list_issues(None, None, None)
    }

    /// Extract priority from tag list.
    pub fn priority_from_tags(tags: &[String]) -> u8 {
        // First look for priority: tag
        for tag in tags {
            if let Some(value) = tag.strip_prefix("priority:") {
                if let Some(p) = Self::parse_priority_value(value) {
                    return p;
                }
            }
        }
        // Then look for p: tag
        for tag in tags {
            if let Some(value) = tag.strip_prefix("p:") {
                if let Some(p) = Self::parse_priority_value(value) {
                    return p;
                }
            }
        }
        // Default: medium priority
        2
    }

    /// Parse priority value (numeric 0-4 or named)
    fn parse_priority_value(value: &str) -> Option<u8> {
        match value {
            "0" | "highest" => Some(0),
            "1" | "high" => Some(1),
            "2" | "medium" | "med" => Some(2),
            "3" | "low" => Some(3),
            "4" | "lowest" => Some(4),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "issues_tests.rs"]
mod tests;
