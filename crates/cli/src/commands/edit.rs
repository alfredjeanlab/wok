// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;
use std::str::FromStr;

use wk_core::OpPayload;

use crate::config::Config;
use crate::db::Database;

use super::open_db;
use crate::error::{Error, Result};
use crate::models::{Action, Event, IssueType};
use crate::validate::{
    validate_and_normalize_title, validate_and_trim_description, validate_assignee,
};

use super::queue_op;

pub fn run(id: &str, attr: &str, value: &str) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    run_impl(&db, &config, &work_dir, id, attr, value)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn run_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    attr: &str,
    value: &str,
) -> Result<()> {
    let issue = db.get_issue(id)?;

    match attr.to_lowercase().as_str() {
        "title" => {
            let normalized = validate_and_normalize_title(value)?;

            if normalized.extracted_description.is_some() {
                return Err(Error::InvalidInput(
                    "Title contains double-newline; use 'wk note' for description".to_string(),
                ));
            }

            let old_title = issue.title.clone();
            db.update_issue_title(id, &normalized.title)?;

            let event = Event::new(id.to_string(), Action::Edited)
                .with_values(Some(old_title), Some(normalized.title.clone()));
            db.log_event(&event)?;

            // Queue SetTitle op for sync
            queue_op(
                work_dir,
                config,
                OpPayload::set_title(id.to_string(), normalized.title.clone()),
            )?;

            println!("Updated title of {} to: {}", id, normalized.title);
        }
        "type" => {
            let new_type = IssueType::from_str(value)?;
            let old_type = issue.issue_type;

            if new_type != old_type {
                db.update_issue_type(id, new_type)?;

                let event = Event::new(id.to_string(), Action::Edited).with_values(
                    Some(old_type.as_str().to_string()),
                    Some(new_type.as_str().to_string()),
                );
                db.log_event(&event)?;

                // Queue SetType op for sync
                queue_op(
                    work_dir,
                    config,
                    OpPayload::set_type(id.to_string(), new_type),
                )?;

                println!("Updated type of {} to: {}", id, new_type.as_str());
            }
        }
        "description" => {
            let trimmed_desc = validate_and_trim_description(value)?;
            let old_desc = issue.description.clone();
            db.update_issue_description(id, &trimmed_desc)?;

            let event = Event::new(id.to_string(), Action::Edited)
                .with_values(old_desc, Some(trimmed_desc.clone()));
            db.log_event(&event)?;

            println!("Updated description of {}", id);
        }
        "assignee" => {
            let old_assignee = issue.assignee.clone();
            let trimmed = value.trim();

            // Clear assignee if value is empty or "none"
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
                if old_assignee.is_none() {
                    println!("{} is not assigned", id);
                } else {
                    db.clear_assignee(id)?;

                    let event = Event::new(id.to_string(), Action::Unassigned)
                        .with_values(old_assignee, None);
                    db.log_event(&event)?;

                    println!("Unassigned {}", id);
                }
            } else {
                validate_assignee(trimmed)?;
                db.set_assignee(id, trimmed)?;

                let event = Event::new(id.to_string(), Action::Assigned)
                    .with_values(old_assignee, Some(trimmed.to_string()));
                db.log_event(&event)?;

                println!("Assigned {} to {}", id, trimmed);
            }
        }
        _ => {
            return Err(Error::InvalidInput(format!(
                "Unknown attribute '{}'. Valid attributes: title, description, type, assignee",
                attr
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
