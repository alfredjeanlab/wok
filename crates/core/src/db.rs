// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! SQLite-backed database for issue storage.
//!
//! The [`Database`] struct provides all data access operations for issues,
//! events, notes, tags, and dependencies.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hlc::Hlc;
use crate::issue::{Dependency, Event, Issue, IssueType, Note, Relation, Status};
use crate::link::{Link, LinkRel, LinkType};

/// SQL schema for the issue tracker database.
const SCHEMA: &str = r#"
-- Core issue table with HLC columns for conflict resolution
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'todo',
    assignee TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_status_hlc TEXT,
    last_title_hlc TEXT,
    last_type_hlc TEXT,
    last_description_hlc TEXT,
    last_assignee_hlc TEXT
);

-- Dependencies with relationship types
CREATE TABLE IF NOT EXISTS deps (
    from_id TEXT NOT NULL,
    to_id TEXT NOT NULL,
    rel TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (from_id, to_id, rel),
    FOREIGN KEY (from_id) REFERENCES issues(id),
    FOREIGN KEY (to_id) REFERENCES issues(id),
    CHECK (from_id != to_id)
);

-- Labels as raw strings
CREATE TABLE IF NOT EXISTS labels (
    issue_id TEXT NOT NULL,
    label TEXT NOT NULL,
    PRIMARY KEY (issue_id, label),
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Status-aware notes
CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    status TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Event log (audit trail)
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    action TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- External links to issue trackers
CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    link_type TEXT,              -- github|jira|gitlab|confluence|NULL
    url TEXT,                    -- full URL (may be NULL for shorthand)
    external_id TEXT,            -- external issue ID (e.g., "PE-5555")
    rel TEXT,                    -- import|blocks|tracks|tracked-by|NULL
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Prefix registry (auto-populated)
CREATE TABLE IF NOT EXISTS prefixes (
    prefix TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    issue_count INTEGER NOT NULL DEFAULT 0
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
CREATE INDEX IF NOT EXISTS idx_issues_type ON issues(type);
CREATE INDEX IF NOT EXISTS idx_deps_to ON deps(to_id);
CREATE INDEX IF NOT EXISTS idx_deps_rel ON deps(rel);
CREATE INDEX IF NOT EXISTS idx_labels_label ON labels(label);
CREATE INDEX IF NOT EXISTS idx_events_issue ON events(issue_id);
CREATE INDEX IF NOT EXISTS idx_links_issue ON links(issue_id);
CREATE INDEX IF NOT EXISTS idx_prefixes_count ON prefixes(issue_count DESC);
"#;

/// Parse a string value from the database, returning a rusqlite error on parse failure.
fn parse_db<T: std::str::FromStr>(
    value: &str,
    column: &str,
) -> std::result::Result<T, rusqlite::Error> {
    value.parse().map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(Error::CorruptedData(format!(
                "invalid value '{value}' in column '{column}'"
            ))),
        )
    })
}

/// Parse an RFC3339 timestamp from the database.
fn parse_timestamp(
    value: &str,
    column: &str,
) -> std::result::Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(Error::CorruptedData(format!(
                    "invalid timestamp '{value}' in column '{column}'"
                ))),
            )
        })
}

/// Parse an optional HLC from the database.
fn parse_hlc_opt(value: Option<String>) -> std::result::Result<Option<Hlc>, rusqlite::Error> {
    match value {
        None => Ok(None),
        Some(s) => s.parse().map(Some).map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(Error::CorruptedData(format!("invalid HLC '{s}'"))),
            )
        }),
    }
}

