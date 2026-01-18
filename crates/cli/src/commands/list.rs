// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::Utc;
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::db::Database;
use crate::display::format_issue_line;
use crate::error::Result;
use crate::filter::{parse_filter, FilterExpr, FilterField};
use crate::models::{IssueType, Status};

use std::collections::HashSet;

use super::open_db;

/// JSON representation of an issue for list output.
#[derive(Serialize)]
struct ListIssueJson {
    id: String,
    issue_type: IssueType,
    status: Status,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<String>,
    labels: Vec<String>,
}

/// JSON output structure for the list command.
#[derive(Serialize)]
struct ListOutputJson {
    issues: Vec<ListIssueJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters_applied: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
}

/// Parse filter values: comma-separated values within each Vec entry are OR'd,
/// multiple Vec entries are AND'd together.
/// Returns None if no filters provided, Some(groups) otherwise.
pub(crate) fn parse_filter_groups<T, F>(
    values: &[String],
    parse_fn: F,
) -> Result<Option<Vec<Vec<T>>>>
where
    F: Fn(&str) -> Result<T>,
{
    if values.is_empty() {
        return Ok(None);
    }

    let mut groups = Vec::new();
    for value in values {
        let mut group = Vec::new();
        for part in value.split(',') {
            let part = part.trim();
            if !part.is_empty() {
                group.push(parse_fn(part)?);
            }
        }
        if !group.is_empty() {
            groups.push(group);
        }
    }

    if groups.is_empty() {
        Ok(None)
    } else {
        Ok(Some(groups))
    }
}

/// Check if an issue matches the filter groups.
/// Each group is OR'd internally, all groups must match (AND).
pub(crate) fn matches_filter_groups<T, F>(groups: &Option<Vec<Vec<T>>>, get_value: F) -> bool
where
    T: PartialEq,
    F: Fn() -> T,
{
    match groups {
        None => true,
        Some(groups) => {
            let value = get_value();
            groups.iter().all(|group| group.contains(&value))
        }
    }
}

/// Check if an issue matches label filter groups.
/// Each group is OR'd internally (issue has at least one label from group),
/// all groups must match (AND).
pub(crate) fn matches_label_groups(
    groups: &Option<Vec<Vec<String>>>,
    issue_labels: &[String],
) -> bool {
    match groups {
        None => true,
        Some(groups) => groups
            .iter()
            .all(|group| group.iter().any(|label| issue_labels.contains(label))),
    }
}

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
    let (db, _) = open_db()?;
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

    // Check if any filter targets the closed field
    let has_closed_filter = filters.iter().any(|f| f.field == FilterField::Closed);

    // Get all issues (we'll filter in-memory for complex multi-value logic)
    let mut issues = db.list_issues(None, None, None)?;

    // Default: show open issues (todo + in_progress) when no status filter and not --all
    // Exception: when closed filter is used, include closed issues (they're the target)
    if !all && status_groups.is_none() && !has_closed_filter {
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

    // Apply limit after sorting
    if let Some(n) = limit {
        issues.truncate(n);
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
                json_issues.push(ListIssueJson {
                    id: issue.id.clone(),
                    issue_type: issue.issue_type,
                    status: issue.status,
                    title: issue.title.clone(),
                    assignee: issue.assignee.clone(),
                    labels,
                });
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
    }

    Ok(())
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
