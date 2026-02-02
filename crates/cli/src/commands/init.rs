// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::{Path, PathBuf};

use crate::completions;
use crate::config::{init_work_dir, init_work_dir_private, write_gitignore};
use crate::db::Database;
use crate::error::{Error, Result};
use crate::id::validate_prefix;

pub fn run(prefix: Option<String>, path: Option<String>, private: bool) -> Result<()> {
    let target_path = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()?,
    };

    let prefix = match prefix {
        Some(p) => p,
        None => derive_prefix_from_path(&target_path)?,
    };

    // Validate the prefix
    if !validate_prefix(&prefix) {
        return Err(Error::InvalidPrefix);
    }

    let work_dir = if private {
        init_work_dir_private(&target_path, &prefix)?
    } else {
        init_work_dir(&target_path, &prefix)?
    };

    // Initialize the database
    if private {
        // Private mode: create database in .wok/
        let db_path = work_dir.join("issues.db");
        Database::open(&db_path)?;
    } else {
        // User-level mode: ensure state directory and database exist
        let state_dir = crate::config::wok_state_dir();
        std::fs::create_dir_all(&state_dir)?;
        let db_path = state_dir.join("issues.db");
        Database::open(&db_path)?;
    }

    // Create .gitignore
    write_gitignore(&work_dir, private)?;

    println!("Initialized issue tracker at {}", work_dir.display());
    println!("Prefix: {}", prefix);
    if private {
        println!("Mode: private (local database)");
    } else {
        println!("Mode: user-level (shared database)");
    }

    // Install shell completions
    if let Err(e) = completions::install_all() {
        eprintln!("Warning: failed to install shell completions: {}", e);
    }

    Ok(())
}

/// Derive a prefix from the directory path.
/// Uses the directory name, converted to lowercase, keeping letters and digits.
fn derive_prefix_from_path(path: &Path) -> Result<String> {
    let dir_name =
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::CannotDerive {
                item: "prefix",
                from: "path".to_string(),
            })?;

    // Convert to lowercase and keep only ASCII alphanumeric characters
    let prefix: String = dir_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    // Must have at least 2 characters and contain at least one letter
    if prefix.len() < 2 || !prefix.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(Error::CannotDerive {
            item: "prefix",
            from: format!(
                "directory name '{}' (need 2+ chars with at least one letter)",
                dir_name
            ),
        });
    }

    Ok(prefix)
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
