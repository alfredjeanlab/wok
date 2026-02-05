// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database operations for the daemon.
//!
//! This module provides SQLite database access for handling IPC requests.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::ipc::{
    Action, Dependency, Event, Issue, IssueType, Link, LinkRel, LinkType, MutateOp, MutateResult,
    Note, PrefixInfo, QueryOp, QueryResult, Relation, Status,
};

/// SQL schema for the issue tracker database.
const SCHEMA: &str = r#"
-- Core issue table
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'todo',
    assignee TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
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
    link_type TEXT,
    url TEXT,
    external_id TEXT,
    rel TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Prefix registry
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

/// Database wrapper for the daemon.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a database at the given path.
    pub fn open(path: &Path) -> Result<Self, String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("failed to create db directory: {}", e))?;
            }
        }

        let conn = Connection::open(path).map_err(|e| format!("failed to open database: {}", e))?;

        // Configure SQLite for concurrent access
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|e| format!("failed to configure database: {}", e))?;

        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .execute_batch(SCHEMA)
            .map_err(|e| format!("failed to run schema: {}", e))?;

        // Add assignee column if missing
        let has_assignee: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('issues') WHERE name = 'assignee'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_assignee {
            self.conn
                .execute("ALTER TABLE issues ADD COLUMN assignee TEXT", [])
                .map_err(|e| format!("migration failed: {}", e))?;
        }

        // Add HLC columns if missing (for sync compatibility)
        let hlc_columns = ["last_status_hlc", "last_title_hlc", "last_type_hlc"];
        for col in hlc_columns {
            let has_col: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM pragma_table_info('issues') WHERE name = ?1",
                    [col],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if !has_col {
                let sql = format!("ALTER TABLE issues ADD COLUMN {} TEXT", col);
                self.conn
                    .execute(&sql, [])
                    .map_err(|e| format!("migration failed: {}", e))?;
            }
        }

        // Backfill prefixes table
        let prefix_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM prefixes", [], |row| row.get(0))
            .unwrap_or(0);

        if prefix_count == 0 {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO prefixes (prefix, created_at, issue_count)
                 SELECT
                     substr(id, 1, instr(id, '-') - 1) as prefix,
                     MIN(created_at) as created_at,
                     COUNT(*) as issue_count
                 FROM issues
                 WHERE id LIKE '%-%'
                 GROUP BY prefix",
                    [],
                )
                .map_err(|e| format!("prefix backfill failed: {}", e))?;
        }

        Ok(())
    }

    /// Execute a query operation and return the result.
    pub fn execute_query(&self, op: QueryOp) -> Result<QueryResult, String> {
        match op {
            QueryOp::ResolveId { partial_id } => self.resolve_id(&partial_id),
            QueryOp::IssueExists { id } => self.issue_exists(&id),
            QueryOp::GetIssue { id } => self.get_issue(&id),
            QueryOp::ListIssues {
                status,
                issue_type,
                label,
            } => self.list_issues(status, issue_type, label),
            QueryOp::SearchIssues { query } => self.search_issues(&query),
            QueryOp::GetBlockedIssueIds => self.get_blocked_issue_ids(),
            QueryOp::GetLabels { id } => self.get_labels(&id),
            QueryOp::GetLabelsBatch { ids } => self.get_labels_batch(&ids),
            QueryOp::GetNotes { id } => self.get_notes(&id),
            QueryOp::GetEvents { id } => self.get_events(&id),
            QueryOp::GetAllEvents { limit } => self.get_all_events(limit),
            QueryOp::GetDepsFrom { id } => self.get_deps_from(&id),
            QueryOp::GetBlockers { id } => self.get_blockers(&id),
            QueryOp::GetBlocking { id } => self.get_blocking(&id),
            QueryOp::GetTracked { id } => self.get_tracked(&id),
            QueryOp::GetTracking { id } => self.get_tracking(&id),
            QueryOp::GetTransitiveBlockers { id } => self.get_transitive_blockers(&id),
            QueryOp::GetLinks { id } => self.get_links(&id),
            QueryOp::GetLinkByUrl { id, url } => self.get_link_by_url(&id, &url),
            QueryOp::ListPrefixes => self.list_prefixes(),
        }
    }

    /// Execute a mutation operation and return the result.
    pub fn execute_mutate(&self, op: MutateOp) -> Result<MutateResult, String> {
        match op {
            MutateOp::CreateIssue { issue } => self.create_issue(&issue),
            MutateOp::UpdateIssueStatus { id, status } => self.update_issue_status(&id, status),
            MutateOp::UpdateIssueTitle { id, title } => self.update_issue_title(&id, &title),
            MutateOp::UpdateIssueDescription { id, description } => {
                self.update_issue_description(&id, &description)
            }
            MutateOp::UpdateIssueType { id, issue_type } => self.update_issue_type(&id, issue_type),
            MutateOp::SetAssignee { id, assignee } => self.set_assignee(&id, &assignee),
            MutateOp::ClearAssignee { id } => self.clear_assignee(&id),
            MutateOp::AddLabel { id, label } => self.add_label(&id, &label),
            MutateOp::RemoveLabel { id, label } => self.remove_label(&id, &label),
            MutateOp::AddNote {
                id,
                status,
                content,
            } => self.add_note(&id, status, &content),
            MutateOp::LogEvent { event } => self.log_event(&event),
            MutateOp::AddDependency {
                from_id,
                to_id,
                relation,
            } => self.add_dependency(&from_id, &to_id, relation),
            MutateOp::RemoveDependency {
                from_id,
                to_id,
                relation,
            } => self.remove_dependency(&from_id, &to_id, relation),
            MutateOp::AddLink {
                id,
                link_type,
                url,
                external_id,
                rel,
            } => self.add_link(&id, link_type, url, external_id, rel),
            MutateOp::RemoveLink { id, url } => self.remove_link(&id, &url),
            MutateOp::EnsurePrefix { prefix } => self.ensure_prefix(&prefix),
            MutateOp::IncrementPrefixCount { prefix } => self.increment_prefix_count(&prefix),
        }
    }

    // ========================================================================
    // Query implementations
    // ========================================================================

    fn resolve_id(&self, partial_id: &str) -> Result<QueryResult, String> {
        // Exact match first
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM issues WHERE id = ?1",
                [partial_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if exists {
            return Ok(QueryResult::ResolvedId {
                id: partial_id.to_string(),
            });
        }

        // Prefix match
        if partial_id.len() < 3 {
            return Err(format!("issue not found: {}", partial_id));
        }

        let pattern = format!("{}%", partial_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM issues WHERE id LIKE ?1")
            .map_err(|e| e.to_string())?;

        let matches: Vec<String> = stmt
            .query_map([&pattern], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        match matches.len() {
            0 => Err(format!("issue not found: {}", partial_id)),
            1 => Ok(QueryResult::ResolvedId {
                id: matches.into_iter().next().unwrap_or_default(),
            }),
            _ => Err(format!(
                "ambiguous ID '{}': matches {}",
                partial_id,
                matches.join(", ")
            )),
        }
    }

    fn issue_exists(&self, id: &str) -> Result<QueryResult, String> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM issues WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(QueryResult::Bool { value: exists })
    }

    fn get_issue(&self, id: &str) -> Result<QueryResult, String> {
        let issue = self
            .conn
            .query_row(
                "SELECT i.id, i.type, i.title, i.description, i.status, i.assignee,
                        i.created_at, i.updated_at,
                        (SELECT MAX(e.created_at) FROM events e
                         WHERE e.issue_id = i.id AND e.action IN ('done', 'closed')
                         AND NOT EXISTS (
                             SELECT 1 FROM events e2
                             WHERE e2.issue_id = e.issue_id
                             AND e2.action = 'reopened'
                             AND e2.created_at > e.created_at
                         )) as closed_at
                 FROM issues i WHERE i.id = ?1",
                [id],
                |row| self.row_to_issue(row),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        match issue {
            Some(issue) => Ok(QueryResult::Issue { issue }),
            None => Err(format!("issue not found: {}", id)),
        }
    }

    fn list_issues(
        &self,
        status: Option<Status>,
        issue_type: Option<IssueType>,
        label: Option<String>,
    ) -> Result<QueryResult, String> {
        let mut sql = String::from(
            "SELECT DISTINCT i.id, i.type, i.title, i.description, i.status, i.assignee,
                    i.created_at, i.updated_at,
                    (SELECT MAX(e.created_at) FROM events e
                     WHERE e.issue_id = i.id AND e.action IN ('done', 'closed')
                     AND NOT EXISTS (
                         SELECT 1 FROM events e2
                         WHERE e2.issue_id = e.issue_id
                         AND e2.action = 'reopened'
                         AND e2.created_at > e.created_at
                     )) as closed_at
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
            params_vec.push(l);
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY i.created_at DESC");

        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let issues = stmt
            .query_map(params_refs.as_slice(), |row| self.row_to_issue(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Issues { issues })
    }

    fn search_issues(&self, query: &str) -> Result<QueryResult, String> {
        let escaped_query = query.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped_query);

        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT i.id, i.type, i.title, i.description, i.status, i.assignee,
                    i.created_at, i.updated_at,
                    (SELECT MAX(e.created_at) FROM events e
                     WHERE e.issue_id = i.id AND e.action IN ('done', 'closed')
                     AND NOT EXISTS (
                         SELECT 1 FROM events e2
                         WHERE e2.issue_id = e.issue_id
                         AND e2.action = 'reopened'
                         AND e2.created_at > e.created_at
                     )) as closed_at
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
            )
            .map_err(|e| e.to_string())?;

        let issues = stmt
            .query_map([&pattern], |row| self.row_to_issue(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Issues { issues })
    }

    fn get_blocked_issue_ids(&self) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
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
            )
            .map_err(|e| e.to_string())?;

        let ids = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Ids { ids })
    }

    fn get_labels(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT label FROM labels WHERE issue_id = ?1 ORDER BY label")
            .map_err(|e| e.to_string())?;

        let labels = stmt
            .query_map([id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Labels { labels })
    }

    fn get_labels_batch(&self, ids: &[String]) -> Result<QueryResult, String> {
        let mut labels_map: HashMap<String, Vec<String>> = HashMap::new();

        for id in ids {
            let mut stmt = self
                .conn
                .prepare("SELECT label FROM labels WHERE issue_id = ?1 ORDER BY label")
                .map_err(|e| e.to_string())?;

            let labels: Vec<String> = stmt
                .query_map([id], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            labels_map.insert(id.clone(), labels);
        }

        Ok(QueryResult::LabelsBatch { labels: labels_map })
    }

    fn get_notes(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, issue_id, status, content, created_at
             FROM notes WHERE issue_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let notes = stmt
            .query_map([id], |row| {
                let status_str: String = row.get(2)?;
                let created_str: String = row.get(4)?;
                Ok(Note {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    status: parse_status(&status_str),
                    content: row.get(3)?,
                    created_at: parse_timestamp(&created_str),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Notes { notes })
    }

    fn get_events(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, issue_id, action, old_value, new_value, reason, created_at
             FROM events WHERE issue_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let events = stmt
            .query_map([id], |row| self.row_to_event(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Events { events })
    }

    fn get_all_events(&self, limit: Option<usize>) -> Result<QueryResult, String> {
        let sql = match limit {
            Some(n) => format!(
                "SELECT id, issue_id, action, old_value, new_value, reason, created_at
                 FROM events ORDER BY created_at DESC LIMIT {}",
                n
            ),
            None => "SELECT id, issue_id, action, old_value, new_value, reason, created_at
                 FROM events ORDER BY created_at DESC"
                .to_string(),
        };

        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;

        let events = stmt
            .query_map([], |row| self.row_to_event(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Events { events })
    }

    fn get_deps_from(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT from_id, to_id, rel, created_at FROM deps
             WHERE from_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let deps = stmt
            .query_map([id], |row| self.row_to_dependency(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Dependencies { deps })
    }

    fn get_blockers(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT from_id, to_id, rel, created_at FROM deps
             WHERE to_id = ?1 AND rel = 'blocks' ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let deps = stmt
            .query_map([id], |row| self.row_to_dependency(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Dependencies { deps })
    }

    fn get_blocking(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT from_id, to_id, rel, created_at FROM deps
             WHERE from_id = ?1 AND rel = 'blocks' ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let deps = stmt
            .query_map([id], |row| self.row_to_dependency(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Dependencies { deps })
    }

    fn get_tracked(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT from_id, to_id, rel, created_at FROM deps
             WHERE from_id = ?1 AND rel = 'tracks' ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let deps = stmt
            .query_map([id], |row| self.row_to_dependency(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Dependencies { deps })
    }

    fn get_tracking(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT from_id, to_id, rel, created_at FROM deps
             WHERE to_id = ?1 AND rel = 'tracks' ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let deps = stmt
            .query_map([id], |row| self.row_to_dependency(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Dependencies { deps })
    }

    fn get_transitive_blockers(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "WITH RECURSIVE blockers(id) AS (
                SELECT from_id FROM deps WHERE to_id = ?1 AND rel = 'blocks'
                UNION
                SELECT d.from_id FROM deps d
                JOIN blockers b ON d.to_id = b.id
                WHERE d.rel = 'blocks'
            )
            SELECT b.id as from_id, ?1 as to_id, 'blocks' as rel, i.created_at
            FROM blockers b
            JOIN issues i ON i.id = b.id
            WHERE i.status IN ('todo', 'in_progress')",
            )
            .map_err(|e| e.to_string())?;

        let deps = stmt
            .query_map([id], |row| self.row_to_dependency(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Dependencies { deps })
    }

    fn get_links(&self, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, issue_id, link_type, url, external_id, rel, created_at
             FROM links WHERE issue_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let links = stmt
            .query_map([id], |row| self.row_to_link(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Links { links })
    }

    fn get_link_by_url(&self, id: &str, url: &str) -> Result<QueryResult, String> {
        let link = self
            .conn
            .query_row(
                "SELECT id, issue_id, link_type, url, external_id, rel, created_at
             FROM links WHERE issue_id = ?1 AND url = ?2",
                [id, url],
                |row| self.row_to_link(row),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Link { link })
    }

    fn list_prefixes(&self) -> Result<QueryResult, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT prefix, issue_count, created_at FROM prefixes ORDER BY issue_count DESC",
            )
            .map_err(|e| e.to_string())?;

        let prefixes = stmt
            .query_map([], |row| {
                let created_str: String = row.get(2)?;
                Ok(PrefixInfo {
                    prefix: row.get(0)?,
                    issue_count: row.get(1)?,
                    created_at: parse_timestamp(&created_str),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(QueryResult::Prefixes { prefixes })
    }

    // ========================================================================
    // Mutation implementations
    // ========================================================================

    fn create_issue(&self, issue: &Issue) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "INSERT INTO issues (id, type, title, description, status, assignee, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    issue.id,
                    issue.issue_type.as_str(),
                    issue.title,
                    issue.description,
                    issue.status.as_str(),
                    issue.assignee,
                    issue.created_at.to_rfc3339(),
                    issue.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn update_issue_status(&self, id: &str, status: Status) -> Result<MutateResult, String> {
        let affected = self
            .conn
            .execute(
                "UPDATE issues SET status = ?1, updated_at = ?2 WHERE id = ?3",
                params![status.as_str(), Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("issue not found: {}", id));
        }
        Ok(MutateResult::Ok)
    }

    fn update_issue_title(&self, id: &str, title: &str) -> Result<MutateResult, String> {
        let affected = self
            .conn
            .execute(
                "UPDATE issues SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![title, Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("issue not found: {}", id));
        }
        Ok(MutateResult::Ok)
    }

    fn update_issue_description(
        &self,
        id: &str,
        description: &str,
    ) -> Result<MutateResult, String> {
        let affected = self
            .conn
            .execute(
                "UPDATE issues SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![description, Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("issue not found: {}", id));
        }
        Ok(MutateResult::Ok)
    }

    fn update_issue_type(&self, id: &str, issue_type: IssueType) -> Result<MutateResult, String> {
        let affected = self
            .conn
            .execute(
                "UPDATE issues SET type = ?1, updated_at = ?2 WHERE id = ?3",
                params![issue_type.as_str(), Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("issue not found: {}", id));
        }
        Ok(MutateResult::Ok)
    }

    fn set_assignee(&self, id: &str, assignee: &str) -> Result<MutateResult, String> {
        let affected = self
            .conn
            .execute(
                "UPDATE issues SET assignee = ?1, updated_at = ?2 WHERE id = ?3",
                params![assignee, Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("issue not found: {}", id));
        }
        Ok(MutateResult::Ok)
    }

    fn clear_assignee(&self, id: &str) -> Result<MutateResult, String> {
        let affected = self
            .conn
            .execute(
                "UPDATE issues SET assignee = NULL, updated_at = ?1 WHERE id = ?2",
                params![Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("issue not found: {}", id));
        }
        Ok(MutateResult::Ok)
    }

    fn add_label(&self, id: &str, label: &str) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO labels (issue_id, label) VALUES (?1, ?2)",
                params![id, label],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn remove_label(&self, id: &str, label: &str) -> Result<MutateResult, String> {
        let affected = self
            .conn
            .execute(
                "DELETE FROM labels WHERE issue_id = ?1 AND label = ?2",
                params![id, label],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::LabelRemoved {
            removed: affected > 0,
        })
    }

    fn add_note(&self, id: &str, status: Status, content: &str) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "INSERT INTO notes (issue_id, status, content, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, status.as_str(), content, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn log_event(&self, event: &Event) -> Result<MutateResult, String> {
        self.conn
            .execute(
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
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn add_dependency(
        &self,
        from_id: &str,
        to_id: &str,
        relation: Relation,
    ) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO deps (from_id, to_id, rel, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![from_id, to_id, relation.as_str(), Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn remove_dependency(
        &self,
        from_id: &str,
        to_id: &str,
        relation: Relation,
    ) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "DELETE FROM deps WHERE from_id = ?1 AND to_id = ?2 AND rel = ?3",
                params![from_id, to_id, relation.as_str()],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn add_link(
        &self,
        id: &str,
        link_type: Option<LinkType>,
        url: Option<String>,
        external_id: Option<String>,
        rel: Option<LinkRel>,
    ) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "INSERT INTO links (issue_id, link_type, url, external_id, rel, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    link_type.map(|t| t.as_str().to_string()),
                    url,
                    external_id,
                    rel.map(|r| r.as_str().to_string()),
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn remove_link(&self, id: &str, url: &str) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "DELETE FROM links WHERE issue_id = ?1 AND url = ?2",
                params![id, url],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn ensure_prefix(&self, prefix: &str) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO prefixes (prefix, created_at, issue_count) VALUES (?1, ?2, 0)",
                params![prefix, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    fn increment_prefix_count(&self, prefix: &str) -> Result<MutateResult, String> {
        self.conn
            .execute(
                "UPDATE prefixes SET issue_count = issue_count + 1 WHERE prefix = ?1",
                params![prefix],
            )
            .map_err(|e| e.to_string())?;
        Ok(MutateResult::Ok)
    }

    // ========================================================================
    // Row conversion helpers
    // ========================================================================

    fn row_to_issue(&self, row: &rusqlite::Row) -> rusqlite::Result<Issue> {
        let type_str: String = row.get(1)?;
        let status_str: String = row.get(4)?;
        let created_str: String = row.get(6)?;
        let updated_str: String = row.get(7)?;
        let closed_str: Option<String> = row.get(8)?;

        Ok(Issue {
            id: row.get(0)?,
            issue_type: parse_issue_type(&type_str),
            title: row.get(2)?,
            description: row.get(3)?,
            status: parse_status(&status_str),
            assignee: row.get(5)?,
            created_at: parse_timestamp(&created_str),
            updated_at: parse_timestamp(&updated_str),
            closed_at: closed_str.map(|s| parse_timestamp(&s)),
        })
    }

    fn row_to_event(&self, row: &rusqlite::Row) -> rusqlite::Result<Event> {
        let action_str: String = row.get(2)?;
        let created_str: String = row.get(6)?;

        Ok(Event {
            id: row.get(0)?,
            issue_id: row.get(1)?,
            action: parse_action(&action_str),
            old_value: row.get(3)?,
            new_value: row.get(4)?,
            reason: row.get(5)?,
            created_at: parse_timestamp(&created_str),
        })
    }

    fn row_to_dependency(&self, row: &rusqlite::Row) -> rusqlite::Result<Dependency> {
        let rel_str: String = row.get(2)?;
        let created_str: String = row.get(3)?;

        Ok(Dependency {
            from_id: row.get(0)?,
            to_id: row.get(1)?,
            relation: parse_relation(&rel_str),
            created_at: parse_timestamp(&created_str),
        })
    }

    fn row_to_link(&self, row: &rusqlite::Row) -> rusqlite::Result<Link> {
        let type_str: Option<String> = row.get(2)?;
        let rel_str: Option<String> = row.get(5)?;
        let created_str: String = row.get(6)?;

        Ok(Link {
            id: row.get(0)?,
            issue_id: row.get(1)?,
            link_type: type_str.and_then(|s| parse_link_type(&s)),
            url: row.get(3)?,
            external_id: row.get(4)?,
            rel: rel_str.and_then(|s| parse_link_rel(&s)),
            created_at: parse_timestamp(&created_str),
        })
    }
}

// ============================================================================
// Parsing helpers
// ============================================================================

fn parse_timestamp(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_issue_type(s: &str) -> IssueType {
    match s {
        "feature" => IssueType::Feature,
        "task" => IssueType::Task,
        "bug" => IssueType::Bug,
        "chore" => IssueType::Chore,
        "idea" => IssueType::Idea,
        "epic" => IssueType::Epic,
        _ => IssueType::Task,
    }
}

fn parse_status(s: &str) -> Status {
    match s {
        "todo" => Status::Todo,
        "in_progress" => Status::InProgress,
        "done" => Status::Done,
        "closed" => Status::Closed,
        _ => Status::Todo,
    }
}

fn parse_action(s: &str) -> Action {
    match s {
        "created" => Action::Created,
        "edited" => Action::Edited,
        "started" => Action::Started,
        "stopped" => Action::Stopped,
        "done" => Action::Done,
        "closed" => Action::Closed,
        "reopened" => Action::Reopened,
        "labeled" => Action::Labeled,
        "unlabeled" => Action::Unlabeled,
        "related" => Action::Related,
        "unrelated" => Action::Unrelated,
        "linked" => Action::Linked,
        "unlinked" => Action::Unlinked,
        "noted" => Action::Noted,
        "unblocked" => Action::Unblocked,
        "assigned" => Action::Assigned,
        "unassigned" => Action::Unassigned,
        _ => Action::Created,
    }
}

fn parse_relation(s: &str) -> Relation {
    match s {
        "blocks" => Relation::Blocks,
        "tracked-by" => Relation::TrackedBy,
        "tracks" => Relation::Tracks,
        _ => Relation::Blocks,
    }
}

fn parse_link_type(s: &str) -> Option<LinkType> {
    match s {
        "github" => Some(LinkType::Github),
        "jira" => Some(LinkType::Jira),
        "gitlab" => Some(LinkType::Gitlab),
        "confluence" => Some(LinkType::Confluence),
        _ => None,
    }
}

fn parse_link_rel(s: &str) -> Option<LinkRel> {
    match s {
        "import" => Some(LinkRel::Import),
        "blocks" => Some(LinkRel::Blocks),
        "tracks" => Some(LinkRel::Tracks),
        "tracked-by" => Some(LinkRel::TrackedBy),
        _ => None,
    }
}
