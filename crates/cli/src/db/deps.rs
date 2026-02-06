// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::error::Result;
use crate::models::{Dependency, Relation};

// Re-exported for test use via `use super::*`
#[cfg(test)]
use crate::error::Error;

use super::Database;

impl Database {
    /// Add a dependency between two issues
    pub fn add_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        Ok(self.0.add_dependency(from_id, to_id, relation.into())?)
    }

    /// Remove a dependency between two issues
    pub fn remove_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        Ok(self.0.remove_dependency(from_id, to_id, relation.into())?)
    }

    /// Get all dependencies from an issue
    pub fn get_deps_from(&self, from_id: &str) -> Result<Vec<Dependency>> {
        let deps = self.0.get_deps_from(from_id)?;
        Ok(deps.into_iter().map(|d| d.into()).collect())
    }

    /// Get issues that directly block the given issue
    pub fn get_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.0.get_blockers(issue_id)?)
    }

    /// Get all issues that transitively block the given issue (active blockers only)
    pub fn get_transitive_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.0.get_transitive_blockers(issue_id)?)
    }

    /// Get issues that this issue blocks
    pub fn get_blocking(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.0.get_blocking(issue_id)?)
    }

    /// Get tracking issues (issues this is tracked by)
    pub fn get_tracking(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.0.get_tracking(issue_id)?)
    }

    /// Get tracked issues (issues this tracks)
    pub fn get_tracked(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.0.get_tracked(issue_id)?)
    }
}

#[cfg(test)]
#[path = "deps_tests.rs"]
mod tests;
