// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::collections::{HashMap, HashSet};
use std::path::Path;

use chrono::{Duration, Utc};

use crate::cli::OutputFormat;
use crate::db::Database;
use crate::display::format_issue_line;
use crate::error::Result;
use crate::models::{Issue, IssueType, Status};
use crate::schema::ready::ReadyOutputJson;
use crate::schema::IssueJson;

use super::filtering::{matches_filter_groups, matches_label_groups, parse_filter_groups};
use super::open_db;

/// Maximum number of issues to show in ready output.
/// Keeps output manageable - you can only work on a few things at once.
const MAX_READY_ISSUES: usize = 5;

/// Assignee filter mode for the ready command.
enum AssigneeFilter {
    /// Show all issues regardless of assignment
    All,
    /// Show only unassigned issues
    Unassigned,
    /// Show only issues assigned to specific assignees
    Specific(Vec<String>),
    /// Show unassigned issues OR issues assigned to specific assignees (default with config)
    UnassignedOrSpecific(Vec<String>),
}

/// Check if an issue matches the assignee filter.
fn matches_assignee_filter(issue: &Issue, filter: &AssigneeFilter) -> bool {
    match filter {
        AssigneeFilter::All => true,
        AssigneeFilter::Unassigned => issue.assignee.is_none(),
        AssigneeFilter::Specific(assignees) => issue
            .assignee
            .as_ref()
            .is_some_and(|a| assignees.iter().any(|f| a == f)),
        AssigneeFilter::UnassignedOrSpecific(assignees) => {
            issue.assignee.is_none()
                || issue
                    .assignee
                    .as_ref()
                    .is_some_and(|a| assignees.iter().any(|f| a == f))
        }
    }
}

/// Get the default assignee filter based on configuration.
/// If .work/current/assignee exists and is non-empty, returns UnassignedOrSpecific.
/// Otherwise, returns Unassigned.
fn get_default_assignee_filter(work_dir: &Path) -> AssigneeFilter {
    let assignee_file = work_dir.join("current").join("assignee");
    if let Ok(content) = std::fs::read_to_string(&assignee_file) {
        let assignee = content.trim();
        if !assignee.is_empty() {
            return AssigneeFilter::UnassignedOrSpecific(vec![assignee.to_string()]);
        }
    }
    AssigneeFilter::Unassigned
}

pub fn run(
    issue_type: Vec<String>,
    label: Vec<String>,
    assignee: Vec<String>,
    unassigned: bool,
    all_assignees: bool,
    format: OutputFormat,
) -> Result<()> {
    let (db, _, _) = open_db()?;
    // Get work directory for default assignee config
    let work_dir = crate::config::find_work_dir()?;
    run_impl(
        &db,
        &work_dir,
        issue_type,
        label,
        assignee,
        unassigned,
        all_assignees,
        format,
    )
}

