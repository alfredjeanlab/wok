// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! DatabaseHandle abstraction for routing database operations.
//!
//! In user-level mode, operations go through the daemon via IPC.
//! In private mode, operations use direct SQLite access.

use std::collections::HashMap;

use crate::daemon::{DaemonClient, MutateOp, MutateResult, QueryOp, QueryResult};
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{
    Dependency, Event, Issue, IssueType, Link, Note, PrefixInfo, Relation, Status,
};

/// Handle for database operations that routes to either direct SQLite or daemon IPC.
pub enum DatabaseHandle {
    /// Direct SQLite access (private mode).
    Direct(Database),
    /// Daemon IPC access (user-level mode).
    Daemon(DaemonClient),
}

impl DatabaseHandle {
    // ========================================================================
    // Issue operations
    // ========================================================================

    /// Resolve a partial ID to a full ID.
    pub fn resolve_id(&mut self, partial_id: &str) -> Result<String> {
        match self {
            DatabaseHandle::Direct(db) => db.resolve_id(partial_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::ResolveId {
                    partial_id: partial_id.to_string(),
                })? {
                    QueryResult::ResolvedId { id } => Ok(id),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Check if an issue exists.
    pub fn issue_exists(&mut self, id: &str) -> Result<bool> {
        match self {
            DatabaseHandle::Direct(db) => db.issue_exists(id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::IssueExists { id: id.to_string() })? {
                    QueryResult::Bool { value } => Ok(value),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get an issue by ID.
    pub fn get_issue(&mut self, id: &str) -> Result<Issue> {
        match self {
            DatabaseHandle::Direct(db) => db.get_issue(id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetIssue { id: id.to_string() })? {
                    QueryResult::Issue { issue } => Ok(issue),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Create a new issue.
    pub fn create_issue(&mut self, issue: &Issue) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.create_issue(issue),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::CreateIssue {
                    issue: issue.clone(),
                })?;
                Ok(())
            }
        }
    }

    /// List issues with optional filters.
    pub fn list_issues(
        &mut self,
        status: Option<Status>,
        issue_type: Option<IssueType>,
        label: Option<&str>,
    ) -> Result<Vec<Issue>> {
        match self {
            DatabaseHandle::Direct(db) => db.list_issues(status, issue_type, label),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::ListIssues {
                    status,
                    issue_type,
                    label: label.map(String::from),
                })? {
                    QueryResult::Issues { issues } => Ok(issues),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Search issues by query string.
    pub fn search_issues(&mut self, query: &str) -> Result<Vec<Issue>> {
        match self {
            DatabaseHandle::Direct(db) => db.search_issues(query),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::SearchIssues {
                    query: query.to_string(),
                })? {
                    QueryResult::Issues { issues } => Ok(issues),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get IDs of blocked issues.
    pub fn get_blocked_issue_ids(&mut self) -> Result<Vec<String>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_blocked_issue_ids(),
            DatabaseHandle::Daemon(client) => match client.query(QueryOp::GetBlockedIssueIds)? {
                QueryResult::Ids { ids } => Ok(ids),
                _ => Err(Error::Daemon("unexpected query result".to_string())),
            },
        }
    }

    /// Update issue status.
    pub fn update_issue_status(&mut self, id: &str, status: Status) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.update_issue_status(id, status),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::UpdateIssueStatus {
                    id: id.to_string(),
                    status,
                })?;
                Ok(())
            }
        }
    }

    /// Update issue title.
    pub fn update_issue_title(&mut self, id: &str, title: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.update_issue_title(id, title),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::UpdateIssueTitle {
                    id: id.to_string(),
                    title: title.to_string(),
                })?;
                Ok(())
            }
        }
    }

