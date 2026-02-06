// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::Utc;
use rusqlite::params;

use crate::error::{Error, Result};
use crate::models::{Dependency, Relation};

use super::{parse_db, parse_timestamp, Database};

/// Map a row to a Dependency.
///
/// Expected columns: from_id, to_id, rel, created_at
fn row_to_dependency(row: &rusqlite::Row) -> rusqlite::Result<Dependency> {
    let rel_str: String = row.get(2)?;
    let created_str: String = row.get(3)?;
    Ok(Dependency {
        from_id: row.get(0)?,
        to_id: row.get(1)?,
        relation: parse_db(&rel_str, "rel")?,
        created_at: parse_timestamp(&created_str, "created_at")?,
    })
}

impl Database {
    /// Add a dependency between two issues
    pub fn add_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        if from_id == to_id {
            return Err(Error::SelfDependency);
        }

        // Check if adding this would create a cycle (only for blocks)
        if relation == Relation::Blocks && self.would_create_cycle(from_id, to_id)? {
            return Err(Error::CycleDetected);
        }

        self.conn.execute(
            "INSERT OR IGNORE INTO deps (from_id, to_id, rel, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![from_id, to_id, relation.as_str(), Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// Remove a dependency between two issues
    pub fn remove_dependency(&self, from_id: &str, to_id: &str, relation: Relation) -> Result<()> {
        let affected = self.conn.execute(
            "DELETE FROM deps WHERE from_id = ?1 AND to_id = ?2 AND rel = ?3",
            params![from_id, to_id, relation.as_str()],
        )?;

        if affected == 0 {
            return Err(Error::DependencyNotFound {
                from: from_id.to_string(),
                rel: relation.to_string(),
                to: to_id.to_string(),
            });
        }

        Ok(())
    }

    /// Check if adding from_id -> to_id would create a cycle
    /// Uses recursive CTE to find if to_id can reach from_id
    fn would_create_cycle(&self, from_id: &str, to_id: &str) -> Result<bool> {
        // Check if to_id can reach from_id through existing blocks relationships
        // If so, adding from_id -> to_id would create a cycle
        let count: i64 = self.conn.query_row(
            "WITH RECURSIVE chain(id) AS (
                SELECT from_id FROM deps WHERE to_id = ?1 AND rel = 'blocks'
                UNION
                SELECT d.from_id FROM deps d JOIN chain c ON d.to_id = c.id WHERE d.rel = 'blocks'
            )
            SELECT COUNT(*) FROM chain WHERE id = ?2",
            params![from_id, to_id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Get all dependencies from an issue
    pub fn get_deps_from(&self, from_id: &str) -> Result<Vec<Dependency>> {
        let mut stmt = self
            .conn
            .prepare("SELECT from_id, to_id, rel, created_at FROM deps WHERE from_id = ?1")?;

        let deps = stmt
            .query_map(params![from_id], row_to_dependency)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(deps)
    }

    /// Get issues that directly block the given issue (by blocks relation)
    pub fn get_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT from_id FROM deps WHERE to_id = ?1 AND rel = 'blocks'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get all issues that transitively block the given issue (active blockers only)
    pub fn get_transitive_blockers(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "WITH RECURSIVE blockers AS (
                SELECT d.from_id as blocker_id
                FROM deps d JOIN issues i ON i.id = d.from_id
                WHERE d.to_id = ?1 AND d.rel = 'blocks'
                  AND i.status NOT IN ('done', 'closed')
                UNION
                SELECT d.from_id
                FROM deps d JOIN issues i ON i.id = d.from_id
                JOIN blockers b ON d.to_id = b.blocker_id
                WHERE d.rel = 'blocks' AND i.status NOT IN ('done', 'closed')
            )
            SELECT blocker_id FROM blockers",
        )?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get issues that this issue blocks
    pub fn get_blocking(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM deps WHERE from_id = ?1 AND rel = 'blocks'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get tracking issues (issues this is tracked by)
    pub fn get_tracking(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM deps WHERE from_id = ?1 AND rel = 'tracked-by'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    /// Get tracked issues (issues this tracks)
    pub fn get_tracked(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT to_id FROM deps WHERE from_id = ?1 AND rel = 'tracks'")?;

        let ids = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(ids)
    }
}

#[cfg(test)]
#[path = "deps_tests.rs"]
mod tests;
