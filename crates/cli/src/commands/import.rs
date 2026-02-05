// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::collections::HashSet;
use std::io::{self, BufRead, BufReader};

use serde::Deserialize;

use crate::config::Config;
use crate::db::links::new_link;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{Action, Event, Issue, IssueType, Link, LinkRel, LinkType, Relation, Status};

use super::filtering::{matches_filter_groups, matches_label_groups, parse_filter_groups};
use super::open_db;

// Type alias for imported issue data
// (issue, labels, notes, deps, close_data, links)
type ImportedIssue = (
    Issue,
    Vec<String>,
    Vec<(Status, String)>,
    Vec<(String, String, Relation)>,
    Option<CloseData>,
    Vec<ImportedLink>,
);

// Close event data from bd import
struct CloseData {
    reason: String,
    is_failure: bool,
}

// Imported link data (simplified)
struct ImportedLink {
    link_type: Option<LinkType>,
    url: Option<String>,
    external_id: Option<String>,
    rel: Option<LinkRel>,
}

// wk native export format (matches export.rs ExportedIssue)
#[derive(Deserialize)]
struct WkIssue {
    #[serde(flatten)]
    issue: Issue,
    labels: Vec<String>,
    notes: Vec<WkNote>,
    deps: Vec<WkDependency>,
    #[serde(default)]
    links: Vec<Link>,
    // NOTE(compat): Required for JSON deserialization
    #[allow(dead_code)]
    events: Vec<Event>,
}

// Note format in wk export (uses Status enum)
#[derive(Deserialize)]
struct WkNote {
    // NOTE(compat): Required for JSON deserialization
    #[allow(dead_code)]
    id: i64,
    // NOTE(compat): Required for JSON deserialization
    #[allow(dead_code)]
    issue_id: String,
    status: Status,
    content: String,
    // NOTE(compat): Required for JSON deserialization
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

// Dependency format in wk export (uses Relation enum)
#[derive(Deserialize)]
struct WkDependency {
    from_id: String,
    to_id: String,
    relation: Relation,
    // NOTE(compat): Required for JSON deserialization
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

// Beads export format
#[derive(Deserialize)]
struct BeadsIssue {
    id: String,
    title: String,
    #[serde(default)]
    description: Option<String>,
    status: String,
    #[serde(default)]
    priority: i32,
    issue_type: String,
    created_at: String,
    updated_at: String,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    dependencies: Vec<BeadsDependency>,
    #[serde(default)]
    comments: Vec<BeadsComment>,
    #[serde(default)]
    close_reason: Option<String>,
    #[serde(default)]
    delete_reason: Option<String>,
}

#[derive(Deserialize)]
struct BeadsDependency {
    depends_on_id: String,
    #[serde(rename = "type")]
    dep_type: String,
}

#[derive(Deserialize)]
struct BeadsComment {
    #[serde(alias = "content")] // Accept both "text" and "content" for backwards compat
    text: String,
    // NOTE(compat): Required for JSON deserialization
    #[allow(dead_code)]
    created_at: String,
}

// Import result tracking
#[derive(Default)]
struct ImportResult {
    created: usize,
    updated: usize,
    filtered: usize,
    collisions: Vec<String>,
    missing_deps: Vec<(String, String)>,
}

// Format detection
fn detect_format<'a>(path: &str, explicit_format: &'a str) -> &'a str {
    if explicit_format != "wok" {
        return explicit_format;
    }
    // Auto-detect beads format from path
    if path.ends_with(".beads/issues.jsonl") {
        return "bd";
    }
    explicit_format
}

// Failure words that indicate a closed issue should map to Closed rather than Done
const FAILURE_WORDS: &[&str] = &[
    "failed",
    "rejected",
    "wontfix",
    "won't fix",
    "canceled",
    "cancelled",
    "abandoned",
    "blocked",
    "error",
    "timeout",
    "aborted",
];

fn is_failure_reason(reason: &str) -> bool {
    let lower = reason.to_lowercase();
    FAILURE_WORDS.iter().any(|w| lower.contains(w))
}