    /// Update issue description.
    pub fn update_issue_description(&mut self, id: &str, description: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.update_issue_description(id, description),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::UpdateIssueDescription {
                    id: id.to_string(),
                    description: description.to_string(),
                })?;
                Ok(())
            }
        }
    }

    /// Update issue type.
    pub fn update_issue_type(&mut self, id: &str, issue_type: IssueType) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.update_issue_type(id, issue_type),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::UpdateIssueType {
                    id: id.to_string(),
                    issue_type,
                })?;
                Ok(())
            }
        }
    }

    /// Set issue assignee.
    pub fn set_assignee(&mut self, id: &str, assignee: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.set_assignee(id, assignee),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::SetAssignee {
                    id: id.to_string(),
                    assignee: assignee.to_string(),
                })?;
                Ok(())
            }
        }
    }

    /// Clear issue assignee.
    pub fn clear_assignee(&mut self, id: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.clear_assignee(id),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::ClearAssignee { id: id.to_string() })?;
                Ok(())
            }
        }
    }

    // ========================================================================
    // Label operations
    // ========================================================================

    /// Get labels for an issue.
    pub fn get_labels(&mut self, issue_id: &str) -> Result<Vec<String>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_labels(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetLabels {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Labels { labels } => Ok(labels),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get labels for multiple issues.
    pub fn get_labels_batch(&mut self, issue_ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_labels_batch(issue_ids),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetLabelsBatch {
                    ids: issue_ids.iter().map(|s| s.to_string()).collect(),
                })? {
                    QueryResult::LabelsBatch { labels } => Ok(labels),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Add a label to an issue.
    pub fn add_label(&mut self, issue_id: &str, label: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.add_label(issue_id, label),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::AddLabel {
                    id: issue_id.to_string(),
                    label: label.to_string(),
                })?;
                Ok(())
            }
        }
    }

    /// Remove a label from an issue.
    pub fn remove_label(&mut self, issue_id: &str, label: &str) -> Result<bool> {
        match self {
            DatabaseHandle::Direct(db) => db.remove_label(issue_id, label),
            DatabaseHandle::Daemon(client) => {
                match client.mutate(MutateOp::RemoveLabel {
                    id: issue_id.to_string(),
                    label: label.to_string(),
                })? {
                    MutateResult::LabelRemoved { removed } => Ok(removed),
                    _ => Ok(false),
                }
            }
        }
    }

    // ========================================================================
    // Note operations
    // ========================================================================

    /// Get notes for an issue.
    pub fn get_notes(&mut self, issue_id: &str) -> Result<Vec<Note>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_notes(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetNotes {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Notes { notes } => Ok(notes),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Add a note to an issue.
    pub fn add_note(&mut self, issue_id: &str, status: Status, content: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => {
                db.add_note(issue_id, status, content)?;
                Ok(())
            }
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::AddNote {
                    id: issue_id.to_string(),
                    status,
                    content: content.to_string(),
                })?;
                Ok(())
            }
        }
    }

    // ========================================================================
    // Event operations
    // ========================================================================

    /// Get events for an issue.
    pub fn get_events(&mut self, issue_id: &str) -> Result<Vec<Event>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_events(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetEvents {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Events { events } => Ok(events),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get recent events across all issues.
    pub fn get_recent_events(&mut self, limit: usize) -> Result<Vec<Event>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_recent_events(limit),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetAllEvents { limit: Some(limit) })? {
                    QueryResult::Events { events } => Ok(events),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Log an event.
    pub fn log_event(&mut self, event: &Event) -> Result<i64> {
        match self {
            DatabaseHandle::Direct(db) => db.log_event(event),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::LogEvent {
                    event: event.clone(),
                })?;
                Ok(0) // Daemon doesn't return the ID
            }
        }
    }

    // ========================================================================
    // Dependency operations
    // ========================================================================

    /// Get dependencies from an issue.
    pub fn get_deps_from(&mut self, issue_id: &str) -> Result<Vec<Dependency>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_deps_from(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetDepsFrom {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Dependencies { deps } => Ok(deps),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get blockers for an issue (returns issue IDs).
    pub fn get_blockers(&mut self, issue_id: &str) -> Result<Vec<String>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_blockers(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetBlockers {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Ids { ids } => Ok(ids),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get issues blocked by an issue (returns issue IDs).
    pub fn get_blocking(&mut self, issue_id: &str) -> Result<Vec<String>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_blocking(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetBlocking {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Ids { ids } => Ok(ids),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get tracked issues (returns issue IDs).
    pub fn get_tracked(&mut self, issue_id: &str) -> Result<Vec<String>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_tracked(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetTracked {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Ids { ids } => Ok(ids),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get tracking issues (returns issue IDs).
    pub fn get_tracking(&mut self, issue_id: &str) -> Result<Vec<String>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_tracking(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetTracking {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Ids { ids } => Ok(ids),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get transitive blockers (returns issue IDs).
    pub fn get_transitive_blockers(&mut self, issue_id: &str) -> Result<Vec<String>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_transitive_blockers(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetTransitiveBlockers {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Ids { ids } => Ok(ids),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Add a dependency.
    pub fn add_dependency(&mut self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.add_dependency(from_id, to_id, relation),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::AddDependency {
                    from_id: from_id.to_string(),
                    to_id: to_id.to_string(),
                    relation,
                })?;
                Ok(())
            }
        }
    }

    /// Remove a dependency.
    pub fn remove_dependency(
        &mut self,
        from_id: &str,
        to_id: &str,
        relation: Relation,
    ) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.remove_dependency(from_id, to_id, relation),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::RemoveDependency {
                    from_id: from_id.to_string(),
                    to_id: to_id.to_string(),
                    relation,
                })?;
                Ok(())
            }
        }
    }

    // ========================================================================
    // Link operations
    // ========================================================================

    /// Get links for an issue.
    pub fn get_links(&mut self, issue_id: &str) -> Result<Vec<Link>> {
        match self {
            DatabaseHandle::Direct(db) => db.get_links(issue_id),
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetLinks {
                    id: issue_id.to_string(),
                })? {
                    QueryResult::Links { links } => Ok(links),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Get a link by URL (searches through all links for the issue).
    pub fn get_link_by_url(&mut self, issue_id: &str, url: &str) -> Result<Option<Link>> {
        match self {
            DatabaseHandle::Direct(db) => {
                // Filter through all links to find one matching the URL
                let links = db.get_links(issue_id)?;
                Ok(links.into_iter().find(|l| l.url.as_deref() == Some(url)))
            }
            DatabaseHandle::Daemon(client) => {
                match client.query(QueryOp::GetLinkByUrl {
                    id: issue_id.to_string(),
                    url: url.to_string(),
                })? {
                    QueryResult::Link { link } => Ok(link),
                    _ => Err(Error::Daemon("unexpected query result".to_string())),
                }
            }
        }
    }

    /// Add a link to an issue.
    pub fn add_link(&mut self, link: &Link) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => {
                db.add_link(link)?;
                Ok(())
            }
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::AddLink {
                    id: link.issue_id.clone(),
                    link_type: link.link_type,
                    url: link.url.clone(),
                    external_id: link.external_id.clone(),
                    rel: link.rel,
                })?;
                Ok(())
            }
        }
    }

    /// Remove a link from an issue by URL.
    pub fn remove_link_by_url(&mut self, issue_id: &str, url: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => {
                // Find the link by URL first, then remove by ID
                let links = db.get_links(issue_id)?;
                if let Some(link) = links.into_iter().find(|l| l.url.as_deref() == Some(url)) {
                    db.remove_link(link.id)?;
                }
                Ok(())
            }
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::RemoveLink {
                    id: issue_id.to_string(),
                    url: url.to_string(),
                })?;
                Ok(())
            }
        }
    }

    // ========================================================================
    // Prefix operations
    // ========================================================================

    /// List all prefixes.
    pub fn list_prefixes(&mut self) -> Result<Vec<PrefixInfo>> {
        match self {
            DatabaseHandle::Direct(db) => db.list_prefixes(),
            DatabaseHandle::Daemon(client) => match client.query(QueryOp::ListPrefixes)? {
                QueryResult::Prefixes { prefixes } => Ok(prefixes),
                _ => Err(Error::Daemon("unexpected query result".to_string())),
            },
        }
    }

    /// Ensure a prefix exists.
    pub fn ensure_prefix(&mut self, prefix: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.ensure_prefix(prefix),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::EnsurePrefix {
                    prefix: prefix.to_string(),
                })?;
                Ok(())
            }
        }
    }

    /// Increment prefix issue count.
    pub fn increment_prefix_count(&mut self, prefix: &str) -> Result<()> {
        match self {
            DatabaseHandle::Direct(db) => db.increment_prefix_count(prefix),
            DatabaseHandle::Daemon(client) => {
                client.mutate(MutateOp::IncrementPrefixCount {
                    prefix: prefix.to_string(),
                })?;
                Ok(())
            }
        }
    }

    // ========================================================================
    // Static helper for priority (doesn't need DB)
    // ========================================================================

    /// Extract priority from tag list (static method, no DB access needed).
    pub fn priority_from_tags(tags: &[String]) -> u8 {
        Database::priority_from_tags(tags)
    }
}
