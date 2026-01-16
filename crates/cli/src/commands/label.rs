// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;
use crate::error::Result;
use crate::models::{Action, Event};
use crate::validate::{validate_label, validate_label_count};

use super::open_db;

pub fn add(ids: &[String], label: &str) -> Result<()> {
    let (db, _) = open_db()?;
    add_impl(&db, ids, label)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn add_impl(db: &Database, ids: &[String], label: &str) -> Result<()> {
    // Validate label once (applies to all)
    validate_label(label)?;

    for id in ids {
        add_single(db, id, label)?;
    }
    Ok(())
}

fn add_single(db: &Database, id: &str, label: &str) -> Result<()> {
    // Verify issue exists
    db.get_issue(id)?;

    // Validate label count
    let current_labels = db.get_labels(id)?;
    validate_label_count(current_labels.len())?;

    db.add_label(id, label)?;

    let event =
        Event::new(id.to_string(), Action::Labeled).with_values(None, Some(label.to_string()));
    db.log_event(&event)?;

    println!("Labeled {} with {}", id, label);

    Ok(())
}

pub fn remove(ids: &[String], label: &str) -> Result<()> {
    let (db, _) = open_db()?;
    remove_impl(&db, ids, label)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn remove_impl(db: &Database, ids: &[String], label: &str) -> Result<()> {
    for id in ids {
        remove_single(db, id, label)?;
    }
    Ok(())
}

fn remove_single(db: &Database, id: &str, label: &str) -> Result<()> {
    // Verify issue exists
    db.get_issue(id)?;

    let removed = db.remove_label(id, label)?;

    if removed {
        let event = Event::new(id.to_string(), Action::Unlabeled)
            .with_values(None, Some(label.to_string()));
        db.log_event(&event)?;

        println!("Removed label {} from {}", label, id);
    } else {
        println!("Label {} not found on {}", label, id);
    }

    Ok(())
}

#[cfg(test)]
#[path = "label_tests.rs"]
mod tests;
