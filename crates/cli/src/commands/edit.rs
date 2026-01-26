// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;
use std::str::FromStr;

use wk_core::OpPayload;

use crate::config::Config;
use crate::db::Database;

use super::{apply_mutation, open_db};
use crate::error::{Error, Result};
use crate::models::{Action, Event, IssueType};
use crate::validate::{validate_and_normalize_title, validate_and_trim_description, validate_assignee};

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
    let resolved_id = db.resolve_id(id)?;
    let issue = db.get_issue(&resolved_id)?;

    match attr.to_lowercase().as_str() {
        "title" => {
            let normalized = validate_and_normalize_title(value)?;

            if normalized.extracted_description.is_some() {
                return Err(Error::TitleContainsDescription);
            }

            let old_title = issue.title.clone();
            db.update_issue_title(&resolved_id, &normalized.title)?;

            apply_mutation(
                db,
                work_dir,
                config,
                Event::new(resolved_id.clone(), Action::Edited)
                    .with_values(Some(old_title), Some(normalized.title.clone())),
                Some(OpPayload::set_title(
                    resolved_id.clone(),
                    normalized.title.clone(),
                )),
            )?;

            println!("Updated title of {} to: {}", resolved_id, normalized.title);
        }
        "type" => {
            let new_type = IssueType::from_str(value)?;
            let old_type = issue.issue_type;

            if new_type != old_type {
                db.update_issue_type(&resolved_id, new_type)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(resolved_id.clone(), Action::Edited).with_values(
                        Some(old_type.as_str().to_string()),
                        Some(new_type.as_str().to_string()),
                    ),
                    Some(OpPayload::set_type(resolved_id.clone(), new_type)),
                )?;

                println!("Updated type of {} to: {}", resolved_id, new_type.as_str());
            }
        }
        "description" => {
            let trimmed_desc = validate_and_trim_description(value)?;
            let old_desc = issue.description.clone();
            db.update_issue_description(&resolved_id, &trimmed_desc)?;

            apply_mutation(
                db,
                work_dir,
                config,
                Event::new(resolved_id.clone(), Action::Edited)
                    .with_values(old_desc, Some(trimmed_desc.clone())),
                None, // No sync for description edits
            )?;

            println!("Updated description of {}", resolved_id);
        }
        "assignee" => {
            let old_assignee = issue.assignee.clone();
            let trimmed = value.trim();

            // Clear assignee if value is empty or "none"
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
                if old_assignee.is_none() {
                    println!("{} is not assigned", resolved_id);
                } else {
                    db.clear_assignee(&resolved_id)?;

                    apply_mutation(
                        db,
                        work_dir,
                        config,
                        Event::new(resolved_id.clone(), Action::Unassigned)
                            .with_values(old_assignee, None),
                        None, // No sync for assignee changes
                    )?;

                    println!("Unassigned {}", resolved_id);
                }
            } else {
                validate_assignee(trimmed)?;
                db.set_assignee(&resolved_id, trimmed)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(resolved_id.clone(), Action::Assigned)
                        .with_values(old_assignee, Some(trimmed.to_string())),
                    None, // No sync for assignee changes
                )?;

                println!("Assigned {} to {}", resolved_id, trimmed);
            }
        }
        _ => {
            return Err(Error::UnknownAttribute {
                attr: attr.to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
