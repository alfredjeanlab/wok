// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use thiserror::Error;

/// All possible errors that can occur in the wkrs library.
///
/// Errors provide user-friendly messages with hints for common issues.
#[derive(Debug, Error)]
pub enum Error {
    #[error("not initialized: run 'wok init' first")]
    NotInitialized,

    #[error("already initialized at {0}")]
    AlreadyInitialized(String),

    #[error("issue not found: {0}")]
    IssueNotFound(String),

    #[error("ambiguous issue ID '{prefix}' matches: {}", matches.join(", "))]
    AmbiguousId {
        prefix: String,
        matches: Vec<String>,
    },

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

    #[error("invalid issue type: '{0}'\n  hint: valid types are: feature, task, bug, chore")]
    InvalidIssueType(String),

    #[error("invalid status: '{0}'\n  hint: valid statuses are: todo, in_progress, done, closed")]
    InvalidStatus(String),

    #[error("invalid relation: '{0}'\n  hint: valid relations are: blocks, blocked-by, tracks, tracked-by")]
    InvalidRelation(String),

    #[error("invalid link type: '{0}'\n  hint: valid types are: github, jira, gitlab, confluence")]
    InvalidLinkType(String),

    #[error("invalid link relation: '{0}'\n  hint: valid relations are: import, blocks, tracks, tracked-by")]
    InvalidLinkRel(String),

    #[error("invalid prefix: must be 2+ lowercase alphanumeric with at least one letter")]
    InvalidPrefix,

    #[error("workspace not found: {0}\n  hint: the workspace directory must exist before creating a link")]
    WorkspaceNotFound(String),

    // Phase 1: Filter Parser Errors
    #[error("empty filter expression")]
    FilterEmpty,

    #[error("unknown filter field: '{field}'")]
    FilterUnknownField { field: String },

    #[error("invalid filter operator '{op}' for field '{field}'")]
    FilterInvalidOperator { field: String, op: String },

    #[error("invalid filter value for {field}: {reason}")]
    FilterInvalidValue { field: String, reason: String },

    #[error("invalid duration: {reason}")]
    InvalidDuration { reason: String },

    // Phase 2: Command Validation Errors
    #[error("operation cancelled")]
    Cancelled,

    #[error("{context} is required for {operation}")]
    RequiredFor {
        context: &'static str,
        operation: &'static str,
    },

    #[error("cannot derive {item} from {from}")]
    CannotDerive { item: &'static str, from: String },

    #[error("line {line}: {reason}")]
    ParseLineError { line: usize, reason: String },

    #[error("invalid scope '{scope}'")]
    InvalidScope { scope: String },

    #[error("interactive mode requires a terminal (TTY)")]
    TtyRequired,

    #[error("permission denied writing to {target}")]
    PermissionDenied { target: String },

    #[error("no input file specified")]
    NoInputFile,

    #[error("invalid timestamp: {reason}")]
    InvalidTimestamp { reason: String },

    // Phase 3: Link and Edit Errors
    #[error("{requirement} requires {dependency}")]
    LinkRequires {
        requirement: &'static str,
        dependency: &'static str,
    },

    #[error("unknown attribute '{attr}'")]
    UnknownAttribute { attr: String },

    // Phase 4: Note and Lookup Errors
    #[error("no notes to replace for issue {issue_id}")]
    NoNotesToReplace { issue_id: String },

    #[error("{field} is required")]
    FieldRequired { field: &'static str },

    #[error("cannot add notes to closed issues")]
    CannotNoteClosedIssue,

    #[error("unknown format '{format}'")]
    UnknownFormat { format: String },

    #[error("cannot create issue: {reason}")]
    CannotCreateIssue { reason: String },

    #[error("failed to generate unique issue ID after multiple retries")]
    IdGenerationFailed,

    #[deprecated(note = "use specific error variants instead")]
    #[error("{0}")]
    InvalidInput(String),

    #[error("{field} too long ({actual} chars, max {max})")]
    FieldTooLong {
        field: &'static str,
        actual: usize,
        max: usize,
    },

    #[error("{field} cannot be empty")]
    FieldEmpty { field: &'static str },

    #[error("too many labels (max {max} per issue)")]
    LabelLimitExceeded { max: usize },

    #[error("export path cannot be empty")]
    ExportPathEmpty,

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("corrupted data in database: {0}")]
    CorruptedData(String),

    #[error("daemon error: {0}")]
    Daemon(String),

    #[error("daemon version mismatch: daemon is v{daemon_version}, CLI is v{cli_version}")]
    DaemonVersionMismatch {
        daemon_version: String,
        cli_version: String,
    },

    #[error("daemon timeout: {0}")]
    DaemonTimeout(String),

    #[error("some operations failed: {succeeded} succeeded, {failed} failed")]
    PartialBulkFailure {
        succeeded: usize,
        failed: usize,
        unknown_ids: Vec<String>,
        transition_failures: Vec<(String, String)>,
    },
}

/// A specialized Result type for wkrs operations.
pub type Result<T> = std::result::Result<T, Error>;

// NOTE(compat): maps core errors that hit the deprecated InvalidInput variant
#[allow(deprecated)]
impl From<wk_core::Error> for Error {
    fn from(e: wk_core::Error) -> Self {
        match e {
            wk_core::Error::IssueNotFound(id) => Error::IssueNotFound(id),
            wk_core::Error::AmbiguousId { prefix, matches } => {
                Error::AmbiguousId { prefix, matches }
            }
            wk_core::Error::InvalidTransition {
                from,
                to,
                valid_targets,
            } => Error::InvalidTransition {
                from,
                to,
                valid_targets,
            },
            wk_core::Error::CycleDetected => Error::CycleDetected,
            wk_core::Error::SelfDependency => Error::SelfDependency,
            wk_core::Error::DependencyNotFound { from, rel, to } => {
                Error::DependencyNotFound { from, rel, to }
            }
            wk_core::Error::InvalidIssueType(s) => Error::InvalidIssueType(s),
            wk_core::Error::InvalidStatus(s) => Error::InvalidStatus(s),
            wk_core::Error::InvalidRelation(s) => Error::InvalidRelation(s),
            wk_core::Error::InvalidAction(s) => {
                Error::InvalidInput(format!("invalid action: {}", s))
            }
            wk_core::Error::InvalidLinkType(s) => Error::InvalidLinkType(s),
            wk_core::Error::InvalidLinkRel(s) => Error::InvalidLinkRel(s),
            wk_core::Error::InvalidInput(s) => Error::InvalidInput(s),
            wk_core::Error::Database(e) => Error::Database(e),
            wk_core::Error::Io(e) => Error::Io(e),
            wk_core::Error::Json(e) => Error::Json(e),
            wk_core::Error::CorruptedData(s) => Error::CorruptedData(s),
            wk_core::Error::DuplicateOp(s) => Error::InvalidInput(format!("duplicate op: {}", s)),
            wk_core::Error::InvalidHlc(s) => Error::InvalidInput(format!("invalid HLC: {}", s)),
            wk_core::Error::Oplog(s) => Error::Daemon(format!("oplog error: {}", s)),
        }
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