// Status conversion for beads format
fn convert_beads_status(
    status: &str,
    close_reason: &Option<String>,
    delete_reason: &Option<String>,
) -> Status {
    match status {
        "open" => Status::Todo,
        "in_progress" => Status::InProgress,
        "closed" => match close_reason {
            Some(r) if is_failure_reason(r) => Status::Closed,
            _ => Status::Done,
        },
        // Tombstoned issues are deleted/abandoned - map to Closed
        "tombstone" => match delete_reason {
            Some(r) if is_failure_reason(r) => Status::Closed,
            Some(_) => Status::Closed, // Any delete_reason means it's closed, not done
            None => Status::Closed,    // Tombstone with no reason is still closed
        },
        "blocked" | "deferred" => Status::Todo,
        _ => Status::Todo,
    }
}

// Dependency type conversion for beads format
fn convert_beads_dep_type(dep_type: &str) -> Relation {
    match dep_type {
        "blocks" => Relation::Blocks,
        "parent" | "tracks" => Relation::Tracks,
        "parent-child" | "child-of" | "tracked-by" => Relation::TrackedBy,
        _ => Relation::Blocks, // Default fallback
    }
}

// Type conversion for beads format
fn convert_beads_type(issue_type: &str) -> IssueType {
    match issue_type {
        "bug" => IssueType::Bug,
        "feature" => IssueType::Feature,
        "epic" => IssueType::Epic,
        "chore" => IssueType::Chore,
        _ => IssueType::Task, // task, etc.
    }
}

// Convert beads issue to internal format
fn convert_beads_issue(bd: BeadsIssue) -> Result<ImportedIssue> {
    let created_at = chrono::DateTime::parse_from_rfc3339(&bd.created_at)
        .map_err(|e| Error::InvalidTimestamp {
            reason: format!("created_at: {}", e),
        })?
        .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&bd.updated_at)
        .map_err(|e| Error::InvalidTimestamp {
            reason: format!("updated_at: {}", e),
        })?
        .with_timezone(&chrono::Utc);

    let issue = Issue {
        id: bd.id.clone(),
        issue_type: convert_beads_type(&bd.issue_type),
        title: bd.title,
        description: bd.description,
        status: convert_beads_status(&bd.status, &bd.close_reason, &bd.delete_reason),
        assignee: None,
        created_at,
        updated_at,
        closed_at: None,
    };

    // Start with labels
    let mut labels = bd.labels;

    // Add priority as label (only if > 0 and <= 4)
    if bd.priority > 0 && bd.priority <= 4 {
        labels.push(format!("priority:{}", bd.priority));
    }

    // Convert comments to notes (using text field)
    let mut notes: Vec<(Status, String)> = bd
        .comments
        .into_iter()
        .map(|c| (Status::Todo, c.text))
        .collect();

    // Convert dependencies using proper type mapping
    // In beads format, "depends_on_id" is the issue this one depends on.
    // For "blocks" type: depends_on_id blocks this issue (depends_on_id is the blocker)
    // For other types: this issue has the relationship to depends_on_id
    let deps: Vec<(String, String, Relation)> = bd
        .dependencies
        .into_iter()
        .map(|d| {
            let rel = convert_beads_dep_type(&d.dep_type);
            if d.dep_type == "blocks" {
                // "blocks" means depends_on_id blocks this issue
                (d.depends_on_id, bd.id.clone(), rel)
            } else {
                (bd.id.clone(), d.depends_on_id, rel)
            }
        })
        .collect();

    // Build close data if applicable (for closed or tombstone status)
    let close_data = if bd.status == "closed" {
        bd.close_reason.clone().map(|reason| {
            let is_failure = is_failure_reason(&reason);
            // Add close reason as a note (always Closed status so it shows under "Close Reason:")
            notes.push((Status::Closed, reason.clone()));
            CloseData { reason, is_failure }
        })
    } else if bd.status == "tombstone" {
        // Tombstoned issues use delete_reason as the close reason
        let reason = bd
            .delete_reason
            .clone()
            .unwrap_or_else(|| "deleted".to_string());
        notes.push((Status::Closed, reason.clone()));
        Some(CloseData {
            reason,
            is_failure: true, // Tombstone is always a "failure" (not completed)
        })
    } else {
        None
    };

    // Beads doesn't have external links
    let links: Vec<ImportedLink> = Vec::new();

    Ok((issue, labels, notes, deps, close_data, links))
}

