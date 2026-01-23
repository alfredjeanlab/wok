// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use serde::Serialize;

use crate::db::Database;
use crate::display::format_issue_details;
use crate::error::{Error, Result};
use crate::models::{Event, Issue, Link, Note};

use super::open_db;

#[derive(Serialize)]
struct IssueDetails {
    #[serde(flatten)]
    issue: Issue,
    labels: Vec<String>,
    blockers: Vec<String>,
    blocking: Vec<String>,
    parents: Vec<String>,
    children: Vec<String>,
    notes: Vec<Note>,
    links: Vec<Link>,
    events: Vec<Event>,
}

pub fn run(id: &str, format: &str) -> Result<()> {
    let (db, _, _) = open_db()?;
    run_impl(&db, id, format)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, id: &str, format: &str) -> Result<()> {
    let issue = db.get_issue(id)?;
    let labels = db.get_labels(id)?;
    let blockers = db.get_blockers(id)?;
    let blocking = db.get_blocking(id)?;
    let parents = db.get_tracking(id)?;
    let children = db.get_tracked(id)?;
    let links = db.get_links(id)?;
    let events = db.get_events(id)?;

    match format {
        "json" => {
            let notes = db.get_notes(id)?;
            let details = IssueDetails {
                issue,
                labels,
                blockers,
                blocking,
                parents,
                children,
                notes,
                links,
                events,
            };
            let json = serde_json::to_string_pretty(&details)?;
            println!("{}", json);
        }
        "text" => {
            let notes = db.get_notes_by_status(id)?;
            let output = format_issue_details(
                &issue, &labels, &blockers, &blocking, &parents, &children, &notes, &links, &events,
            );
            println!("{}", output);
        }
        _ => {
            return Err(Error::UnknownFormat {
                format: format.to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "show_tests.rs"]
mod tests;
