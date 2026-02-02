// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use crate::cli::{ConfigCommand, OutputFormat};
use crate::config::{find_work_dir, get_db_path, Config};
use crate::db::Database;
use crate::error::{Error, Result};
use crate::id::validate_prefix;

use super::open_db;

/// Execute a config subcommand.
pub fn run(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Rename {
            old_prefix,
            new_prefix,
        } => {
            let (db, config, _) = open_db()?;
            let work_dir = find_work_dir()?;
            run_rename_prefix(&db, &config, &work_dir, &old_prefix, &new_prefix)
        }
        ConfigCommand::Prefixes { output } => run_list_prefixes(output),
    }
}

/// List all prefixes in the issue tracker.
fn run_list_prefixes(output: OutputFormat) -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;

    let prefixes = db.list_prefixes()?;

    match output {
        OutputFormat::Text => {
            if prefixes.is_empty() {
                println!("No prefixes found.");
                return Ok(());
            }

            // Show current/default prefix with marker
            for p in &prefixes {
                let marker = if p.prefix == config.prefix {
                    " (default)"
                } else {
                    ""
                };
                let noun = if p.issue_count == 1 {
                    "issue"
                } else {
                    "issues"
                };
                println!("{}: {} {}{}", p.prefix, p.issue_count, noun, marker);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "default": config.prefix,
                "prefixes": prefixes.iter().map(|p| {
                    serde_json::json!({
                        "prefix": p.prefix,
                        "issue_count": p.issue_count,
                        "is_default": p.prefix == config.prefix
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Id => {
            // Just list prefix names
            for p in &prefixes {
                println!("{}", p.prefix);
            }
        }
    }
    Ok(())
}

/// Rename the issue ID prefix across all issues and config.
pub(crate) fn run_rename_prefix(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    old_prefix: &str,
    new_prefix: &str,
) -> Result<()> {
    // 1. Validate old prefix
    if !validate_prefix(old_prefix) {
        return Err(Error::InvalidPrefix);
    }

    // 2. Validate new prefix
    if !validate_prefix(new_prefix) {
        return Err(Error::InvalidPrefix);
    }

    // 3. Check if prefix is unchanged
    if old_prefix == new_prefix {
        println!("Prefix is already '{}'", new_prefix);
        return Ok(());
    }

    // 4. Update all issue IDs in database (within transaction)
    rename_all_issue_ids(db, old_prefix, new_prefix)?;

    // 5. Update config file if old_prefix matches current config prefix
    if config.prefix == old_prefix {
        let mut new_config = config.clone();
        new_config.prefix = new_prefix.to_string();
        new_config.save(work_dir)?;
    }

    println!("Renamed prefix from '{}' to '{}'", old_prefix, new_prefix);
    Ok(())
}

/// Rename all issue IDs in the database from old_prefix to new_prefix.
/// Uses a transaction to ensure atomicity.
fn rename_all_issue_ids(db: &Database, old_prefix: &str, new_prefix: &str) -> Result<()> {
    let old_pattern = format!("{}-", old_prefix);
    let new_pattern = format!("{}-", new_prefix);
    let like_pattern = format!("{}%", old_pattern);

    // Disable foreign keys, perform updates, then re-enable
    // Note: PRAGMA foreign_keys cannot be changed inside a transaction,
    // so we handle this carefully.
    db.conn.execute("PRAGMA foreign_keys = OFF", [])?;

    let result = (|| -> Result<()> {
        let tx = db.conn.unchecked_transaction()?;

        // Update issues table (primary)
        tx.execute(
            "UPDATE issues SET id = replace(id, ?1, ?2) WHERE id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;

        // Update deps table (both columns)
        tx.execute(
            "UPDATE deps SET from_id = replace(from_id, ?1, ?2) WHERE from_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE deps SET to_id = replace(to_id, ?1, ?2) WHERE to_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;

        // Update labels, notes, events, links tables
        tx.execute(
            "UPDATE labels SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE notes SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE events SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE links SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;

        tx.commit()?;
        Ok(())
    })();

    // Re-enable foreign keys regardless of success/failure
    db.conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Update the prefixes table (outside transaction since it doesn't have foreign key constraints)
    if result.is_ok() {
        db.rename_prefix(old_prefix, new_prefix)?;
    }

    result
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
