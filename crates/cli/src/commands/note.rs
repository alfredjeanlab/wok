// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;

use super::{apply_mutation, open_db};
use crate::error::{Error, Result};
use crate::models::{Action, Event, Status};
use crate::validate::validate_and_trim_note;

pub fn run(id: &str, content: &str, replace: bool) -> Result<()> {
    let (db, _config, _work_dir) = open_db()?;
    run_impl(&db, id, content, replace)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, id: &str, content: &str, replace: bool) -> Result<()> {
    let resolved_id = db.resolve_id(id)?;
    let issue = db.get_issue(&resolved_id)?;

    // Cannot add notes to closed issues
    if issue.status == Status::Closed {
        return Err(Error::CannotNoteClosedIssue);
    }

    // Validate and trim note
    let trimmed_content = validate_and_trim_note(content)?;

    // Check if empty after trimming
    if trimmed_content.is_empty() {
        return Err(Error::FieldEmpty { field: "Note" });
    }

    if replace {
        db.replace_note(&resolved_id, issue.status, &trimmed_content)?;

        apply_mutation(
            db,
            Event::new(resolved_id.clone(), Action::Noted).with_values(None, Some(trimmed_content)),
        )?;

        println!("Replaced note on {}", resolved_id);
    } else {
        db.add_note(&resolved_id, issue.status, &trimmed_content)?;

        apply_mutation(
            db,
            Event::new(resolved_id.clone(), Action::Noted).with_values(None, Some(trimmed_content)),
        )?;

        println!("Added note to {} ({})", resolved_id, issue.status);
    }

    Ok(())
}

#[cfg(test)]
#[path = "note_tests.rs"]
mod tests;
