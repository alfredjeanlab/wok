// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Link database operations for external issue tracking.

use chrono::Utc;
use rusqlite::params;

use super::{parse_db, parse_timestamp, Database};
use crate::error::Result;
use crate::models::{Link, LinkRel, LinkType};

/// Map a row to a Link.
///
/// Expected columns: id, issue_id, link_type, url, external_id, rel, created_at
fn row_to_link(row: &rusqlite::Row) -> rusqlite::Result<Link> {
    let link_type_str: Option<String> = row.get(2)?;
    let link_type = link_type_str
        .map(|s| parse_db::<LinkType>(&s, "link_type"))
        .transpose()?;
    let rel_str: Option<String> = row.get(5)?;
    let rel = rel_str
        .map(|s| parse_db::<LinkRel>(&s, "rel"))
        .transpose()?;
    let created_at_str: String = row.get(6)?;
    Ok(Link {
        id: row.get(0)?,
        issue_id: row.get(1)?,
        link_type,
        url: row.get(3)?,
        external_id: row.get(4)?,
        rel,
        created_at: parse_timestamp(&created_at_str, "created_at")?,
    })
}

impl Database {
    /// Add an external link to an issue.
    ///
    /// Returns the database-assigned ID for the new link.
    pub fn add_link(&self, link: &Link) -> Result<i64> {
        let created_at = link.created_at.to_rfc3339();
        let link_type_str = link.link_type.map(|t| t.as_str().to_string());
        let rel_str = link.rel.map(|r| r.as_str().to_string());

        self.conn.execute(
            "INSERT INTO links (issue_id, link_type, url, external_id, rel, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                link.issue_id,
                link_type_str,
                link.url,
                link.external_id,
                rel_str,
                created_at,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all external links for an issue.
    pub fn get_links(&self, issue_id: &str) -> Result<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, link_type, url, external_id, rel, created_at
             FROM links
             WHERE issue_id = ?1
             ORDER BY created_at ASC",
        )?;

        let links = stmt
            .query_map([issue_id], row_to_link)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(links)
    }

    /// Remove an external link by its ID.
    pub fn remove_link(&self, link_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM links WHERE id = ?1", [link_id])?;
        Ok(())
    }

    /// Remove all links for an issue.
    ///
    /// This is useful when deleting an issue.
    pub fn remove_all_links(&self, issue_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM links WHERE issue_id = ?1", [issue_id])?;
        Ok(())
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