/// Map a row to an Issue.
///
/// Expected columns: id, type, title, description, status, assignee,
/// created_at, updated_at, last_status_hlc, last_title_hlc,
/// last_type_hlc, last_description_hlc, last_assignee_hlc
fn row_to_issue(row: &rusqlite::Row) -> rusqlite::Result<Issue> {
    let type_str: String = row.get(1)?;
    let status_str: String = row.get(4)?;
    let created_str: String = row.get(6)?;
    let updated_str: String = row.get(7)?;
    let status_hlc: Option<String> = row.get(8)?;
    let title_hlc: Option<String> = row.get(9)?;
    let type_hlc: Option<String> = row.get(10)?;
    let desc_hlc: Option<String> = row.get(11)?;
    let assignee_hlc: Option<String> = row.get(12)?;

    Ok(Issue {
        id: row.get(0)?,
        issue_type: parse_db(&type_str, "type")?,
        title: row.get(2)?,
        description: row.get(3)?,
        status: parse_db(&status_str, "status")?,
        assignee: row.get(5)?,
        created_at: parse_timestamp(&created_str, "created_at")?,
        updated_at: parse_timestamp(&updated_str, "updated_at")?,
        last_status_hlc: parse_hlc_opt(status_hlc)?,
        last_title_hlc: parse_hlc_opt(title_hlc)?,
        last_type_hlc: parse_hlc_opt(type_hlc)?,
        last_description_hlc: parse_hlc_opt(desc_hlc)?,
        last_assignee_hlc: parse_hlc_opt(assignee_hlc)?,
    })
}

/// Map a row to an Event.
///
/// Expected columns: id, issue_id, action, old_value, new_value, reason, created_at
fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
    let action_str: String = row.get(2)?;
    let created_str: String = row.get(6)?;
    Ok(Event {
        id: row.get(0)?,
        issue_id: row.get(1)?,
        action: parse_db(&action_str, "action")?,
        old_value: row.get(3)?,
        new_value: row.get(4)?,
        reason: row.get(5)?,
        created_at: parse_timestamp(&created_str, "created_at")?,
    })
}

/// Map a row to a Note.
///
/// Expected columns: id, issue_id, status, content, created_at
fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let status_str: String = row.get(2)?;
    let created_str: String = row.get(4)?;
    Ok(Note {
        id: row.get(0)?,
        issue_id: row.get(1)?,
        status: parse_db(&status_str, "status")?,
        content: row.get(3)?,
        created_at: parse_timestamp(&created_str, "created_at")?,
    })
}

/// Map a row to a Dependency.
///
/// Expected columns: from_id, to_id, rel, created_at
fn row_to_dependency(row: &rusqlite::Row) -> rusqlite::Result<Dependency> {
    let rel_str: String = row.get(2)?;
    let created_str: String = row.get(3)?;
    Ok(Dependency {
        from_id: row.get(0)?,
        to_id: row.get(1)?,
        relation: parse_db(&rel_str, "rel")?,
        created_at: parse_timestamp(&created_str, "created_at")?,
    })
}

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

/// SQLite database connection with issue tracker operations.
pub struct Database {
    /// The underlying SQLite connection.
    pub conn: Connection,
}

impl Database {
    /// Open a database connection at the given path, creating and migrating if needed.
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(path)?;

