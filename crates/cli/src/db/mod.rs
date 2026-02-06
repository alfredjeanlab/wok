// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! SQLite-backed database for issue storage.
//!
//! The [`Database`] struct wraps [`wk_core::Database`] and provides all data access
//! operations for the CLI, including CLI-specific extensions like search,
//! batch label queries, and priority extraction.

pub mod deps;
pub mod events;
pub mod issues;
pub mod labels;
pub mod links;
pub mod notes;
pub mod prefixes;

use std::path::Path;

use crate::error::Result;

// Re-export core db utilities for tests and internal use.
pub use wk_core::db::{parse_db, parse_timestamp};

/// SQLite database connection with issue tracker operations.
///
/// Wraps [`wk_core::Database`] and adds CLI-specific query methods.
pub struct Database(pub wk_core::Database);

impl Database {
    /// Open a database connection at the given path, creating and migrating if needed
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Database(wk_core::Database::open(path)?))
    }

    /// Open an in-memory database (for testing and benchmarks)
    pub fn open_in_memory() -> Result<Self> {
        Ok(Database(wk_core::Database::open_in_memory()?))
    }
}

impl std::ops::Deref for Database {
    type Target = wk_core::Database;

    fn deref(&self) -> &wk_core::Database {
        &self.0
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
