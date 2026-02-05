// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::models::{Action, Event, Issue, Link, Note, Status};

/// Maximum line width for wrapped text content (excluding 4-space indent).
const WRAP_WIDTH: usize = 96;

/// Map issue status to semantic note section label.
///
/// - `todo` → "Description" (requirements, context before work starts)
/// - `in_progress` → "Progress" (updates during active work)
/// - `done` → "Summary" (what was accomplished)
/// - `closed` → "Close Reason" (why the issue was closed without completion)
pub fn note_section_label(status: Status) -> &'static str {
    match status {
        Status::Todo => "Description",
        Status::InProgress => "Progress",
        Status::Done => "Summary",
        Status::Closed => "Close Reason",
    }
}

/// Wrap text at word boundaries if it's a single line.
///
/// - If content contains newlines: return as-is (preserve user formatting)
/// - If content is single line >width: wrap at word boundaries
/// - If content is single line <=width: return as-is
pub fn wrap_text(content: &str, width: usize) -> String {
    // If content contains newlines, preserve exactly
    if content.contains('\n') {
        return content.to_string();
    }

    // If fits in width, return as-is
    if content.len() <= width {
        return content.to_string();
    }

    // Wrap at word boundaries
    let mut result = String::new();
    let mut current_line = String::new();

    for word in content.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&current_line);
    }

    result
}

/// Format a single note with metadata line and indented content.
///
/// Output format:
/// ```text
///   2024-01-10 10:30
///     Content goes here, potentially
///     wrapped across multiple lines.
/// ```
pub fn format_note(note: &Note) -> Vec<String> {
    let mut lines = Vec::new();

    // Metadata line: 2 spaces + timestamp
    let timestamp = note.created_at.format("%Y-%m-%d %H:%M");
    lines.push(format!("  {}", timestamp));

    // Content: wrap if single line, then indent each line with 4 spaces
    let wrapped = wrap_text(&note.content, WRAP_WIDTH);
    for line in wrapped.lines() {
        lines.push(format!("    {}", line));
    }

    lines
}

/// Format a single issue line for list output
pub fn format_issue_line(issue: &Issue) -> String {
    let status_display = match &issue.assignee {
        Some(assignee) => format!("{}, @{}", issue.status, assignee),
        None => issue.status.to_string(),
    };
    format!(
        "- [{}] ({}) {}: {}",
        issue.issue_type, status_display, issue.id, issue.title
    )
}

/// Format issue details for show command
#[allow(clippy::too_many_arguments)] // TODO(refactor): Consider using an options struct to bundle parameters
pub fn format_issue_details(
    issue: &Issue,
    labels: &[String],
    blockers: &[String],
    blocking: &[String],
    parents: &[String],
    children: &[String],
    notes: &[(Status, Vec<Note>)],
    links: &[Link],
    events: &[Event],
) -> String {
    let mut output = Vec::new();

    // Header: [type] id
    output.push(format!("[{}] {}", issue.issue_type, issue.id));

    // Metadata on separate lines
    output.push(format!("Title: {}", issue.title));
    output.push(format!("Status: {}", issue.status));
    if let Some(assignee) = &issue.assignee {
        output.push(format!("Assignee: {}", assignee));
    }
    output.push(format!(
        "Created: {}",
        issue.created_at.format("%Y-%m-%d %H:%M")
    ));
    output.push(format!(
        "Updated: {}",
        issue.updated_at.format("%Y-%m-%d %H:%M")
    ));

    // Labels
    if !labels.is_empty() {
        output.push(format!("Labels: {}", labels.join(", ")));
    }

    // Blocked by
    if !blockers.is_empty() {
        output.push(String::new());
        output.push("Blocked by:".to_string());
        for id in blockers {
            output.push(format!("  - {}", id));
        }
    }

    // Blocks
    if !blocking.is_empty() {
        output.push(String::new());
        output.push("Blocks:".to_string());
        for id in blocking {
            output.push(format!("  - {}", id));
        }
    }

    // Tracked by
    if !parents.is_empty() {
        output.push(String::new());
        output.push("Tracked by:".to_string());
        for id in parents {
            output.push(format!("  - {}", id));
        }
    }

    // Tracks
    if !children.is_empty() {
        output.push(String::new());
        output.push("Tracks:".to_string());
        for id in children {
            output.push(format!("  - {}", id));
        }
    }

    // External links
    if !links.is_empty() {
        output.push(String::new());
        output.push("Links:".to_string());
        for link in links {
            output.push(format_link(link));
        }
    }

    // Notes grouped by status with semantic labels
    for (status, status_notes) in notes {
        if !status_notes.is_empty() {
            output.push(String::new());
            output.push(format!("{}:", note_section_label(*status)));
            for (i, note) in status_notes.iter().enumerate() {
                // Add blank line between notes within a section
                if i > 0 {
                    output.push(String::new());
                }
                output.extend(format_note(note));
            }
        }
    }

    // Event log (skip Created event since it's redundant with Created: line,
    // and skip Noted events at creation time since they appear in Description section)
    let filtered_events: Vec<_> = events
        .iter()
        .filter(|e| {
            if e.action == Action::Created {
                return false;
            }
            if e.action == Action::Noted && e.created_at == issue.created_at {
                return false;
            }
            true
        })
        .collect();
    if !filtered_events.is_empty() {
        output.push(String::new());
        output.push("Log:".to_string());
        for event in filtered_events {
            output.push(format_event(event));
        }
    }

    output.join("\n")
}

