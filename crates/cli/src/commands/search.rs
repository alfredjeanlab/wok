// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::Utc;

use crate::cli::OutputFormat;
use crate::db::Database;
use crate::display::format_issue_line;
use crate::error::Result;
use crate::filter::{parse_filter, FilterExpr};
use crate::models::{IssueType, Status};
use crate::schema::search::SearchOutputJson;
use crate::schema::IssueJson;

use super::list::{matches_filter_groups, matches_label_groups, parse_filter_groups};
use super::open_db;

/// Default limit for search results in text output.
const DEFAULT_LIMIT: usize = 25;

#[allow(clippy::too_many_arguments)]
pub fn run(
    query: &str,
    status: Vec<String>,
    issue_type: Vec<String>,
    label: Vec<String>,
    assignee: Vec<String>,
    unassigned: bool,
    filter: Vec<String>,
    limit: Option<usize>,
    format: OutputFormat,
) -> Result<()> {
    let (db, _, _) = open_db()?;
    run_impl(
        &db, query, status, issue_type, label, assignee, unassigned, filter, limit, format,
    )
}

/// Internal implementation that accepts db for testing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_impl(
    db: &Database,
    query: &str,
    status: Vec<String>,
    issue_type: Vec<String>,
    label: Vec<String>,
    assignee: Vec<String>,
    unassigned: bool,
    filter: Vec<String>,
    limit: Option<usize>,
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

    // Search issues
    let mut issues = db.search_issues(query)?;

    // Apply filters (same logic as list)
    if status_groups.is_some() {
        issues.retain(|issue| matches_filter_groups(&status_groups, || issue.status));
    }

    if type_groups.is_some() {
        issues.retain(|issue| matches_filter_groups(&type_groups, || issue.issue_type));
    }

    if label_groups.is_some() {
        issues.retain(|issue| {
            let issue_labels = db.get_labels(&issue.id).unwrap_or_default();
            matches_label_groups(&label_groups, &issue_labels)
        });
    }

    // Apply assignee filter
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

    // Sort by priority ASC, then created_at DESC (same as list)
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

    // Use explicit limit or default
    let effective_limit = limit.unwrap_or(DEFAULT_LIMIT);

    // Calculate how many more results exist beyond the limit
    let total_count = issues.len();
    let more_count = if total_count > effective_limit {
        Some(total_count - effective_limit)
    } else {
        None
    };

    match format {
        OutputFormat::Text => {
            for issue in issues.iter().take(effective_limit) {
                println!("{}", format_issue_line(issue));
            }
            if let Some(count) = more_count {
                println!("... {} more", count);
            }
        }
        OutputFormat::Json => {
            let mut json_issues = Vec::new();
            for issue in issues.iter().take(effective_limit) {
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
            let output = SearchOutputJson {
                issues: json_issues,
                filters_applied,
                limit,
                more: more_count,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Ids => {
            for issue in issues.iter().take(effective_limit) {
                println!("{}", issue.id);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
