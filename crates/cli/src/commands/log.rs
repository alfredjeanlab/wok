// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;
use crate::display::{format_event, format_event_with_id};
use crate::error::Result;

use super::open_db;

/// Default limit for log output when not explicitly specified.
const DEFAULT_LIMIT: usize = 20;

pub fn run(id: Option<String>, limit: Option<usize>, no_limit: bool) -> Result<()> {
    let (db, _, _) = open_db()?;
    let effective_limit = if no_limit {
        0
    } else {
        limit.unwrap_or(DEFAULT_LIMIT)
    };
    run_impl(&db, id, effective_limit)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, id: Option<String>, limit: usize) -> Result<()> {
    match id {
        Some(issue_id) => {
            // Resolve potentially partial ID
            let resolved_id = db.resolve_id(&issue_id)?;

            // Show events for specific issue
            let events = db.get_events(&resolved_id)?;

            if events.is_empty() {
                println!("No events for {}", resolved_id);
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
