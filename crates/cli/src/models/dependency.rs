// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::str::FromStr;

use crate::error::{Error, Result};

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

#[cfg(test)]
#[path = "dependency_tests.rs"]
mod tests;
