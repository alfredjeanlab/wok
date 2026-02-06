// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database operations for the daemon.
//!
//! Wraps [`wk_core::Database`] and dispatches IPC query/mutation operations.

use std::path::Path;

use chrono::{DateTime, Utc};

use crate::ipc::{
    Dependency, DependencyRef, Issue, Link, MutateOp, MutateResult, QueryOp, QueryResult, Relation,
};

/// Database wrapper for the daemon.
///
/// Delegates all operations to [`wk_core::Database`] and converts between
/// IPC types and core types at the boundary.
pub struct Database {
    inner: wk_core::Database,
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

        let inner = wk_core::Database::open(path).map_err(|e| e.to_string())?;
        Ok(Database { inner })
    }

    /// Execute a query operation and return the result.
    pub fn execute_query(&self, op: QueryOp) -> Result<QueryResult, String> {
        match op {
            QueryOp::ResolveId { partial_id } => {
                let id = self
                    .inner
                    .resolve_id(&partial_id)
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::ResolvedId { id })
            }
            QueryOp::IssueExists { id } => {
                let value = self.inner.issue_exists(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Bool { value })
            }
            QueryOp::GetIssue { id } => {
                let issue: Issue = self.inner.get_issue(&id).map_err(|e| e.to_string())?.into();
                Ok(QueryResult::Issue { issue })
            }
            QueryOp::ListIssues {
                status,
                issue_type,
                label,
            } => {
                let issues: Vec<Issue> = self
                    .inner
                    .list_issues(status, issue_type, label.as_deref())
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                Ok(QueryResult::Issues { issues })
            }
            QueryOp::SearchIssues { query } => {
                let issues: Vec<Issue> = self
                    .inner
                    .search_issues(&query)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                Ok(QueryResult::Issues { issues })
            }
            QueryOp::GetBlockedIssueIds => {
                let ids = self
                    .inner
                    .get_blocked_issue_ids()
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetLabels { id } => {
                let labels = self.inner.get_labels(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Labels { labels })
            }
            QueryOp::GetLabelsBatch { ids } => {
                let refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
                let labels = self
                    .inner
                    .get_labels_batch(&refs)
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::LabelsBatch { labels })
            }
            QueryOp::GetNotes { id } => {
                let notes = self.inner.get_notes(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Notes { notes })
            }
            QueryOp::GetEvents { id } => {
                let events = self.inner.get_events(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetAllEvents { limit } => {
                let events = self
                    .inner
                    .get_recent_events(limit.unwrap_or(i64::MAX as usize))
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetDepsFrom { id } => {
                let deps = self.inner.get_deps_from(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetBlockers { id } => self.query_deps(
                "SELECT from_id, to_id, rel, created_at FROM deps
                 WHERE to_id = ?1 AND rel = 'blocks' ORDER BY created_at DESC",
                &id,
            ),
            QueryOp::GetBlocking { id } => self.query_deps(
                "SELECT from_id, to_id, rel, created_at FROM deps
                 WHERE from_id = ?1 AND rel = 'blocks' ORDER BY created_at DESC",
                &id,
            ),
            QueryOp::GetTracked { id } => self.query_deps(
                "SELECT from_id, to_id, rel, created_at FROM deps
                 WHERE from_id = ?1 AND rel = 'tracks' ORDER BY created_at DESC",
                &id,
            ),
            QueryOp::GetTracking { id } => self.query_deps(
                "SELECT from_id, to_id, rel, created_at FROM deps
                 WHERE to_id = ?1 AND rel = 'tracks' ORDER BY created_at DESC",
                &id,
            ),
            QueryOp::GetTransitiveBlockers { id } => {
                let mut stmt = self
                    .inner
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
                    .query_map([&id], row_to_dependency)
                    .map_err(|e| e.to_string())?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?;

                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetLinks { id } => {
                let links = self.inner.get_links(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Links { links })
            }
            QueryOp::GetLinkByUrl { id, url } => {
                let link = self
                    .inner
                    .get_link_by_url(&id, &url)
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::Link { link })
            }
            QueryOp::ListPrefixes => {
                let prefixes = self.inner.list_prefixes().map_err(|e| e.to_string())?;
                Ok(QueryResult::Prefixes { prefixes })
            }
        }
    }

    /// Execute a mutation operation and return the result.
    pub fn execute_mutate(&self, op: MutateOp) -> Result<MutateResult, String> {
        match op {
            MutateOp::CreateIssue { issue } => {
                let core_issue: wk_core::Issue = issue.into();
                self.inner
                    .create_issue(&core_issue)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueStatus { id, status } => {
                self.inner
                    .update_issue_status(&id, status)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueTitle { id, title } => {
                self.inner
                    .update_issue_title(&id, &title)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueDescription { id, description } => {
                self.inner
                    .update_issue_description(&id, &description)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueType { id, issue_type } => {
                self.inner
                    .update_issue_type(&id, issue_type)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::SetAssignee { id, assignee } => {
                self.inner
                    .set_assignee(&id, &assignee)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::ClearAssignee { id } => {
                self.inner.clear_assignee(&id).map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddLabel { id, label } => {
                self.inner
                    .add_label(&id, &label)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLabel { id, label } => {
                let removed = self
                    .inner
                    .remove_label(&id, &label)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::LabelRemoved { removed })
            }
            MutateOp::AddNote {
                id,
                status,
                content,
            } => {
                self.inner
                    .add_note(&id, status, &content)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::LogEvent { event } => {
                self.inner.log_event(&event).map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddDependency(DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                self.inner
                    .add_dependency(&from_id, &to_id, relation)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveDependency(DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                self.inner
                    .remove_dependency(&from_id, &to_id, relation)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddLink {
                id,
                link_type,
                url,
                external_id,
                rel,
            } => {
                let mut link = Link::new(id);
                link.link_type = link_type;
                link.url = url;
                link.external_id = external_id;
                link.rel = rel;
                self.inner.add_link(&link).map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLink { id, url } => {
                let link = self
                    .inner
                    .get_link_by_url(&id, &url)
                    .map_err(|e| e.to_string())?;
                if let Some(link) = link {
                    self.inner.remove_link(link.id).map_err(|e| e.to_string())?;
                }
                Ok(MutateResult::Ok)
            }
            MutateOp::EnsurePrefix { prefix } => {
                self.inner
                    .ensure_prefix(&prefix)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::IncrementPrefixCount { prefix } => {
                self.inner
                    .increment_prefix_count(&prefix)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
        }
    }

    /// Query dependencies using raw SQL (for IPC responses needing full Dependency objects).
    fn query_deps(&self, sql: &str, id: &str) -> Result<QueryResult, String> {
        let mut stmt = self.inner.conn.prepare(sql).map_err(|e| e.to_string())?;
        let deps = stmt
            .query_map([id], row_to_dependency)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(QueryResult::Dependencies { deps })
    }
}

fn row_to_dependency(row: &rusqlite::Row) -> rusqlite::Result<Dependency> {
    let rel_str: String = row.get(2)?;
    let created_str: String = row.get(3)?;
    Ok(Dependency {
        from_id: row.get(0)?,
        to_id: row.get(1)?,
        relation: parse_relation(&rel_str),
        created_at: parse_timestamp(&created_str),
    })
}

fn parse_timestamp(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_relation(s: &str) -> Relation {
    match s {
        "blocks" => Relation::Blocks,
        "tracked-by" => Relation::TrackedBy,
        "tracks" => Relation::Tracks,
        _ => Relation::Blocks,
    }
}