        // Enable foreign keys and WAL mode for concurrency
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Run database migrations.
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA)?;
        self.migrate_tracked_by_relation()?;
        Ok(())
    }

    /// Migration: Rewrite "tracked_by" to "tracked-by" in deps table.
    ///
    /// Early versions serialized TrackedBy as "tracked_by" (underscore).
    /// The canonical form is "tracked-by" (kebab-case).
    fn migrate_tracked_by_relation(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE deps SET rel = 'tracked-by' WHERE rel = 'tracked_by'",
            [],
        )?;
        Ok(())
    }

    /// Create a new issue.
    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        self.conn.execute(
            "INSERT INTO issues (id, type, title, description, status, assignee,
             created_at, updated_at, last_status_hlc, last_title_hlc, last_type_hlc,
             last_description_hlc, last_assignee_hlc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                issue.id,
                issue.issue_type.as_str(),
                issue.title,
                issue.description,
                issue.status.as_str(),
                issue.assignee,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
                issue.last_status_hlc.map(|h| h.to_string()),
                issue.last_title_hlc.map(|h| h.to_string()),
                issue.last_type_hlc.map(|h| h.to_string()),
                issue.last_description_hlc.map(|h| h.to_string()),
                issue.last_assignee_hlc.map(|h| h.to_string()),
            ],
        )?;
        Ok(())
    }

    /// Get an issue by ID.
    pub fn get_issue(&self, id: &str) -> Result<Issue> {
        let issue = self
            .conn
            .query_row(
                "SELECT id, type, title, description, status, assignee,
                        created_at, updated_at, last_status_hlc, last_title_hlc,
                        last_type_hlc, last_description_hlc, last_assignee_hlc
                 FROM issues WHERE id = ?1",
                params![id],
                row_to_issue,
            )
            .optional()?;

        issue.ok_or_else(|| Error::IssueNotFound(id.to_string()))
    }

    /// Check if an issue exists.
    pub fn issue_exists(&self, id: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM issues WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Update issue status.
    pub fn update_issue_status(&mut self, id: &str, status: Status) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue status HLC.
    pub fn update_issue_status_hlc(&mut self, id: &str, hlc: Hlc) -> Result<()> {
        self.conn.execute(
            "UPDATE issues SET last_status_hlc = ?1 WHERE id = ?2",
            params![hlc.to_string(), id],
        )?;
        Ok(())
    }

    /// Update issue title.
    pub fn update_issue_title(&mut self, id: &str, title: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue title HLC.
    pub fn update_issue_title_hlc(&mut self, id: &str, hlc: Hlc) -> Result<()> {
        self.conn.execute(
            "UPDATE issues SET last_title_hlc = ?1 WHERE id = ?2",
            params![hlc.to_string(), id],
        )?;
        Ok(())
    }

    /// Update issue type.
    pub fn update_issue_type(&mut self, id: &str, issue_type: IssueType) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET type = ?1, updated_at = ?2 WHERE id = ?3",
            params![issue_type.as_str(), Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Update issue type HLC.
    pub fn update_issue_type_hlc(&mut self, id: &str, hlc: Hlc) -> Result<()> {
        self.conn.execute(
            "UPDATE issues SET last_type_hlc = ?1 WHERE id = ?2",
            params![hlc.to_string(), id],
        )?;
        Ok(())
    }

    /// List issues with optional filters.
    pub fn list_issues(
        &self,
        status: Option<Status>,
        issue_type: Option<IssueType>,
        label: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let mut sql = String::from(
            "SELECT DISTINCT i.id, i.type, i.title, i.description, i.status, i.assignee,
             i.created_at, i.updated_at, i.last_status_hlc, i.last_title_hlc,
             i.last_type_hlc, i.last_description_hlc, i.last_assignee_hlc
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
            .query_map(params_refs.as_slice(), row_to_issue)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(issues)
    }

    /// Get IDs of blocked issues (issues with at least one open blocker).
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

    /// Get all issues.
    pub fn get_all_issues(&self) -> Result<Vec<Issue>> {
        self.list_issues(None, None, None)
    }

    /// Log an event.
    pub fn log_event(&self, event: &Event) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO events (issue_id, action, old_value, new_value, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.issue_id,
                event.action.as_str(),
                event.old_value,
                event.new_value,
                event.reason,
                event.created_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all events for an issue, ordered by creation time.
    pub fn get_events(&self, issue_id: &str) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, action, old_value, new_value, reason, created_at
             FROM events WHERE issue_id = ?1 ORDER BY created_at",
        )?;

        let events = stmt
            .query_map(params![issue_id], row_to_event)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(events)
    }

    /// Get recent events across all issues.
    pub fn get_recent_events(&self, limit: usize) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, action, old_value, new_value, reason, created_at
             FROM events ORDER BY created_at DESC LIMIT ?1",
        )?;

        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        let events = stmt
            .query_map(params![limit_i64], row_to_event)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(events)
    }

    /// Add a note to an issue.
    pub fn add_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO notes (issue_id, status, content, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![issue_id, status.as_str(), content, Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all notes for an issue, ordered by creation time.
    pub fn get_notes(&self, issue_id: &str) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, status, content, created_at
             FROM notes WHERE issue_id = ?1 ORDER BY created_at",
        )?;

        let notes = stmt
            .query_map(params![issue_id], row_to_note)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    /// Add a label to an issue.
    pub fn add_label(&self, issue_id: &str, label: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO labels (issue_id, label) VALUES (?1, ?2)",
            params![issue_id, label],
        )?;
        Ok(())
    }

    /// Remove a label from an issue.
    pub fn remove_label(&self, issue_id: &str, label: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM labels WHERE issue_id = ?1 AND label = ?2",
            params![issue_id, label],
        )?;
        Ok(affected > 0)
    }

    /// Get all labels for an issue.
    pub fn get_labels(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT label FROM labels WHERE issue_id = ?1 ORDER BY label")?;

        let labels = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(labels)
    }

    /// Get all labels as (issue_id, label) pairs.
    pub fn get_all_labels(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT issue_id, label FROM labels ORDER BY issue_id, label")?;

        let labels = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(labels)
    }

    /// Add a dependency between two issues.
    pub fn add_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        if from_id == to_id {
            return Err(Error::SelfDependency);
        }

        // Check if adding this would create a cycle (only for blocks)
        if relation == Relation::Blocks && self.would_create_cycle(from_id, to_id)? {
            return Err(Error::CycleDetected);
        }

        self.conn.execute(
            "INSERT OR IGNORE INTO deps (from_id, to_id, rel, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![from_id, to_id, relation.as_str(), Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// Remove a dependency between two issues.
    pub fn remove_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        let affected = self.conn.execute(
            "DELETE FROM deps WHERE from_id = ?1 AND to_id = ?2 AND rel = ?3",
            params![from_id, to_id, relation.as_str()],
        )?;

        if affected == 0 {
            return Err(Error::DependencyNotFound {
                from: from_id.to_string(),
                rel: relation.to_string(),
                to: to_id.to_string(),
            });
        }

        Ok(())
    }

    /// Check if adding from_id -> to_id would create a cycle.
    fn would_create_cycle(&self, from_id: &str, to_id: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "WITH RECURSIVE chain(id) AS (
                SELECT from_id FROM deps WHERE to_id = ?1 AND rel = 'blocks'
                UNION
                SELECT d.from_id FROM deps d JOIN chain c ON d.to_id = c.id WHERE d.rel = 'blocks'
            )
            SELECT COUNT(*) FROM chain WHERE id = ?2",
            params![from_id, to_id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Get all dependencies from an issue.
    pub fn get_deps_from(&self, from_id: &str) -> Result<Vec<Dependency>> {
        let mut stmt = self
            .conn
            .prepare("SELECT from_id, to_id, rel, created_at FROM deps WHERE from_id = ?1")?;

        let deps = stmt
            .query_map(params![from_id], row_to_dependency)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(deps)
    }

    /// Get issues that directly block the given issue.
    pub fn get_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT from_id FROM deps WHERE to_id = ?1 AND rel = 'blocks'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get all issues that transitively block the given issue (active blockers only).
    pub fn get_transitive_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "WITH RECURSIVE blockers AS (
                SELECT d.from_id as blocker_id
                FROM deps d JOIN issues i ON i.id = d.from_id
                WHERE d.to_id = ?1 AND d.rel = 'blocks'
                  AND i.status NOT IN ('done', 'closed')
                UNION
                SELECT d.from_id
                FROM deps d JOIN issues i ON i.id = d.from_id
                JOIN blockers b ON d.to_id = b.blocker_id
                WHERE d.rel = 'blocks' AND i.status NOT IN ('done', 'closed')
            )
            SELECT blocker_id FROM blockers",
        )?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get issues that this issue blocks.
    pub fn get_blocking(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM deps WHERE from_id = ?1 AND rel = 'blocks'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get tracking issues (issues this is tracked by).
    pub fn get_tracking(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM deps WHERE from_id = ?1 AND rel = 'tracked-by'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get tracked issues (issues this tracks).
    pub fn get_tracked(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM deps WHERE from_id = ?1 AND rel = 'tracks'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    // -- Upstreamed from CLI --------------------------------------------------

    /// Minimum prefix length for prefix matching.
    const MIN_PREFIX_LENGTH: usize = 3;

    /// Resolve a potentially partial issue ID to a full ID.
    ///
    /// Resolution strategy:
    /// 1. Exact match (fast path)
    /// 2. Prefix match if length >= 3
    /// 3. Error if no match or multiple matches
    pub fn resolve_id(&self, partial_id: &str) -> Result<String> {
        if self.issue_exists(partial_id)? {
            return Ok(partial_id.to_string());
        }

        if partial_id.len() < Self::MIN_PREFIX_LENGTH {
            return Err(Error::IssueNotFound(partial_id.to_string()));
        }

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

    /// Search issues by query string across title, description, and assignee.
    ///
    /// Special characters % and _ are escaped to prevent SQL LIKE interpretation.
    pub fn search_issues(&self, query: &str) -> Result<Vec<Issue>> {
        let escaped_query = query.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped_query);
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT i.id, i.type, i.title, i.description, i.status, i.assignee,
                    i.created_at, i.updated_at, i.last_status_hlc, i.last_title_hlc,
                    i.last_type_hlc, i.last_description_hlc, i.last_assignee_hlc
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
            .query_map(params![&pattern], row_to_issue)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(issues)
    }

    /// Update issue description.
    pub fn update_issue_description(&mut self, id: &str, description: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![description, Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Set issue assignee.
    pub fn set_assignee(&mut self, id: &str, assignee: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET assignee = ?1, updated_at = ?2 WHERE id = ?3",
            params![assignee, Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Clear issue assignee.
    pub fn clear_assignee(&mut self, id: &str) -> Result<()> {
        let affected = self.conn.execute(
            "UPDATE issues SET assignee = NULL, updated_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;

        if affected == 0 {
            return Err(Error::IssueNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Get labels for multiple issues in a single query.
    pub fn get_labels_batch(&self, issue_ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
        if issue_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<_> = (1..=issue_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "SELECT issue_id, label FROM labels WHERE issue_id IN ({}) ORDER BY issue_id, label",
            placeholders.join(", ")
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = issue_ids
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let mut rows = stmt.query(params.as_slice())?;
        while let Some(row) = rows.next()? {
            let issue_id: String = row.get(0)?;
            let label: String = row.get(1)?;
            map.entry(issue_id).or_default().push(label);
        }

        Ok(map)
    }

    /// Get all external links for an issue.
    pub fn get_links(&self, issue_id: &str) -> Result<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, link_type, url, external_id, rel, created_at
             FROM links WHERE issue_id = ?1 ORDER BY created_at ASC",
        )?;

        let links = stmt
            .query_map([issue_id], row_to_link)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(links)
    }

    /// Get a specific link by issue ID and URL.
    pub fn get_link_by_url(&self, issue_id: &str, url: &str) -> Result<Option<Link>> {
        let link = self
            .conn
            .query_row(
                "SELECT id, issue_id, link_type, url, external_id, rel, created_at
                 FROM links WHERE issue_id = ?1 AND url = ?2",
                params![issue_id, url],
                row_to_link,
            )
            .optional()?;

        Ok(link)
    }

    /// Add an external link to an issue.
    pub fn add_link(&self, link: &Link) -> Result<i64> {
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
                link.created_at.to_rfc3339(),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Remove an external link by its ID.
    pub fn remove_link(&self, link_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM links WHERE id = ?1", [link_id])?;
        Ok(())
    }

    /// Remove all links for an issue.
    pub fn remove_all_links(&self, issue_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM links WHERE issue_id = ?1", [issue_id])?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;
