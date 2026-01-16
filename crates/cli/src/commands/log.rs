// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;
use crate::display::{format_event, format_event_with_id};
use crate::error::{Error, Result};

use super::open_db;

pub fn run(id: Option<String>, limit: usize) -> Result<()> {
    let (db, _) = open_db()?;
    run_impl(&db, id, limit)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, id: Option<String>, limit: usize) -> Result<()> {
    match id {
        Some(issue_id) => {
            // Verify issue exists first
            if !db.issue_exists(&issue_id)? {
                return Err(Error::IssueNotFound(issue_id));
            }

            // Show events for specific issue
            let events = db.get_events(&issue_id)?;

            if events.is_empty() {
                println!("No events for {}", issue_id);
            } else {
                for event in events.iter().rev().take(limit) {
                    println!("{}", format_event(event));
                }
            }
        }
        None => {
            // Show recent events across all issues
            let events = db.get_recent_events(limit)?;

            if events.is_empty() {
                println!("No events");
            } else {
                for event in &events {
                    println!("{}", format_event_with_id(event));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "log_tests.rs"]
mod tests;
