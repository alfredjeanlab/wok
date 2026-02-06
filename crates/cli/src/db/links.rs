// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Link database operations for external issue tracking.

use chrono::Utc;

use super::Database;
use crate::error::Result;
use crate::models::Link;

// Re-exported for test use via `use super::*`
#[cfg(test)]
use crate::models::{LinkRel, LinkType};

impl Database {
    /// Add an external link to an issue.
    pub fn add_link(&self, link: &Link) -> Result<i64> {
        Ok(self.0.add_link(link)?)
    }

    /// Get all external links for an issue.
    pub fn get_links(&self, issue_id: &str) -> Result<Vec<Link>> {
        Ok(self.0.get_links(issue_id)?)
    }

    /// Remove an external link by its ID.
    pub fn remove_link(&self, link_id: i64) -> Result<()> {
        Ok(self.0.remove_link(link_id)?)
    }

    /// Remove all links for an issue.
    pub fn remove_all_links(&self, issue_id: &str) -> Result<()> {
        Ok(self.0.remove_all_links(issue_id)?)
    }
}

/// Create a new Link with default values and the current timestamp.
pub fn new_link(issue_id: &str) -> Link {
    Link {
        id: 0,
        issue_id: issue_id.to_string(),
        link_type: None,
        url: None,
        external_id: None,
        rel: None,
        created_at: Utc::now(),
    }
}

#[cfg(test)]
#[path = "links_tests.rs"]
mod tests;
