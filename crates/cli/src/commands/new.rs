// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use chrono::Utc;
use wk_core::OpPayload;

use crate::config::{find_work_dir, get_db_path, Config};
use crate::db::Database;
use crate::error::Result;
use crate::id::generate_unique_id;
use crate::models::{Action, Event, Issue, IssueType, Status};
use crate::validate::{
    validate_and_normalize_title, validate_and_trim_note, validate_assignee, validate_label,
};

use super::link::add_link_impl;
use super::queue_op;

#[allow(clippy::too_many_arguments)]
pub fn run(
    type_or_title: String,
    title: Option<String>,
    labels: Vec<String>,
    note: Option<String>,
    links: Vec<String>,
    assignee: Option<String>,
    priority: Option<u8>,
    description: Option<String>,
) -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
    run_impl(
        &db,
        &config,
        &work_dir,
        type_or_title,
        title,
        labels,
        note,
        links,
        assignee,
        priority,
        description,
    )
}

/// Expand comma-separated labels into individual labels.
/// For example, ["a,b", "c"] becomes ["a", "b", "c"].
fn expand_labels(labels: &[String]) -> Vec<String> {
    labels
        .iter()
        .flat_map(|label| {
            label
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(String::from)
        })
        .collect()
}

/// Internal implementation that accepts db/config for testing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    type_or_title: String,
    title: Option<String>,
    labels: Vec<String>,
    note: Option<String>,
    links: Vec<String>,
    assignee: Option<String>,
    priority: Option<u8>,
    description: Option<String>,
) -> Result<()> {
    // Expand comma-separated labels into individual labels
    let mut labels = expand_labels(&labels);

    // Convert priority to label if provided
    if let Some(p) = priority {
        labels.push(format!("priority:{}", p));
    }

    // Validate assignee if provided
    let assignee = if let Some(ref a) = assignee {
        validate_assignee(a)?;
        Some(a.trim().to_string())
    } else {
        None
    };

    // Merge description into note - description is a hidden alias for note.
    // If both are provided, note (documented flag) takes precedence.
    let effective_note = note.or(description);

    // Determine issue type and title
    let (issue_type, raw_title) = if let Some(t) = title {
        // First arg is type, second is title
        let issue_type: IssueType = type_or_title.parse()?;
        (issue_type, t)
    } else {
        // First arg is title, type defaults to task
        (IssueType::Task, type_or_title)
    };

    // Normalize and validate title (may extract description)
    let normalized = validate_and_normalize_title(&raw_title)?;

    // Combine extracted description with explicit note
    let final_note = match (effective_note, normalized.extracted_description) {
        (Some(note), Some(extracted)) => Some(format!("{}\n\n{}", extracted, note)),
        (Some(note), None) => Some(note),
        (None, Some(extracted)) => Some(extracted),
        (None, None) => None,
    };

    // Validate that prefix is not empty - empty prefix would create IDs like "-a1b2"
    // which cause CLI issues because they look like flags
    if config.prefix.is_empty() {
        return Err(crate::error::Error::InvalidInput(
            "cannot create issue: project has no prefix configured\n  hint: workspace links without a prefix can only view issues, not create them".to_string(),
        ));
    }

    let created_at = Utc::now();
    let id = generate_unique_id(&config.prefix, &normalized.title, &created_at, |id| {
        db.issue_exists(id).unwrap_or(false)
    });

    let issue = Issue {
        id: id.clone(),
        issue_type,
        title: normalized.title.clone(),
        description: None,
        status: Status::Todo,
        assignee,
        created_at,
        updated_at: created_at,
        closed_at: None,
    };

    db.create_issue(&issue)?;

    // Log creation event
    let event = Event::new(id.clone(), Action::Created);
    db.log_event(&event)?;

    // Queue CreateIssue op for sync (convert CLI IssueType to core IssueType)
    let core_issue_type = issue_type
        .as_str()
        .parse()
        .unwrap_or(wk_core::IssueType::Task);
    queue_op(
        work_dir,
        config,
        OpPayload::create_issue(id.clone(), core_issue_type, normalized.title.clone()),
    )?;

    // Validate and add labels
    for label in &labels {
        validate_label(label)?;
        db.add_label(&id, label)?;
        let event = Event::new(id.clone(), Action::Labeled).with_values(None, Some(label.clone()));
        db.log_event(&event)?;

        // Queue AddLabel op for sync
        queue_op(
            work_dir,
            config,
            OpPayload::add_label(id.clone(), label.clone()),
        )?;
    }

    // Add note if provided (note or description flag or extracted)
    if let Some(note_content) = final_note {
        let trimmed_note = validate_and_trim_note(&note_content)?;
        if !trimmed_note.is_empty() {
            db.add_note(&id, Status::Todo, &trimmed_note)?;
            let event =
                Event::new(id.clone(), Action::Noted).with_values(None, Some(trimmed_note.clone()));
            db.log_event(&event)?;

            // Queue AddNote op for sync
            queue_op(
                work_dir,
                config,
                OpPayload::add_note(id.clone(), trimmed_note, wk_core::Status::Todo),
            )?;
        }
    }

    // Add links if provided
    for link_url in &links {
        add_link_impl(db, &id, link_url)?;
    }

    println!(
        "Created [{}] ({}) {}: {}",
        issue_type, issue.status, id, normalized.title
    );

    Ok(())
}

#[cfg(test)]
#[path = "new_tests.rs"]
mod tests;