/// Internal implementation that accepts db for testing.
#[allow(clippy::too_many_arguments)] // TODO(refactor): Consider using an options struct to bundle parameters
pub(crate) fn run_impl(
    db: &Database,
    work_dir: &Path,
    issue_type: Vec<String>,
    label: Vec<String>,
    assignee: Vec<String>,
    unassigned: bool,
    all_assignees: bool,
    format: OutputFormat,
) -> Result<()> {
    // Parse filter groups
    let type_groups =
        parse_filter_groups(&issue_type, |s| s.parse::<IssueType>().map_err(Into::into))?;
    let label_groups = parse_filter_groups(&label, |s| Ok(s.to_string()))?;

    // Ready = unblocked todo items only
    let mut issues = db.list_issues(Some(Status::Todo), None, None)?;

    // Apply type filter first (no DB access needed)
    if type_groups.is_some() {
        issues.retain(|issue| matches_filter_groups(&type_groups, || issue.issue_type));
    }

    // Pre-fetch all labels for remaining issues in one query
    let issue_ids: Vec<&str> = issues.iter().map(|i| i.id.as_str()).collect();
    let labels_map: HashMap<String, Vec<String>> = db.get_labels_batch(&issue_ids)?;

    // Apply label filter using pre-fetched map
    if label_groups.is_some() {
        issues.retain(|issue| {
            let issue_labels = labels_map
                .get(&issue.id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            matches_label_groups(&label_groups, issue_labels)
        });
    }

    // Determine assignee filter behavior
    let assignee_filter = if all_assignees {
        AssigneeFilter::All
    } else if unassigned {
        AssigneeFilter::Unassigned
    } else if !assignee.is_empty() {
        AssigneeFilter::Specific(assignee)
    } else {
        // Default: unassigned + current user's assignee (if configured)
        get_default_assignee_filter(work_dir)
    };

    // Apply assignee filter
    issues.retain(|issue| matches_assignee_filter(issue, &assignee_filter));

    // Get blocked IDs and filter to ready (unblocked) only
    let blocked_ids: HashSet<String> = db.get_blocked_issue_ids()?.into_iter().collect();
    let mut ready_issues: Vec<_> = issues
        .into_iter()
        .filter(|issue| !blocked_ids.contains(&issue.id))
        .collect();

    // Sort with multi-tier comparator:
    // 1. Recent issues (created <48h ago) come first
    // 2. Within recent: sort by priority ASC (0=highest first)
    // 3. Old issues (created >=48h ago) come after
    // 4. Within old: sort by created_at ASC (oldest first)
    // 5. Tiebreaker: created_at ASC
    let cutoff = Utc::now() - Duration::hours(48);
    ready_issues.sort_by(|a, b| {
        let a_recent = a.created_at >= cutoff;
        let b_recent = b.created_at >= cutoff;

        match (a_recent, b_recent) {
            // Recent before old
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            // Both recent: sort by priority ASC, then created_at ASC as tiebreaker
            (true, true) => {
                // Use pre-fetched labels - no DB access
                let empty_vec = Vec::new();
                let tags_a = labels_map.get(&a.id).unwrap_or(&empty_vec);
                let tags_b = labels_map.get(&b.id).unwrap_or(&empty_vec);
                let priority_a = Database::priority_from_tags(tags_a);
                let priority_b = Database::priority_from_tags(tags_b);
                match priority_a.cmp(&priority_b) {
                    std::cmp::Ordering::Equal => a.created_at.cmp(&b.created_at), // ASC tiebreaker
                    other => other,
                }
            }
            // Both old: sort by created_at ASC (oldest first)
            (false, false) => a.created_at.cmp(&b.created_at),
        }
    });

    // Truncate to hard limit - ready queue shows only top priorities
    let total_ready = ready_issues.len();
    ready_issues.truncate(MAX_READY_ISSUES);

    match format {
        OutputFormat::Text => {
            if ready_issues.is_empty() {
                println!("No ready issues");
            } else {
                for issue in &ready_issues {
                    println!("{}", format_issue_line(issue));
                }
                if total_ready > MAX_READY_ISSUES {
                    let remaining = total_ready - MAX_READY_ISSUES;
                    println!("\n({remaining} more — use `wk list` to see all)",);
                }
            }
        }
        OutputFormat::Json => {
            let mut json_issues = Vec::new();
            for issue in &ready_issues {
                // Use pre-fetched labels - no additional DB access
                let labels = labels_map.get(&issue.id).cloned().unwrap_or_default();
                json_issues.push(IssueJson::new(
                    issue.id.clone(),
                    issue.issue_type,
                    issue.status,
                    issue.title.clone(),
                    issue.assignee.clone(),
                    labels,
                ));
            }
            let output = ReadyOutputJson {
                issues: json_issues,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Id => {
            for issue in &ready_issues {
                println!("{}", issue.id);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "ready_tests.rs"]
mod tests;
