// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Internal relationship types stored in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    /// A blocks B = B should wait for A
    Blocks,
    /// A is tracked by B (B contains A)
    TrackedBy,
    /// A tracks B (A contains B)
    Tracks,
}

impl Relation {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            Relation::Blocks => "blocks",
            Relation::TrackedBy => "tracked-by",
            Relation::Tracks => "tracks",
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Relation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "blocks" => Ok(Relation::Blocks),
            "tracked-by" => Ok(Relation::TrackedBy),
            "tracks" => Ok(Relation::Tracks),
            _ => Err(Error::InvalidRelation(s.to_string())),
        }
    }
}

/// User-facing relationship types for CLI commands.
///
/// This provides a simplified view of relationships compared to [`Relation`].
/// `tracks` is translated to tracks/tracked-by relationships internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRelation {
    /// A blocks B (B cannot proceed until A is done).
    Blocks,
    /// A is blocked by B (A cannot proceed until B is done).
    /// Equivalent to "B blocks A"
    BlockedBy,
    /// A tracks B (A is the parent/epic containing B).
    Tracks,
    /// A is tracked by B (B is the parent/epic containing A).
    /// Equivalent to "B tracks A"
    TrackedBy,
}

impl FromStr for UserRelation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "blocks" => Ok(UserRelation::Blocks),
            "blocked-by" | "blocked_by" | "blockedby" => Ok(UserRelation::BlockedBy),
            "tracks" | "contains" => Ok(UserRelation::Tracks),
            "tracked-by" | "tracked_by" | "trackedby" => Ok(UserRelation::TrackedBy),
            _ => Err(Error::InvalidRelation(s.to_string())),
        }
    }
}

/// A relationship between two issues.
///
/// Dependencies are directional: `from_id` has the relationship to `to_id`.
/// For example, if issue A blocks issue B, the dependency is stored as
/// `from_id: "A", to_id: "B", relation: Blocks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// The source issue of the relationship.
    pub from_id: String,
    /// The target issue of the relationship.
    pub to_id: String,
    /// The type of relationship.
    pub relation: Relation,
    /// When the dependency was created.
    pub created_at: DateTime<Utc>,
}

impl Dependency {
    /// Test helper: construct a Dependency with current timestamp.
    /// Production code constructs from DB rows which include stored timestamps.
    #[cfg(test)]
    pub fn new(from_id: String, to_id: String, relation: Relation) -> Self {
        Dependency {
            from_id,
            to_id,
            relation,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
#[path = "dependency_tests.rs"]
mod tests;
