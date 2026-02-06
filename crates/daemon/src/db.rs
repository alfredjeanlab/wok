// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database adapter for the daemon.
//!
//! Thin wrapper over [`wk_core::Database`] that dispatches IPC query and
//! mutation operations, converting types at the boundary.

use std::path::Path;

use chrono::Utc;
use rusqlite::params;

use crate::ipc::{MutateOp, MutateResult, QueryOp, QueryResult};

/// Database adapter that delegates to core.
pub struct Database {
    core: wk_core::Database,
}

impl Database {
    /// Open or create a database at the given path.
    pub fn open(path: &Path) -> Result<Self, String> {
        let core = wk_core::Database::open(path).map_err(|e| e.to_string())?;
        Ok(Database { core })
    }

    /// Execute a query operation and return the result.
    pub fn execute_query(&self, op: QueryOp) -> Result<QueryResult, String> {
        match op {
            QueryOp::ResolveId { partial_id } => {
                let id = self
                    .core
                    .resolve_id(&partial_id)
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::ResolvedId { id })
            }
            QueryOp::IssueExists { id } => {
                let value = self.core.issue_exists(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Bool { value })
            }
            QueryOp::GetIssue { id } => {
                let issue = self.core.get_issue(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Issue {
                    issue: issue.into(),
                })
            }
            QueryOp::ListIssues {
                status,
                issue_type,
                label,
            } => {
                let issues = self
                    .core
                    .list_issues(status, issue_type, label.as_deref())
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::Issues {
                    issues: issues.into_iter().map(|i| i.into()).collect(),
                })
            }
            QueryOp::SearchIssues { query } => {
                let issues = self.core.search_issues(&query).map_err(|e| e.to_string())?;
                Ok(QueryResult::Issues {
                    issues: issues.into_iter().map(|i| i.into()).collect(),
                })
            }
            QueryOp::GetBlockedIssueIds => {
                let ids = self
                    .core
                    .get_blocked_issue_ids()
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetLabels { id } => {
                let labels = self.core.get_labels(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Labels { labels })
            }
            QueryOp::GetLabelsBatch { ids } => {
                let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
                let labels = self
                    .core
                    .get_labels_batch(&id_refs)
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::LabelsBatch { labels })
            }
            QueryOp::GetNotes { id } => {
                let notes = self.core.get_notes(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Notes { notes })
            }
            QueryOp::GetEvents { id } => {
                let events = self.core.get_events(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetAllEvents { limit } => {
                let events = match limit {
                    Some(n) => self.core.get_recent_events(n),
                    None => self.core.get_recent_events(usize::MAX),
                }
                .map_err(|e| e.to_string())?;
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetDepsFrom { id } => {
                let deps = self.core.get_deps_from(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetBlockers { id } => {
                let deps = self.query_deps(
                    "SELECT from_id, to_id, rel, created_at FROM deps
                     WHERE to_id = ?1 AND rel = 'blocks' ORDER BY created_at DESC",
                    &id,
                )?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetBlocking { id } => {
                let deps = self.query_deps(
                    "SELECT from_id, to_id, rel, created_at FROM deps
                     WHERE from_id = ?1 AND rel = 'blocks' ORDER BY created_at DESC",
                    &id,
                )?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetTracked { id } => {
                let deps = self.query_deps(
                    "SELECT from_id, to_id, rel, created_at FROM deps
                     WHERE from_id = ?1 AND rel = 'tracks' ORDER BY created_at DESC",
                    &id,
                )?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetTracking { id } => {
                let deps = self.query_deps(
                    "SELECT from_id, to_id, rel, created_at FROM deps
                     WHERE to_id = ?1 AND rel = 'tracks' ORDER BY created_at DESC",
                    &id,
                )?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetTransitiveBlockers { id } => {
                let deps = self.query_deps(
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
                    &id,
                )?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetLinks { id } => {
                let links = self.core.get_links(&id).map_err(|e| e.to_string())?;
                Ok(QueryResult::Links { links })
            }
            QueryOp::GetLinkByUrl { id, url } => {
                let link = self
                    .core
                    .get_link_by_url(&id, &url)
                    .map_err(|e| e.to_string())?;
                Ok(QueryResult::Link { link })
            }
            QueryOp::ListPrefixes => {
                let prefixes = self.core.list_prefixes().map_err(|e| e.to_string())?;
                Ok(QueryResult::Prefixes { prefixes })
            }
        }
    }

    /// Execute a mutation operation and return the result.
    pub fn execute_mutate(&self, op: MutateOp) -> Result<MutateResult, String> {
        match op {
            MutateOp::CreateIssue { issue } => {
                let core_issue: wk_core::Issue = issue.into();
                self.core
                    .create_issue(&core_issue)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueStatus { id, status } => {
                let now = Utc::now();
                let closed_at = if status.is_terminal() {
                    Some(now.to_rfc3339())
                } else {
                    None
                };
                let affected = self
                    .core
                    .conn
                    .execute(
                        "UPDATE issues SET status = ?1, updated_at = ?2, closed_at = ?3 WHERE id = ?4",
                        params![status.as_str(), now.to_rfc3339(), closed_at, id],
                    )
                    .map_err(|e| e.to_string())?;
                if affected == 0 {
                    return Err(format!("issue not found: {}", id));
                }
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueTitle { id, title } => {
                let affected = self
                    .core
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
            MutateOp::UpdateIssueDescription { id, description } => {
                let affected = self
                    .core
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
            MutateOp::UpdateIssueType { id, issue_type } => {
                let affected = self
                    .core
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
            MutateOp::SetAssignee { id, assignee } => {
                let affected = self
                    .core
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
            MutateOp::ClearAssignee { id } => {
                let affected = self
                    .core
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
            MutateOp::AddLabel { id, label } => {
                self.core
                    .add_label(&id, &label)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLabel { id, label } => {
                let removed = self
                    .core
                    .remove_label(&id, &label)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::LabelRemoved { removed })
            }
            MutateOp::AddNote {
                id,
                status,
                content,
            } => {
                self.core
                    .add_note(&id, status, &content)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::LogEvent { event } => {
                self.core.log_event(&event).map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddDependency(crate::ipc::DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                self.core
                    .add_dependency(&from_id, &to_id, relation)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveDependency(crate::ipc::DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                self.core
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
                let core_link = wk_core::Link {
                    id: 0,
                    issue_id: id,
                    link_type,
                    url,
                    external_id,
                    rel,
                    created_at: Utc::now(),
                };
                self.core.add_link(&core_link).map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLink { id, url } => {
                self.core
                    .conn
                    .execute(
                        "DELETE FROM links WHERE issue_id = ?1 AND url = ?2",
                        params![id, url],
                    )
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::EnsurePrefix { prefix } => {
                self.core
                    .ensure_prefix(&prefix)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
            MutateOp::IncrementPrefixCount { prefix } => {
                self.core
                    .increment_prefix_count(&prefix)
                    .map_err(|e| e.to_string())?;
                Ok(MutateResult::Ok)
            }
        }
    }

    /// Helper: query dependency rows and convert to IPC Dependency.
    fn query_deps(&self, sql: &str, id: &str) -> Result<Vec<crate::ipc::Dependency>, String> {
        let mut stmt = self.core.conn.prepare(sql).map_err(|e| e.to_string())?;
        let deps = stmt
            .query_map([id], |row| {
                let rel_str: String = row.get(2)?;
                let created_str: String = row.get(3)?;
                Ok(crate::ipc::Dependency {
                    from_id: row.get(0)?,
                    to_id: row.get(1)?,
                    relation: rel_str.parse().unwrap_or(crate::ipc::Relation::Blocks),
                    created_at: wk_core::db::parse_timestamp(&created_str, "created_at")?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(deps)
    }
}
