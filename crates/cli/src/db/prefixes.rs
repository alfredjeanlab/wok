// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database operations for prefix tracking.

use super::Database;
use crate::error::Result;
use crate::models::PrefixInfo;

impl Database {
    /// Ensure a prefix exists in the prefixes table.
    pub fn ensure_prefix(&self, prefix: &str) -> Result<()> {
        Ok(self.0.ensure_prefix(prefix)?)
    }

    /// Increment the issue count for a prefix.
    pub fn increment_prefix_count(&self, prefix: &str) -> Result<()> {
        Ok(self.0.increment_prefix_count(prefix)?)
    }

    /// Decrement the issue count for a prefix.
    pub fn decrement_prefix_count(&self, prefix: &str) -> Result<()> {
        Ok(self.0.decrement_prefix_count(prefix)?)
    }

    /// List all prefixes with their issue counts.
    pub fn list_prefixes(&self) -> Result<Vec<PrefixInfo>> {
        Ok(self.0.list_prefixes()?)
    }

    /// Rename a prefix in the prefixes table.
    pub fn rename_prefix(&self, old: &str, new: &str) -> Result<()> {
        Ok(self.0.rename_prefix(old, new)?)
    }
}
