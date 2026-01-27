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

pub fn run(ids: &[String], format: &str) -> Result<()> {
    let (db, _, _) = open_db()?;
    run_impl(&db, ids, format)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, ids: &[String], format: &str) -> Result<()> {
    // Resolve all IDs first (fail fast if any is invalid)
    let resolved_ids: Vec<String> = ids
        .iter()
        .map(|id| db.resolve_id(id))
        .collect::<Result<Vec<_>>>()?;

    match format {
        "json" => output_json(db, &resolved_ids),
        "text" => output_text(db, &resolved_ids),
        _ => Err(Error::UnknownFormat {
            format: format.to_string(),
        }),
    }
}

fn build_issue_details(db: &Database, id: &str) -> Result<IssueDetails> {
    let issue = db.get_issue(id)?;
    let labels = db.get_labels(id)?;
    let blockers = db.get_blockers(id)?;
    let blocking = db.get_blocking(id)?;
    let parents = db.get_tracking(id)?;
    let children = db.get_tracked(id)?;
    let notes = db.get_notes(id)?;
    let links = db.get_links(id)?;
    let events = db.get_events(id)?;

    Ok(IssueDetails {
        issue,
        labels,
        blockers,
        blocking,
        parents,
        children,
        notes,
        links,
        events,
    })
}

fn output_json(db: &Database, ids: &[String]) -> Result<()> {
    for id in ids {
        let details = build_issue_details(db, id)?;
        // Use to_string (not to_string_pretty) for JSONL format
        let json = serde_json::to_string(&details)?;
        println!("{json}");
    }
    Ok(())
}

fn output_text(db: &Database, ids: &[String]) -> Result<()> {
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            println!("---");
        }
        output_single_text(db, id)?;
    }
    Ok(())
}

fn output_single_text(db: &Database, id: &str) -> Result<()> {
    let issue = db.get_issue(id)?;
    let labels = db.get_labels(id)?;
    let blockers = db.get_blockers(id)?;
    let blocking = db.get_blocking(id)?;
    let parents = db.get_tracking(id)?;
    let children = db.get_tracked(id)?;
    let notes = db.get_notes_by_status(id)?;
    let links = db.get_links(id)?;
    let events = db.get_events(id)?;

    print!(
        "{}",
        format_issue_details(
            &issue, &labels, &blockers, &blocking, &parents, &children, &notes, &links, &events,
        )
    );
    Ok(())
}

#[cfg(test)]
#[path = "show_tests.rs"]
mod tests;
