// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! SQLite-backed database for issue storage.
//!
//! The [`Database`] struct provides all data access operations for issues,
//! events, notes, tags, and dependencies.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use crate::error::{Error, Result};
use crate::hlc::Hlc;
use crate::issue::{Dependency, Event, Issue, IssueType, Note, Relation, Status};

/// SQL schema for the issue tracker database.
pub const SCHEMA: &str = r#"
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

/// Run schema creation and all migrations on a database connection.
///
/// This is the single migration path for all crates (core, CLI, daemon).
/// It applies the canonical schema and runs idempotent migrations to upgrade
/// older databases that may be missing columns or data.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    migrate_add_assignee(conn)?;
    migrate_add_hlc_columns(conn)?;
    migrate_backfill_prefixes(conn)?;
    Ok(())
}

/// Migration: Add assignee column to existing databases.
fn migrate_add_assignee(conn: &Connection) -> Result<()> {
    let has_assignee: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('issues') WHERE name = 'assignee'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_assignee {
        conn.execute("ALTER TABLE issues ADD COLUMN assignee TEXT", [])?;
    }
    Ok(())
}

/// Migration: Add HLC columns for CRDT sync compatibility.
///
/// Adds all HLC (Hybrid Logical Clock) columns used for conflict resolution
/// during sync. Older databases may be missing some or all of these.
fn migrate_add_hlc_columns(conn: &Connection) -> Result<()> {
    let columns = [
        "last_status_hlc",
        "last_title_hlc",
        "last_type_hlc",
        "last_description_hlc",
        "last_assignee_hlc",
    ];

    for column in columns {
        let has_column: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('issues') WHERE name = ?1",
                [column],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_column {
            let sql = format!("ALTER TABLE issues ADD COLUMN {column} TEXT");
            conn.execute(&sql, [])?;
        }
    }

    Ok(())
}

/// Migration: Backfill prefixes table from existing issues.
///
/// Extracts prefixes from issue IDs and populates the prefixes table
/// with correct issue counts. Only runs if the table is empty but
/// issues exist.
fn migrate_backfill_prefixes(conn: &Connection) -> Result<()> {
    let prefix_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM prefixes", [], |row| row.get(0))
        .unwrap_or(0);

    if prefix_count == 0 {
        conn.execute(
            "INSERT OR IGNORE INTO prefixes (prefix, created_at, issue_count)
             SELECT
                 substr(id, 1, instr(id, '-') - 1) as prefix,
                 MIN(created_at) as created_at,
                 COUNT(*) as issue_count
             FROM issues
             WHERE id LIKE '%-%'
             GROUP BY prefix",
            [],
        )?;
    }
    Ok(())
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
        run_migrations(&db.conn)?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Database { conn };
        run_migrations(&db.conn)?;
        Ok(db)
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
                |row| {
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
                },
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
            .query_map(params_refs.as_slice(), |row| {
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
            })?
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
            .query_map(params![issue_id], |row| {
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
            })?
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
            .query_map(params![limit_i64], |row| {
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
            })?
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
            .query_map(params![issue_id], |row| {
                let status_str: String = row.get(2)?;
                let created_str: String = row.get(4)?;
                Ok(Note {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    status: parse_db(&status_str, "status")?,
                    content: row.get(3)?,
                    created_at: parse_timestamp(&created_str, "created_at")?,
                })
            })?
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
            .query_map(params![from_id], |row| {
                let rel_str: String = row.get(2)?;
                let created_str: String = row.get(3)?;
                Ok(Dependency {
                    from_id: row.get(0)?,
                    to_id: row.get(1)?,
                    relation: parse_db(&rel_str, "rel")?,
                    created_at: parse_timestamp(&created_str, "created_at")?,
                })
            })?
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
            .prepare("SELECT to_id FROM deps WHERE from_id = ?1 AND rel = 'tracked_by'")?;

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
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;