/// Format a single external link for display.
fn format_link(link: &Link) -> String {
    let mut parts = Vec::new();

    // Add link type if known
    if let Some(link_type) = &link.link_type {
        parts.push(format!("[{}]", link_type));
    }

    // Add URL or external ID
    if let Some(url) = &link.url {
        parts.push(url.clone());
    } else if let Some(ext_id) = &link.external_id {
        parts.push(ext_id.clone());
    }

    // Add relationship if present
    if let Some(rel) = &link.rel {
        parts.push(format!("({})", rel));
    }

    format!("  - {}", parts.join(" "))
}

/// Format a single event for log output
pub fn format_event(event: &Event) -> String {
    let timestamp = event.created_at.format("%Y-%m-%d %H:%M");
    let mut line = format!("  {}  {}", timestamp, event.action);

    match event.action {
        Action::Edited => {
            // Only show new value, not old value (to avoid leaking old titles)
            if let Some(new) = &event.new_value {
                line.push_str(&format!(" -> {}", new));
            }
        }
        Action::Labeled | Action::Unlabeled => {
            if let Some(val) = &event.new_value {
                line.push_str(&format!(" {}", val));
            }
        }
        Action::Related | Action::Unrelated => {
            if let Some(val) = &event.new_value {
                line.push_str(&format!(" {}", val));
            }
        }
        Action::Linked | Action::Unlinked => {
            if let Some(val) = &event.new_value {
                line.push_str(&format!(" {}", val));
            }
        }
        Action::Done | Action::Closed | Action::Reopened => {
            if let Some(reason) = &event.reason {
                line.push_str(&format!(" \"{}\"", reason));
            }
        }
        Action::Noted => {
            if let Some(val) = &event.new_value {
                // Truncate long notes
                let display = if val.len() > 50 {
                    format!("{}...", &val[..47])
                } else {
                    val.clone()
                };
                line.push_str(&format!(" \"{}\"", display));
            }
        }
        Action::Assigned => {
            if let Some(val) = &event.new_value {
                line.push_str(&format!(" to {}", val));
            }
        }
        Action::Unassigned => {
            if let Some(val) = &event.old_value {
                line.push_str(&format!(" (was {})", val));
            }
        }
        _ => {}
    }

    line
}

/// Format event with issue ID (for global log)
pub fn format_event_with_id(event: &Event) -> String {
    let timestamp = event.created_at.format("%Y-%m-%d %H:%M");
    let mut line = format!("  {}  {} {}", timestamp, event.issue_id, event.action);

    match event.action {
        Action::Done | Action::Closed | Action::Reopened => {
            if let Some(reason) = &event.reason {
                line.push_str(&format!(" \"{}\"", reason));
            }
        }
        Action::Labeled | Action::Unlabeled => {
            if let Some(val) = &event.new_value {
                line.push_str(&format!(" {}", val));
            }
        }
        _ => {}
    }

    line
}

/// Type of relationship for tree display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// Issue is tracked by the parent
    Tracks,
    /// Issue is blocked by the parent
    Blocks,
}

impl RelationType {
    /// Returns the display label for this relation type
    pub fn label(&self) -> &'static str {
        match self {
            RelationType::Tracks => "tracks",
            RelationType::Blocks => "blocks",
        }
    }
}

/// Format tree output for root node
pub fn format_tree_root(issue: &Issue, blocked_by: Option<&[String]>) -> String {
    let status_str = if issue.status != Status::Todo {
        format!(" [{}]", issue.status)
    } else {
        String::new()
    };

    let mut output = format!("{}: {}{}", issue.id, issue.title, status_str);

    // Show blockers if any
    if let Some(blockers) = blocked_by {
        if !blockers.is_empty() {
            output.push_str(&format!("\n└── (blocked by {})", blockers.join(", ")));
        }
    }

    output
}

/// Format tree output for child node
pub fn format_tree_child(
    issue: &Issue,
    prefix: &str,
    is_last: bool,
    blocked_by: Option<&[String]>,
    relation_label: Option<RelationType>,
) -> Vec<String> {
    let mut lines = Vec::new();

    let connector = if is_last { "└── " } else { "├── " };

    let status_str = if issue.status != Status::Todo {
        format!(" [{}]", issue.status)
    } else {
        String::new()
    };

    let label_str = match relation_label {
        Some(rel) => format!(" ({})", rel.label()),
        None => String::new(),
    };

    lines.push(format!(
        "{}{}{}: {}{}{}",
        prefix, connector, issue.id, issue.title, status_str, label_str
    ));

    // Show blockers if any
    if let Some(blockers) = blocked_by {
        if !blockers.is_empty() {
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            lines.push(format!(
                "{}└── (blocked by {})",
                child_prefix,
                blockers.join(", ")
            ));
        }
    }

    lines
}

#[cfg(test)]
#[path = "display_tests.rs"]
mod tests;
