// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database operations for the daemon.
//!
//! Thin adapter that delegates all operations to [`wk_core::Database`],
//! converting between IPC protocol types and core types as needed.

use std::path::Path;

use crate::ipc::{DependencyRef, MutateOp, MutateResult, QueryOp, QueryResult};

/// Database wrapper for the daemon.
///
/// Wraps [`wk_core::Database`] and adapts IPC operations to core method calls.
pub struct Database {
    core: wk_core::Database,
}

impl Database {
    /// Open or create a database at the given path.
    pub fn open(path: &Path) -> Result<Self, String> {
        let core = wk_core::Database::open(path).map_err(|e| format!("{}", e))?;
        Ok(Database { core })
    }

    /// Execute a query operation and return the result.
    pub fn execute_query(&self, op: QueryOp) -> Result<QueryResult, String> {
        self.dispatch_query(op).map_err(|e| format!("{}", e))
    }

    /// Execute a mutation operation and return the result.
    pub fn execute_mutate(&mut self, op: MutateOp) -> Result<MutateResult, String> {
        self.dispatch_mutate(op).map_err(|e| format!("{}", e))
    }

    fn dispatch_query(&self, op: QueryOp) -> wk_core::Result<QueryResult> {
        match op {
            QueryOp::ResolveId { partial_id } => {
                let id = self.core.resolve_id(&partial_id)?;
                Ok(QueryResult::ResolvedId { id })
            }
            QueryOp::IssueExists { id } => {
                let value = self.core.issue_exists(&id)?;
                Ok(QueryResult::Bool { value })
            }
            QueryOp::GetIssue { id } => {
                let issue = self.core.get_issue(&id)?;
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
                    .list_issues(status, issue_type, label.as_deref())?;
                Ok(QueryResult::Issues {
                    issues: issues.into_iter().map(Into::into).collect(),
                })
            }
            QueryOp::SearchIssues { query } => {
                let issues = self.core.search_issues(&query)?;
                Ok(QueryResult::Issues {
                    issues: issues.into_iter().map(Into::into).collect(),
                })
            }
            QueryOp::GetBlockedIssueIds => {
                let ids = self.core.get_blocked_issue_ids()?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetLabels { id } => {
                let labels = self.core.get_labels(&id)?;
                Ok(QueryResult::Labels { labels })
            }
            QueryOp::GetLabelsBatch { ids } => {
                let refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
                let labels = self.core.get_labels_batch(&refs)?;
                Ok(QueryResult::LabelsBatch { labels })
            }
            QueryOp::GetNotes { id } => {
                let notes = self.core.get_notes(&id)?;
                Ok(QueryResult::Notes { notes })
            }
            QueryOp::GetEvents { id } => {
                let events = self.core.get_events(&id)?;
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetAllEvents { limit } => {
                let events = self.core.get_recent_events(limit.unwrap_or(usize::MAX))?;
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetDepsFrom { id } => {
                let deps = self.core.get_deps_from(&id)?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetBlockers { id } => {
                let ids = self.core.get_blockers(&id)?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetBlocking { id } => {
                let ids = self.core.get_blocking(&id)?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetTracked { id } => {
                let ids = self.core.get_tracked(&id)?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetTracking { id } => {
                let ids = self.core.get_tracking(&id)?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetTransitiveBlockers { id } => {
                let ids = self.core.get_transitive_blockers(&id)?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetLinks { id } => {
                let links = self.core.get_links(&id)?;
                Ok(QueryResult::Links { links })
            }
            QueryOp::GetLinkByUrl { id, url } => {
                let link = self.core.get_link_by_url(&id, &url)?;
                Ok(QueryResult::Link { link })
            }
            QueryOp::ListPrefixes => {
                let prefixes = self.core.list_prefixes()?;
                Ok(QueryResult::Prefixes { prefixes })
            }
        }
    }

    fn dispatch_mutate(&mut self, op: MutateOp) -> wk_core::Result<MutateResult> {
        match op {
            MutateOp::CreateIssue { issue } => {
                let core_issue: wk_core::Issue = issue.into();
                self.core.create_issue(&core_issue)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueStatus { id, status } => {
                self.core.update_issue_status(&id, status)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueTitle { id, title } => {
                self.core.update_issue_title(&id, &title)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueDescription { id, description } => {
                self.core.update_issue_description(&id, &description)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueType { id, issue_type } => {
                self.core.update_issue_type(&id, issue_type)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::SetAssignee { id, assignee } => {
                self.core.set_assignee(&id, &assignee)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::ClearAssignee { id } => {
                self.core.clear_assignee(&id)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddLabel { id, label } => {
                self.core.add_label(&id, &label)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLabel { id, label } => {
                let removed = self.core.remove_label(&id, &label)?;
                Ok(MutateResult::LabelRemoved { removed })
            }
            MutateOp::AddNote {
                id,
                status,
                content,
            } => {
                self.core.add_note(&id, status, &content)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::LogEvent { event } => {
                self.core.log_event(&event)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddDependency(DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                self.core.add_dependency(&from_id, &to_id, relation)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveDependency(DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                self.core.remove_dependency(&from_id, &to_id, relation)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddLink {
                id,
                link_type,
                url,
                external_id,
                rel,
            } => {
                let mut link = wk_core::Link::new(id);
                if let Some(lt) = link_type {
                    link = link.with_type(lt);
                }
                if let Some(u) = url {
                    link = link.with_url(u);
                }
                if let Some(eid) = external_id {
                    link = link.with_external_id(eid);
                }
                if let Some(r) = rel {
                    link = link.with_rel(r);
                }
                self.core.add_link(&link)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLink { id, url } => {
                if let Some(link) = self.core.get_link_by_url(&id, &url)? {
                    self.core.remove_link(link.id)?;
                }
                Ok(MutateResult::Ok)
            }
            MutateOp::EnsurePrefix { prefix } => {
                self.core.ensure_prefix(&prefix)?;
                Ok(MutateResult::Ok)
            }
            MutateOp::IncrementPrefixCount { prefix } => {
                self.core.increment_prefix_count(&prefix)?;
                Ok(MutateResult::Ok)
            }
        }
    }
}
