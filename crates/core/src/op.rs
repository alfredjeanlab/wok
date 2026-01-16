// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Operations for distributed mutation tracking.
//!
//! All mutations in wk are represented as operations (ops). Each op has a
//! unique ID based on HLC timestamp and contains a payload describing the
//! mutation. Ops are designed to be:
//!
//! - Serializable: Can be stored and transmitted
//! - Idempotent: Applying twice has same effect as applying once
//! - Commutative: Order of application doesn't matter (with merge rules)

use serde::{Deserialize, Serialize};

use crate::hlc::Hlc;
use crate::issue::{IssueType, Relation, Status};

/// Unique identifier for an operation.
///
/// OpId is essentially an HLC wrapped for clarity.
pub type OpId = Hlc;

/// An operation representing a mutation to the issue database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Op {
    /// Unique identifier for this operation (HLC timestamp).
    pub id: OpId,
    /// The actual mutation being performed.
    pub payload: OpPayload,
}

impl Op {
    /// Creates a new operation with the given ID and payload.
    pub fn new(id: OpId, payload: OpPayload) -> Self {
        Op { id, payload }
    }

    /// Returns the issue ID affected by this operation.
    ///
    /// Returns empty string for operations that are not issue-specific
    /// (such as ConfigRename).
    pub fn issue_id(&self) -> &str {
        match &self.payload {
            OpPayload::CreateIssue { id, .. } => id,
            OpPayload::SetStatus { issue_id, .. } => issue_id,
            OpPayload::SetTitle { issue_id, .. } => issue_id,
            OpPayload::SetType { issue_id, .. } => issue_id,
            OpPayload::AddLabel { issue_id, .. } => issue_id,
            OpPayload::RemoveLabel { issue_id, .. } => issue_id,
            OpPayload::AddNote { issue_id, .. } => issue_id,
            OpPayload::AddDep { from_id, .. } => from_id,
            OpPayload::RemoveDep { from_id, .. } => from_id,
            OpPayload::ConfigRename { .. } => "",
        }
    }
}

impl PartialOrd for Op {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Op {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/// Payload describing the specific mutation being performed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpPayload {
    /// Create a new issue.
    CreateIssue {
        id: String,
        issue_type: IssueType,
        title: String,
    },

    /// Set the status of an issue.
    SetStatus {
        issue_id: String,
        status: Status,
        /// Optional reason for the status change (e.g., close reason).
        reason: Option<String>,
    },

    /// Set the title of an issue.
    SetTitle { issue_id: String, title: String },

    /// Set the type of an issue.
    SetType {
        issue_id: String,
        issue_type: IssueType,
    },

    /// Add a label to an issue.
    AddLabel { issue_id: String, label: String },

    /// Remove a label from an issue.
    RemoveLabel { issue_id: String, label: String },

    /// Add a note to an issue.
    AddNote {
        issue_id: String,
        content: String,
        /// The status when the note was added.
        status: Status,
    },

    /// Add a dependency between issues.
    AddDep {
        from_id: String,
        to_id: String,
        relation: Relation,
    },

    /// Remove a dependency between issues.
    RemoveDep {
        from_id: String,
        to_id: String,
        relation: Relation,
    },

    /// Rename the issue ID prefix across all issues.
    ConfigRename {
        old_prefix: String,
        new_prefix: String,
    },
}

impl OpPayload {
    /// Creates a CreateIssue payload.
    pub fn create_issue(id: String, issue_type: IssueType, title: String) -> Self {
        OpPayload::CreateIssue {
            id,
            issue_type,
            title,
        }
    }

    /// Creates a SetStatus payload.
    pub fn set_status(issue_id: String, status: Status, reason: Option<String>) -> Self {
        OpPayload::SetStatus {
            issue_id,
            status,
            reason,
        }
    }

    /// Creates a SetTitle payload.
    pub fn set_title(issue_id: String, title: String) -> Self {
        OpPayload::SetTitle { issue_id, title }
    }

    /// Creates a SetType payload.
    pub fn set_type(issue_id: String, issue_type: IssueType) -> Self {
        OpPayload::SetType {
            issue_id,
            issue_type,
        }
    }

    /// Creates an AddLabel payload.
    pub fn add_label(issue_id: String, label: String) -> Self {
        OpPayload::AddLabel { issue_id, label }
    }

    /// Creates a RemoveLabel payload.
    pub fn remove_label(issue_id: String, label: String) -> Self {
        OpPayload::RemoveLabel { issue_id, label }
    }

    /// Creates an AddNote payload.
    pub fn add_note(issue_id: String, content: String, status: Status) -> Self {
        OpPayload::AddNote {
            issue_id,
            content,
            status,
        }
    }

    /// Creates an AddDep payload.
    pub fn add_dep(from_id: String, to_id: String, relation: Relation) -> Self {
        OpPayload::AddDep {
            from_id,
            to_id,
            relation,
        }
    }

    /// Creates a RemoveDep payload.
    pub fn remove_dep(from_id: String, to_id: String, relation: Relation) -> Self {
        OpPayload::RemoveDep {
            from_id,
            to_id,
            relation,
        }
    }

    /// Creates a ConfigRename payload.
    pub fn config_rename(old_prefix: String, new_prefix: String) -> Self {
        OpPayload::ConfigRename {
            old_prefix,
            new_prefix,
        }
    }
}

#[cfg(test)]
#[path = "op_tests.rs"]
mod tests;
