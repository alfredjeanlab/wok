// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database adapter for the daemon.
//!
//! Thin adapter dispatching IPC [`QueryOp`]/[`MutateOp`] to the core
//! [`Database`](wk_core::Database) with IPC type conversions.

use std::path::Path;

use chrono::Utc;

use crate::ipc::{
    DependencyRef, Issue as IpcIssue, Link, MutateOp, MutateResult, QueryOp, QueryResult, Relation,
};

/// Database adapter wrapping core Database for IPC operations.
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
                let id = q(self.core.resolve_id(&partial_id))?;
                Ok(QueryResult::ResolvedId { id })
            }
            QueryOp::IssueExists { id } => {
                let value = q(self.core.issue_exists(&id))?;
                Ok(QueryResult::Bool { value })
            }
            QueryOp::GetIssue { id } => {
                let issue = q(self.core.get_issue(&id))?;
                Ok(QueryResult::Issue {
                    issue: core_to_ipc(issue),
                })
            }
            QueryOp::ListIssues {
                status,
                issue_type,
                label,
            } => {
                let issues = q(self.core.list_issues(status, issue_type, label.as_deref()))?;
                Ok(QueryResult::Issues {
                    issues: issues.into_iter().map(core_to_ipc).collect(),
                })
            }
            QueryOp::SearchIssues { query } => {
                let issues = q(self.core.search_issues(&query))?;
                Ok(QueryResult::Issues {
                    issues: issues.into_iter().map(core_to_ipc).collect(),
                })
            }
            QueryOp::GetBlockedIssueIds => {
                let ids = q(self.core.get_blocked_issue_ids())?;
                Ok(QueryResult::Ids { ids })
            }
            QueryOp::GetLabels { id } => {
                let labels = q(self.core.get_labels(&id))?;
                Ok(QueryResult::Labels { labels })
            }
            QueryOp::GetLabelsBatch { ids } => {
                let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
                let labels = q(self.core.get_labels_batch(&refs))?;
                Ok(QueryResult::LabelsBatch { labels })
            }
            QueryOp::GetNotes { id } => {
                let mut notes = q(self.core.get_notes(&id))?;
                notes.reverse();
                Ok(QueryResult::Notes { notes })
            }
            QueryOp::GetEvents { id } => {
                let mut events = q(self.core.get_events(&id))?;
                events.reverse();
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetAllEvents { limit } => {
                let events = q(self.core.get_recent_events(limit.unwrap_or(usize::MAX)))?;
                Ok(QueryResult::Events { events })
            }
            QueryOp::GetDepsFrom { id } => {
                let deps = q(self.core.get_deps_from(&id))?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetBlockers { id } => {
                let mut deps = q(self.core.get_deps_to(&id))?;
                deps.retain(|d| d.relation == Relation::Blocks);
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetBlocking { id } => {
                let mut deps = q(self.core.get_deps_from(&id))?;
                deps.retain(|d| d.relation == Relation::Blocks);
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetTracked { id } => {
                let mut deps = q(self.core.get_deps_from(&id))?;
                deps.retain(|d| d.relation == Relation::Tracks);
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetTracking { id } => {
                let mut deps = q(self.core.get_deps_to(&id))?;
                deps.retain(|d| d.relation == Relation::Tracks);
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetTransitiveBlockers { id } => {
                let deps = q(self.core.get_transitive_blocker_deps(&id))?;
                Ok(QueryResult::Dependencies { deps })
            }
            QueryOp::GetLinks { id } => {
                let mut links = q(self.core.get_links(&id))?;
                links.reverse();
                Ok(QueryResult::Links { links })
            }
            QueryOp::GetLinkByUrl { id, url } => {
                let link = q(self.core.get_link_by_url(&id, &url))?;
                Ok(QueryResult::Link { link })
            }
            QueryOp::ListPrefixes => {
                let prefixes = q(self.core.list_prefixes())?;
                Ok(QueryResult::Prefixes { prefixes })
            }
        }
    }

    /// Execute a mutation operation and return the result.
    pub fn execute_mutate(&mut self, op: MutateOp) -> Result<MutateResult, String> {
        match op {
            MutateOp::CreateIssue { issue } => {
                q(self.core.create_issue(&ipc_to_core(issue)))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueStatus { id, status } => {
                q(self.core.update_issue_status(&id, status))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueTitle { id, title } => {
                q(self.core.update_issue_title(&id, &title))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueDescription { id, description } => {
                q(self.core.update_issue_description(&id, &description))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::UpdateIssueType { id, issue_type } => {
                q(self.core.update_issue_type(&id, issue_type))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::SetAssignee { id, assignee } => {
                q(self.core.set_assignee(&id, &assignee))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::ClearAssignee { id } => {
                q(self.core.clear_assignee(&id))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddLabel { id, label } => {
                q(self.core.add_label(&id, &label))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLabel { id, label } => {
                let removed = q(self.core.remove_label(&id, &label))?;
                Ok(MutateResult::LabelRemoved { removed })
            }
            MutateOp::AddNote {
                id,
                status,
                content,
            } => {
                q(self.core.add_note(&id, status, &content))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::LogEvent { event } => {
                q(self.core.log_event(&event))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddDependency(DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                q(self.core.add_dependency(&from_id, &to_id, relation))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveDependency(DependencyRef {
                from_id,
                to_id,
                relation,
            }) => {
                q(self.core.remove_dependency(&from_id, &to_id, relation))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::AddLink {
                id,
                link_type,
                url,
                external_id,
                rel,
            } => {
                let link = Link {
                    id: 0,
                    issue_id: id,
                    link_type,
                    url,
                    external_id,
                    rel,
                    created_at: Utc::now(),
                };
                q(self.core.add_link(&link))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::RemoveLink { id, url } => {
                q(self.core.remove_link_by_url(&id, &url))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::EnsurePrefix { prefix } => {
                q(self.core.ensure_prefix(&prefix))?;
                Ok(MutateResult::Ok)
            }
            MutateOp::IncrementPrefixCount { prefix } => {
                q(self.core.increment_prefix_count(&prefix))?;
                Ok(MutateResult::Ok)
            }
        }
    }
}

/// Map core errors to IPC error strings.
fn q<T>(result: wk_core::Result<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

/// Convert a core Issue to an IPC Issue (strip HLC fields).
fn core_to_ipc(issue: wk_core::Issue) -> IpcIssue {
    IpcIssue {
        id: issue.id,
        issue_type: issue.issue_type,
        title: issue.title,
        description: issue.description,
        status: issue.status,
        assignee: issue.assignee,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
    }
}

/// Convert an IPC Issue to a core Issue (add None HLC fields).
fn ipc_to_core(issue: IpcIssue) -> wk_core::Issue {
    wk_core::Issue {
        id: issue.id,
        issue_type: issue.issue_type,
        title: issue.title,
        description: issue.description,
        status: issue.status,
        assignee: issue.assignee,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    }
}
