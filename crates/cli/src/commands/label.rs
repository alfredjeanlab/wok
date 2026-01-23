// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use wk_core::OpPayload;

use crate::config::Config;
use crate::db::Database;

use super::{apply_mutation, open_db};
use crate::error::Result;
use crate::models::{Action, Event};
use crate::validate::{validate_label, validate_label_count};

pub fn add(ids: &[String], label: &str) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    add_impl(&db, &config, &work_dir, ids, label)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn add_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    label: &str,
) -> Result<()> {
    // Validate label once (applies to all)
    validate_label(label)?;

    for id in ids {
        add_single(db, config, work_dir, id, label)?;
    }
    Ok(())
}

fn add_single(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    label: &str,
) -> Result<()> {
    // Verify issue exists
    db.get_issue(id)?;

    // Validate label count
    let current_labels = db.get_labels(id)?;
    validate_label_count(current_labels.len())?;

    db.add_label(id, label)?;

    apply_mutation(
        db,
        work_dir,
        config,
        Event::new(id.to_string(), Action::Labeled).with_values(None, Some(label.to_string())),
        Some(OpPayload::add_label(id.to_string(), label.to_string())),
    )?;

    println!("Labeled {} with {}", id, label);

    Ok(())
}

pub fn remove(ids: &[String], label: &str) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    remove_impl(&db, &config, &work_dir, ids, label)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn remove_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    label: &str,
) -> Result<()> {
    for id in ids {
        remove_single(db, config, work_dir, id, label)?;
    }
    Ok(())
}

fn remove_single(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    label: &str,
) -> Result<()> {
    // Verify issue exists
    db.get_issue(id)?;

    let removed = db.remove_label(id, label)?;

    if removed {
        apply_mutation(
            db,
            work_dir,
            config,
            Event::new(id.to_string(), Action::Unlabeled)
                .with_values(None, Some(label.to_string())),
            Some(OpPayload::remove_label(id.to_string(), label.to_string())),
        )?;

        println!("Removed label {} from {}", label, id);
    } else {
        println!("Label {} not found on {}", label, id);
    }

    Ok(())
}

#[cfg(test)]
#[path = "label_tests.rs"]
mod tests;