// Convert wk issue to internal format
fn convert_wk_issue(wk: WkIssue) -> ImportedIssue {
    let notes: Vec<(Status, String)> = wk
        .notes
        .into_iter()
        .map(|n| (n.status, n.content))
        .collect();

    let deps: Vec<(String, String, Relation)> = wk
        .deps
        .into_iter()
        .map(|d| (d.from_id, d.to_id, d.relation))
        .collect();

    let links: Vec<ImportedLink> = wk
        .links
        .into_iter()
        .map(|l| ImportedLink {
            link_type: l.link_type,
            url: l.url,
            external_id: l.external_id,
            rel: l.rel,
        })
        .collect();

    (wk.issue, wk.labels, notes, deps, None, links) // wk format has no close_data
}

// TODO(refactor): Consider using an options struct to bundle parameters
#[allow(clippy::too_many_arguments)]
pub fn run(
    file: Option<String>,
    input: Option<String>,
    format: &str,
    dry_run: bool,
    status: Vec<String>,
    issue_type: Vec<String>,
    label: Vec<String>,
    prefix: Option<String>,
) -> Result<()> {
    // Determine input source
    let source = file.or(input);
    let path = match &source {
        Some(p) if p != "-" => p.as_str(),
        Some(_) => "-",
        None => return Err(Error::NoInputFile),
    };

    let (db, config, _) = open_db()?;
    run_impl(
        &db, &config, path, format, dry_run, status, issue_type, label, prefix,
    )
}

