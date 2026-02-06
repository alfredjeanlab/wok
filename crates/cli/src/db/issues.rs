// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::Utc;
use rusqlite::{params, OptionalExtension};

use crate::error::{Error, Result};
use crate::models::{Issue, IssueType, Status};

use super::{parse_db, parse_timestamp, Database};

impl Database {
    /// Create a new issue
    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        self.conn.execute(
            "INSERT INTO issues (id, type, title, description, status, assignee,
             created_at, updated_at, closed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                issue.id,
                issue.issue_type.as_str(),
                issue.title,
                issue.description,
                issue.status.as_str(),
                issue.assignee,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
                issue.closed_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    /// Get an issue by ID
    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let issue = self
            .conn
            .query_row(
                "SELECT i.id, i.type, i.title, i.description, i.status, i.assignee,
                        i.created_at, i.updated_at, i.closed_at
                 FROM issues i WHERE i.id = ?1",
                params![id],
                |row| {
                    let type_str: String = row.get(1)?;
                    let status_str: String = row.get(4)?;
                    let created_str: String = row.get(6)?;
                    let updated_str: String = row.get(7)?;
                    let closed_str: Option<String> = row.get(8)?;
                    Ok(Issue {
                        id: row.get(0)?,
                        issue_type: parse_db(&type_str, "type")?,
                        title: row.get(2)?,
                        description: row.get(3)?,
                        status: parse_db(&status_str, "status")?,
                        assignee: row.get(5)?,
                        created_at: parse_timestamp(&created_str, "created_at")?,
                        updated_at: parse_timestamp(&updated_str, "updated_at")?,
                        closed_at: closed_str
                            .as_ref()
                            .map(|s| parse_timestamp(s, "closed_at"))
                            .transpose()?,
                    })
                },
            )
            .optional()?;

        issue.ok_or_else(|| Error::IssueNotFound(id.to_string()))
    }

