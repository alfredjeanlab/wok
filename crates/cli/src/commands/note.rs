// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{Action, Event, Status};
use crate::validate::validate_and_trim_note;

use super::open_db;

pub fn run(id: &str, content: &str, replace: bool) -> Result<()> {
    let (db, _) = open_db()?;
    run_impl(&db, id, content, replace)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, id: &str, content: &str, replace: bool) -> Result<()> {
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

    if replace {
        db.replace_note(id, issue.status, &trimmed_content)?;

        let event =
            Event::new(id.to_string(), Action::Noted).with_values(None, Some(trimmed_content));
        db.log_event(&event)?;

        println!("Replaced note on {}", id);
    } else {
        db.add_note(id, issue.status, &trimmed_content)?;

        let event =
            Event::new(id.to_string(), Action::Noted).with_values(None, Some(trimmed_content));
        db.log_event(&event)?;

        println!("Added note to {} ({})", id, issue.status);
    }

    Ok(())
}

#[cfg(test)]
#[path = "note_tests.rs"]
mod tests;