// TODO(refactor): Consider using an options struct to bundle parameters
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_impl(
    db: &Database,
    _config: &Config,
    path: &str,
    format: &str,
    dry_run: bool,
    status: Vec<String>,
    issue_type: Vec<String>,
    label: Vec<String>,
    prefix: Option<String>,
) -> Result<()> {
    // Detect format
    let format = detect_format(path, format);

    // Open input
    let reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let file = std::fs::File::open(path).map_err(|e| {
            Error::Io(std::io::Error::other(format!(
                "cannot open {}: {}",
                path, e
            )))
        })?;
        Box::new(BufReader::new(file))
    };

    // Parse filters
    let status_groups = parse_filter_groups(&status, |s| Ok(s.parse::<Status>()?))?;
    let type_groups =
        parse_filter_groups(&issue_type, |s| s.parse::<IssueType>().map_err(Into::into))?;
    let label_groups = parse_filter_groups(&label, |s| Ok(s.to_string()))?;

    // Parse input
    let mut entries: Vec<ImportedIssue> = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry = match format {
            "bd" => {
                let bd: BeadsIssue =
                    serde_json::from_str(line).map_err(|e| Error::ParseLineError {
                        line: line_num + 1,
                        reason: e.to_string(),
                    })?;
                convert_beads_issue(bd)?
            }
            _ => {
                let wk: WkIssue =
                    serde_json::from_str(line).map_err(|e| Error::ParseLineError {
                        line: line_num + 1,
                        reason: e.to_string(),
                    })?;
                convert_wk_issue(wk)
            }
        };

        entries.push(entry);
    }

    // Apply filters
    let mut filtered_entries = Vec::new();
    let mut result = ImportResult::default();

    for (issue, labels, notes, deps, close_data, links) in entries {
        // Filter by prefix
        if let Some(ref pfx) = prefix {
            if !issue.id.starts_with(pfx) {
                result.filtered += 1;
                continue;
            }
        }

        if !matches_filter_groups(&status_groups, || issue.status) {
            result.filtered += 1;
            continue;
        }
        if !matches_filter_groups(&type_groups, || issue.issue_type) {
            result.filtered += 1;
            continue;
        }
        if !matches_label_groups(&label_groups, &labels) {
            result.filtered += 1;
            continue;
        }
        filtered_entries.push((issue, labels, notes, deps, close_data, links));
    }

    // Collect existing IDs for dependency checking
    let existing_ids: HashSet<String> = db
        .list_issues(None, None, None)?
        .into_iter()
        .map(|i| i.id)
        .collect();
    let import_ids: HashSet<String> = filtered_entries
        .iter()
        .map(|(i, _, _, _, _, _)| i.id.clone())
        .collect();

    // Process imports
    for (issue, labels, notes, deps, close_data, links) in &filtered_entries {
        // Check for missing dependencies
        for (_, to_id, _) in deps {
            if !existing_ids.contains(to_id) && !import_ids.contains(to_id) {
                result.missing_deps.push((issue.id.clone(), to_id.clone()));
            }
        }

        // Check if issue exists
        match db.get_issue(&issue.id) {
            Ok(existing) => {
                // Check for collision (different content)
                if existing.title != issue.title || existing.status != issue.status {
                    result.collisions.push(issue.id.clone());
                }

                if !dry_run {
                    // Update issue
                    if existing.title != issue.title {
                        db.update_issue_title(&issue.id, &issue.title)?;
                    }
                    if existing.status != issue.status {
                        db.update_issue_status(&issue.id, issue.status)?;
                    }
                    if existing.issue_type != issue.issue_type {
                        db.update_issue_type(&issue.id, issue.issue_type)?;
                    }

                    // Sync labels
                    let existing_labels = db.get_labels(&issue.id)?;
                    for l in &existing_labels {
                        if !labels.contains(l) {
                            db.remove_label(&issue.id, l)?;
                        }
                    }
                    for l in labels {
                        if !existing_labels.contains(l) {
                            db.add_label(&issue.id, l)?;
                        }
                    }

                    // Add new notes
                    let existing_notes = db.get_notes(&issue.id)?;
                    for (status, content) in notes {
                        if !existing_notes.iter().any(|n| n.content == *content) {
                            db.add_note(&issue.id, *status, content)?;
                        }
                    }

                    // Add deps (idempotent via INSERT OR IGNORE)
                    for (from_id, to_id, rel) in deps {
                        // Only add if target exists
                        if existing_ids.contains(to_id) || import_ids.contains(to_id) {
                            let _ = db.add_dependency(from_id, to_id, *rel);
                        }
                    }

                    // Add links (check if URL already exists to avoid duplicates)
                    let existing_links = db.get_links(&issue.id)?;
                    for imported_link in links {
                        let url_exists = existing_links.iter().any(|l| l.url == imported_link.url);
                        if !url_exists {
                            let mut link = new_link(&issue.id);
                            link.link_type = imported_link.link_type;
                            link.url = imported_link.url.clone();
                            link.external_id = imported_link.external_id.clone();
                            link.rel = imported_link.rel;
                            db.add_link(&link)?;
                        }
                    }
                }
                result.updated += 1;
            }
            Err(Error::IssueNotFound(_)) => {
                if !dry_run {
                    // Create new issue
                    db.create_issue(issue)?;

                    // Add labels
                    for l in labels {
                        db.add_label(&issue.id, l)?;
                    }

                    // Add notes
                    for (status, content) in notes {
                        db.add_note(&issue.id, *status, content)?;
                    }

                    // Add deps (only if target exists or will be created)
                    for (from_id, to_id, rel) in deps {
                        if existing_ids.contains(to_id) || import_ids.contains(to_id) {
                            let _ = db.add_dependency(from_id, to_id, *rel);
                        }
                    }

                    // Log close event if applicable
                    if let Some(cd) = close_data {
                        let action = if cd.is_failure {
                            Action::Closed
                        } else {
                            Action::Done
                        };
                        let event = Event::new(issue.id.clone(), action)
                            .with_reason(Some(cd.reason.clone()));
                        db.log_event(&event)?;
                    }

                    // Add links
                    for imported_link in links {
                        let mut link = new_link(&issue.id);
                        link.link_type = imported_link.link_type;
                        link.url = imported_link.url.clone();
                        link.external_id = imported_link.external_id.clone();
                        link.rel = imported_link.rel;
                        db.add_link(&link)?;
                    }
                }
                result.created += 1;
            }
            Err(e) => return Err(e),
        }
    }

    // Print results
    if dry_run {
        println!("Dry run - no changes made");
    }

    println!("Import summary:");
    if result.created > 0 {
        println!("  create: {}", result.created);
    }
    if result.updated > 0 {
        println!("  update: {}", result.updated);
    }
    if result.filtered > 0 {
        println!("  filtered: {}", result.filtered);
    }

    if !result.collisions.is_empty() {
        eprintln!(
            "\nwarning: {} collision(s) detected:",
            result.collisions.len()
        );
        for id in &result.collisions {
            eprintln!("  - {}", id);
        }
    }

    if !result.missing_deps.is_empty() {
        eprintln!(
            "\nwarning: {} missing dependency reference(s):",
            result.missing_deps.len()
        );
        for (issue_id, dep_id) in &result.missing_deps {
            eprintln!("  - {} references nonexistent {}", issue_id, dep_id);
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "import_tests.rs"]
mod tests;
