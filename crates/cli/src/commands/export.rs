// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::fs::File;
use std::io::{BufWriter, Write};

use serde::Serialize;

use crate::db::Database;
use crate::error::Result;
use crate::models::{Dependency, Event, Issue, Link, Note};
use crate::validate::validate_export_path;

use super::open_db;

#[derive(Serialize)]
struct ExportedIssue {
    #[serde(flatten)]
    issue: Issue,
    labels: Vec<String>,
    notes: Vec<Note>,
    deps: Vec<Dependency>,
    links: Vec<Link>,
    events: Vec<Event>,
}

pub fn run(filepath: &str) -> Result<()> {
    // Validate export path
    validate_export_path(filepath)?;

    let (db, _, _) = open_db()?;
    run_impl(&db, filepath)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, filepath: &str) -> Result<()> {
    let issues = db.get_all_issues()?;
    let file = File::create(filepath)?;
    let mut writer = BufWriter::new(file);

    let mut count = 0;
    for issue in issues {
        let labels = db.get_labels(&issue.id)?;
        let notes = db.get_notes(&issue.id)?;
        let deps = db.get_deps_from(&issue.id)?;
        let links = db.get_links(&issue.id)?;
        let events = db.get_events(&issue.id)?;

        let exported = ExportedIssue {
            issue,
            labels,
            notes,
            deps,
            links,
            events,
        };

        let json = serde_json::to_string(&exported)?;
        writeln!(writer, "{}", json)?;
        count += 1;
    }

    writer.flush()?;
    println!("Exported {} issues to {}", count, filepath);

    Ok(())
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;
