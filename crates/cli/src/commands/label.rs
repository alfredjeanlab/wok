// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;

use super::apply_mutation;
use crate::error::Result;
use crate::models::{Action, Event};
use crate::validate::{validate_label, validate_label_count};

/// Add multiple labels to multiple issues. DB is already open.
pub fn add_with_db(db: &Database, ids: &[String], labels: &[String]) -> Result<()> {
    // Validate all labels first
    for label in labels {
        validate_label(label)?;
    }

    // Add each label to each issue
    for label in labels {
        add_impl_resolved(db, ids, label)?;
    }
    Ok(())
}

/// Internal implementation that accepts db for testing.
#[cfg(test)]
pub(crate) fn add_impl(db: &Database, ids: &[String], label: &str) -> Result<()> {
    // Validate label once (applies to all)
    validate_label(label)?;

    for id in ids {
        add_single(db, id, label)?;
    }
    Ok(())
}

#[cfg(test)]
fn add_single(db: &Database, id: &str, label: &str) -> Result<()> {
    // Resolve potentially partial ID
    let resolved_id = db.resolve_id(id)?;
    add_single_resolved(db, &resolved_id, label)
}

/// Internal implementation for pre-resolved IDs (no re-resolution needed).
fn add_impl_resolved(db: &Database, resolved_ids: &[String], label: &str) -> Result<()> {
    for resolved_id in resolved_ids {
        add_single_resolved(db, resolved_id, label)?;
    }
    Ok(())
}

fn add_single_resolved(db: &Database, resolved_id: &str, label: &str) -> Result<()> {
    // Verify issue exists
    db.get_issue(resolved_id)?;

    // Validate label count
    let current_labels = db.get_labels(resolved_id)?;
    validate_label_count(current_labels.len())?;

    db.add_label(resolved_id, label)?;

    apply_mutation(
        db,
        Event::new(resolved_id.to_string(), Action::Labeled)
            .with_values(None, Some(label.to_string())),
    )?;

    println!("Labeled {} with {}", resolved_id, label);

    Ok(())
}

/// Remove multiple labels from multiple issues. DB is already open.
pub fn remove_with_db(db: &Database, ids: &[String], labels: &[String]) -> Result<()> {
    // Remove each label from each issue
    for label in labels {
        remove_impl_resolved(db, ids, label)?;
    }
    Ok(())
}

/// Internal implementation that accepts db for testing.
#[cfg(test)]
pub(crate) fn remove_impl(db: &Database, ids: &[String], label: &str) -> Result<()> {
    for id in ids {
        remove_single(db, id, label)?;
    }
    Ok(())
}

#[cfg(test)]
fn remove_single(db: &Database, id: &str, label: &str) -> Result<()> {
    // Resolve potentially partial ID
    let resolved_id = db.resolve_id(id)?;
    remove_single_resolved(db, &resolved_id, label)
}

/// Internal implementation for pre-resolved IDs (no re-resolution needed).
fn remove_impl_resolved(db: &Database, resolved_ids: &[String], label: &str) -> Result<()> {
    for resolved_id in resolved_ids {
        remove_single_resolved(db, resolved_id, label)?;
    }
    Ok(())
}

fn remove_single_resolved(db: &Database, resolved_id: &str, label: &str) -> Result<()> {
    // Verify issue exists
    db.get_issue(resolved_id)?;

    let removed = db.remove_label(resolved_id, label)?;

    if removed {
        apply_mutation(
            db,
            Event::new(resolved_id.to_string(), Action::Unlabeled)
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
