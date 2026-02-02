// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;

use super::{apply_mutation, open_db};
use crate::error::Result;
use crate::models::{Action, Event};
use crate::validate::{validate_label, validate_label_count};

pub fn add(ids: &[String], label: &str) -> Result<()> {
    let (db, _config, _work_dir) = open_db()?;
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
    // Resolve potentially partial ID
    let resolved_id = db.resolve_id(id)?;

    // Verify issue exists
    db.get_issue(&resolved_id)?;

    // Validate label count
    let current_labels = db.get_labels(&resolved_id)?;
    validate_label_count(current_labels.len())?;

    db.add_label(&resolved_id, label)?;

    apply_mutation(
        db,
        Event::new(resolved_id.clone(), Action::Labeled).with_values(None, Some(label.to_string())),
    )?;

    println!("Labeled {} with {}", resolved_id, label);

    Ok(())
}

pub fn remove(ids: &[String], label: &str) -> Result<()> {
    let (db, _config, _work_dir) = open_db()?;
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
    // Resolve potentially partial ID
    let resolved_id = db.resolve_id(id)?;

    // Verify issue exists
    db.get_issue(&resolved_id)?;

    let removed = db.remove_label(&resolved_id, label)?;

    if removed {
        apply_mutation(
            db,
            Event::new(resolved_id.clone(), Action::Unlabeled)
                .with_values(None, Some(label.to_string())),
        )?;

        println!("Removed label {} from {}", label, resolved_id);
    } else {
        println!("Label {} not found on {}", label, resolved_id);
    }

    Ok(())
}

#[cfg(test)]
#[path = "label_tests.rs"]
mod tests;