    /// Check if an issue exists
    pub fn issue_exists(&self, id: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM issues WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Minimum prefix length for prefix matching
    const MIN_PREFIX_LENGTH: usize = 3;

    /// Resolve a potentially partial issue ID to a full ID.
    ///
    /// Returns the full ID if exactly one match is found.
    /// Returns an error if no match or multiple matches.
    ///
    /// Resolution strategy:
    /// 1. Exact match - Check if the ID exists exactly (fast path)
    /// 2. Prefix match - If no exact match and length >= 3, search for prefix matches
    /// 3. Ambiguity check - If multiple prefix matches, return error with all matches
    pub fn resolve_id(&self, partial_id: &str) -> Result<String> {
        // First try exact match (fast path)
        if self.issue_exists(partial_id)? {
            return Ok(partial_id.to_string());
        }

        // Check minimum length for prefix matching
        if partial_id.len() < Self::MIN_PREFIX_LENGTH {
            return Err(Error::IssueNotFound(partial_id.to_string()));
        }

        // Find all IDs that start with the prefix
        let pattern = format!("{}%", partial_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM issues WHERE id LIKE ?1")?;

        let matches: Vec<String> = stmt
            .query_map([&pattern], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        match matches.as_slice() {
            [] => Err(Error::IssueNotFound(partial_id.to_string())),
            [single] => Ok(single.clone()),
            _ => Err(Error::AmbiguousId {
                prefix: partial_id.to_string(),
                matches,
            }),
        }
    }

    /// Update issue status
    ///
    /// Sets `closed_at` to now when transitioning to a terminal state (done/closed),
    /// and clears it when transitioning to an active state (todo/in_progress).
    pub fn update_issue_status(&self, id: &str, status: Status) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let closed_at = if matches!(status, Status::Done | Status::Closed) {
            Some(now.clone())
        } else {
            None
        };
        let affected = self.conn.execute(
            "UPDATE issues SET status = ?1, updated_at = ?2, closed_at = ?3 WHERE id = ?4",
            params![status.as_str(), now, closed_at, id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue title
    pub fn update_issue_title(&self, id: &str, title: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue description
    pub fn update_issue_description(&self, id: &str, description: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![description, Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue type
    pub fn update_issue_type(&self, id: &str, issue_type: IssueType) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET type = ?1, updated_at = ?2 WHERE id = ?3",
            params![issue_type.as_str(), Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Set issue assignee
    pub fn set_assignee(&self, id: &str, assignee: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET assignee = ?1, updated_at = ?2 WHERE id = ?3",
            params![assignee, Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Clear issue assignee
    pub fn clear_assignee(&self, id: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET assignee = NULL, updated_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
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
        let mut sql = String::from(
            "SELECT DISTINCT i.id, i.type, i.title, i.description, i.status, i.assignee,
                    i.created_at, i.updated_at, i.closed_at
             FROM issues i",
        );

        let mut conditions = Vec::new();
        let mut params_vec: Vec<String> = Vec::new();

        if label.is_some() {
            sql.push_str(" JOIN labels l ON i.id = l.issue_id");
        }

        if let Some(s) = status {
            conditions.push("i.status = ?".to_string());
            params_vec.push(s.as_str().to_string());
        }

        if let Some(t) = issue_type {
            conditions.push("i.type = ?".to_string());
            params_vec.push(t.as_str().to_string());
        }

        if let Some(l) = label {
            conditions.push("l.label = ?".to_string());
            params_vec.push(l.to_string());
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY i.created_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let issues = stmt
            .query_map(params_refs.as_slice(), |row| {
                let type_str: String = row.get(1)?;
                let status_str: String = row.get(4)?;
                let created_str: String = row.get(6)?;
                let updated_str: String = row.get(7)?;
                let closed_str: Option<String> = row.get(8)?;
                Ok(Issue {
                    id: row.get(0)?,
                    issue_type: parse_db(&type_str, "type")?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    status: parse_db(&status_str, "status")?,
                    assignee: row.get(5)?,
                    created_at: parse_timestamp(&created_str, "created_at")?,
                    updated_at: parse_timestamp(&updated_str, "updated_at")?,
                    closed_at: closed_str
                        .as_ref()
                        .map(|s| parse_timestamp(s, "closed_at"))
                        .transpose()?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(issues)
    }

    /// Search issues by query string across title, description, assignee, notes, labels, and links.
    /// Returns issues where any searchable field contains the query (case-insensitive).
    /// Special characters % and _ are escaped to prevent SQL LIKE syntax interpretation.
    pub fn search_issues(&self, query: &str) -> Result<Vec<Issue>> {
        let escaped_query = query.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped_query);
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT i.id, i.type, i.title, i.description, i.status, i.assignee,
                    i.created_at, i.updated_at, i.closed_at
             FROM issues i
             LEFT JOIN notes n ON n.issue_id = i.id
             LEFT JOIN labels l ON l.issue_id = i.id
             LEFT JOIN links lk ON lk.issue_id = i.id
             WHERE i.title LIKE ?1 COLLATE NOCASE ESCAPE '\\'
                OR i.description LIKE ?1 COLLATE NOCASE ESCAPE '\\'
                OR i.assignee LIKE ?1 COLLATE NOCASE ESCAPE '\\'
                OR n.content LIKE ?1 COLLATE NOCASE ESCAPE '\\'
                OR l.label LIKE ?1 COLLATE NOCASE ESCAPE '\\'
                OR lk.url LIKE ?1 COLLATE NOCASE ESCAPE '\\'
                OR lk.external_id LIKE ?1 COLLATE NOCASE ESCAPE '\\'
             ORDER BY i.created_at DESC",
        )?;

        let issues = stmt
            .query_map(params![&pattern], |row| {
                let type_str: String = row.get(1)?;
                let status_str: String = row.get(4)?;
                let created_str: String = row.get(6)?;
                let updated_str: String = row.get(7)?;
                let closed_str: Option<String> = row.get(8)?;
                Ok(Issue {
                    id: row.get(0)?,
                    issue_type: parse_db(&type_str, "type")?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    status: parse_db(&status_str, "status")?,
                    assignee: row.get(5)?,
                    created_at: parse_timestamp(&created_str, "created_at")?,
                    updated_at: parse_timestamp(&updated_str, "updated_at")?,
                    closed_at: closed_str
                        .as_ref()
                        .map(|s| parse_timestamp(s, "closed_at"))
                        .transpose()?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(issues)
    }

    /// Get IDs of blocked issues (issues with at least one open blocker)
    pub fn get_blocked_issue_ids(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "WITH RECURSIVE all_blockers(issue_id, blocker_id) AS (
                SELECT to_id, from_id FROM deps WHERE rel = 'blocks'
                UNION
                SELECT ab.issue_id, d.from_id
                FROM all_blockers ab
                JOIN deps d ON d.to_id = ab.blocker_id AND d.rel = 'blocks'
            )
            SELECT DISTINCT issue_id FROM all_blockers ab
            JOIN issues i ON i.id = ab.blocker_id
            WHERE i.status IN ('todo', 'in_progress')",
        )?;

        let ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get all issues for export (includes all data)
    pub fn get_all_issues(&self) -> Result<Vec<Issue>> {
        self.list_issues(None, None, None)
    }

    /// Extract priority from tag list.
    /// Prefers "priority:" over "p:" if both present.
    /// Returns 0-4 where 0 is highest priority.
    /// Default (no priority tag): 2 (medium)
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
