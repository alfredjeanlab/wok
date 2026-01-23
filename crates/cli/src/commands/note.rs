// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use wk_core::OpPayload;

use crate::config::Config;
use crate::db::Database;

use super::{apply_mutation, open_db};
use crate::error::{Error, Result};
use crate::models::{Action, Event, Status};
use crate::validate::validate_and_trim_note;

pub fn run(id: &str, content: &str, replace: bool) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    run_impl(&db, &config, &work_dir, id, content, replace)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn run_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    content: &str,
    replace: bool,
) -> Result<()> {
    let issue = db.get_issue(id)?;

    // Cannot add notes to closed issues
    if issue.status == Status::Closed {
        return Err(Error::InvalidInput(
            "cannot add notes to closed issues".to_string(),
        ));
    }

    // Validate and trim note
    let trimmed_content = validate_and_trim_note(content)?;

    // Check if empty after trimming
    if trimmed_content.is_empty() {
        return Err(Error::InvalidInput(
            "Note content cannot be empty".to_string(),
        ));
    }

    // Convert status for sync
    let core_status: wk_core::Status = issue.status.into();

    if replace {
        db.replace_note(id, issue.status, &trimmed_content)?;

        apply_mutation(
            db,
            work_dir,
            config,
            Event::new(id.to_string(), Action::Noted)
                .with_values(None, Some(trimmed_content.clone())),
            Some(OpPayload::add_note(
                id.to_string(),
                trimmed_content,
                core_status,
            )),
        )?;

        println!("Replaced note on {}", id);
    } else {
        db.add_note(id, issue.status, &trimmed_content)?;

        apply_mutation(
            db,
            work_dir,
            config,
            Event::new(id.to_string(), Action::Noted)
                .with_values(None, Some(trimmed_content.clone())),
            Some(OpPayload::add_note(
                id.to_string(),
                trimmed_content,
                core_status,
            )),
        )?;

        println!("Added note to {} ({})", id, issue.status);
    }

    Ok(())
}

#[cfg(test)]
#[path = "note_tests.rs"]
mod tests;
