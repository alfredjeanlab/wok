// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Error types for wk-core operations.

use thiserror::Error;

/// All possible errors that can occur in wk-core operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("issue not found: {0}")]
    IssueNotFound(String),

    #[error("invalid status transition: cannot go from {from} to {to}\n  hint: from '{from}' you can go to: {valid_targets}")]
    InvalidTransition {
        from: String,
        to: String,
        valid_targets: String,
    },

    #[error(
        "would create a dependency cycle\n  hint: this would create a circular dependency chain"
    )]
    CycleDetected,

    #[error("cannot create self-dependency\n  hint: an issue cannot block or track itself")]
    SelfDependency,

    #[error("dependency not found: {from} {rel} {to}")]
    DependencyNotFound {
        from: String,
        rel: String,
        to: String,
    },

    #[error(
        "invalid issue type: '{0}'\n  hint: valid types are: feature, task, bug, chore, idea, epic"
    )]
    InvalidIssueType(String),

    #[error("invalid status: '{0}'\n  hint: valid statuses are: todo, in_progress, done, closed")]
    InvalidStatus(String),

    #[error("invalid relation: '{0}'\n  hint: valid relations are: blocks, tracked-by, tracks")]
    InvalidRelation(String),

    #[error("invalid action: '{0}'")]
    InvalidAction(String),

    #[error("invalid link type: '{0}'\n  hint: valid types are: github, jira, gitlab, confluence")]
    InvalidLinkType(String),

    #[error("invalid link relation: '{0}'\n  hint: valid relations are: import, blocks, tracks, tracked-by")]
    InvalidLinkRel(String),

    #[error("{0}")]
    InvalidInput(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("corrupted data: {0}")]
    CorruptedData(String),

    #[error("duplicate operation: {0}")]
    DuplicateOp(String),

    #[error("invalid HLC: {0}")]
    InvalidHlc(String),

    #[error("oplog error: {0}")]
    Oplog(String),
}

/// A specialized Result type for wk-core operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
