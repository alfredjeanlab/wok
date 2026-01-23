// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::collections::HashSet;

use chrono::Utc;

use crate::cli::OutputFormat;
use crate::db::Database;
use crate::display::format_issue_line;
use crate::error::Result;
use crate::filter::{parse_filter, FilterExpr, FilterField};
use crate::models::{IssueType, Status};
use crate::schema::list::ListOutputJson;
use crate::schema::IssueJson;

use super::filtering::{matches_filter_groups, matches_label_groups, parse_filter_groups};
use super::open_db;

/// Default limit for list output when not explicitly specified.
/// Prevents large result sets from overwhelming terminal output.
const DEFAULT_LIMIT: usize = 100;

#[allow(clippy::too_many_arguments)]
pub fn run(
    status: Vec<String>,
    issue_type: Vec<String>,
    label: Vec<String>,
    assignee: Vec<String>,
    unassigned: bool,
    filter: Vec<String>,
    limit: Option<usize>,
    blocked_only: bool,
    all: bool,
    format: OutputFormat,
) -> Result<()> {
    let (db, _, _) = open_db()?;
    run_impl(
        &db,
        status,
        issue_type,
        label,
        assignee,
        unassigned,
        filter,
        limit,
        blocked_only,
        all,
        format,
    )
}

/// Internal implementation that accepts db for testing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_impl(
    db: &Database,
    status: Vec<String>,
    issue_type: Vec<String>,
    label: Vec<String>,
    assignee: Vec<String>,
    unassigned: bool,
    filter: Vec<String>,
    limit: Option<usize>,
    blocked_only: bool,
    all: bool,
    format: OutputFormat,
) -> Result<()> {
    // Parse filter groups
    let status_groups = parse_filter_groups(&status, |s| s.parse::<Status>())?;
    let type_groups =
        parse_filter_groups(&issue_type, |s| s.parse::<IssueType>().map_err(Into::into))?;
    let label_groups = parse_filter_groups(&label, |s| Ok(s.to_string()))?;

    // Parse time-based filter expressions
    let filters: Vec<FilterExpr> = filter
        .iter()
        .map(|f| parse_filter(f))
        .collect::<Result<_>>()?;

    // Check if any filter targets a terminal state field (completed, skipped, closed)
    let has_terminal_filter = filters.iter().any(|f| {
        matches!(
            f.field,
            FilterField::Completed | FilterField::Skipped | FilterField::Closed
        )
    });

    // Get all issues (we'll filter in-memory for complex multi-value logic)
    let mut issues = db.list_issues(None, None, None)?;

    // Default: show open issues (todo + in_progress) when no status filter and not --all
    // Exception: when terminal filter is used, include closed issues (they're the target)
    if !all && status_groups.is_none() && !has_terminal_filter {
        issues.retain(|issue| issue.status == Status::Todo || issue.status == Status::InProgress);
    } else if status_groups.is_some() {
        // Filter by explicit status groups
        issues.retain(|issue| matches_filter_groups(&status_groups, || issue.status));
    }

    // Filter by type groups
    if type_groups.is_some() {
        issues.retain(|issue| matches_filter_groups(&type_groups, || issue.issue_type));
    }

    // Filter by label groups
    if label_groups.is_some() {
        issues.retain(|issue| {
            let issue_labels = db.get_labels(&issue.id).unwrap_or_default();
            matches_label_groups(&label_groups, &issue_labels)
        });
    }

    // Filter by assignee
    if unassigned {
        issues.retain(|issue| issue.assignee.is_none());
    } else if !assignee.is_empty() {
        issues.retain(|issue| {
            issue
                .assignee
                .as_ref()
                .is_some_and(|a| assignee.iter().any(|f| a == f))
        });
    }

    // Apply time-based filters
    if !filters.is_empty() {
        let now = Utc::now();
        issues.retain(|issue| filters.iter().all(|f| f.matches(issue, now)));
    }

    // Apply blocked filter if specified
    if blocked_only {
        let blocked_ids: HashSet<String> = db.get_blocked_issue_ids()?.into_iter().collect();
        issues.retain(|issue| blocked_ids.contains(&issue.id));
    }

    // Sort by priority ASC, then created_at DESC
    issues.sort_by(|a, b| {
        let tags_a = db.get_labels(&a.id).unwrap_or_default();
        let tags_b = db.get_labels(&b.id).unwrap_or_default();
        let priority_a = Database::priority_from_tags(&tags_a);
        let priority_b = Database::priority_from_tags(&tags_b);

        match priority_a.cmp(&priority_b) {
            std::cmp::Ordering::Equal => b.created_at.cmp(&a.created_at), // DESC
            other => other,
        }
    });

    // Apply limit after sorting (default 100, or explicit value, 0 = unlimited)
    let effective_limit = limit.unwrap_or(DEFAULT_LIMIT);
    if effective_limit > 0 {
        issues.truncate(effective_limit);
    }

    match format {
        OutputFormat::Text => {
            for issue in &issues {
                println!("{}", format_issue_line(issue));
            }
        }
        OutputFormat::Json => {
            let mut json_issues = Vec::new();
            for issue in &issues {
                let labels = db.get_labels(&issue.id)?;
                json_issues.push(IssueJson::new(
                    issue.id.clone(),
                    issue.issue_type,
                    issue.status,
                    issue.title.clone(),
                    issue.assignee.clone(),
                    labels,
                ));
            }
            let filters_applied = if filter.is_empty() {
                None
            } else {
                Some(filter)
            };
            let output = ListOutputJson {
                issues: json_issues,
                filters_applied,
                limit,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Ids => {
            let ids: Vec<&str> = issues.iter().map(|i| i.id.as_str()).collect();
            if !ids.is_empty() {
                println!("{}", ids.join(" "));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
