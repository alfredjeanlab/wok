// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Merge operations into database state with HLC conflict resolution.
//!
//! Merge rules:
//! - CreateIssue: first write wins (duplicate creates are ignored)
//! - SetStatus/SetTitle/SetType: last HLC wins
//! - AddLabel/RemoveLabel: add always succeeds, remove always succeeds
//! - AddNote: always append
//! - AddDep/RemoveDep: add always succeeds, remove always succeeds
//!
//! All merge operations are idempotent and commutative.

use crate::db::Database;
use crate::error::Result;
use crate::hlc::Hlc;
use crate::issue::{Action, Event, Issue, Status};
use crate::op::{Op, OpPayload};

/// Trait for applying operations with HLC-based conflict resolution.
pub trait Merge {
    /// Applies an operation to the database.
    ///
    /// Returns Ok(true) if the operation was applied, Ok(false) if it was
    /// a no-op (e.g., duplicate create, stale update).
    fn apply(&mut self, op: &Op) -> Result<bool>;

    /// Applies multiple operations in order.
    ///
    /// Returns the number of operations that were actually applied.
    fn apply_all(&mut self, ops: &[Op]) -> Result<usize> {
        let mut applied = 0;
        for op in ops {
            if self.apply(op)? {
                applied += 1;
            }
        }
        Ok(applied)
    }
}

impl Merge for Database {
    fn apply(&mut self, op: &Op) -> Result<bool> {
        match &op.payload {
            OpPayload::CreateIssue {
                id,
                issue_type,
                title,
            } => {
                // First write wins
                if self.issue_exists(id)? {
                    return Ok(false);
                }

                let issue = Issue {
                    id: id.clone(),
                    issue_type: *issue_type,
                    title: title.clone(),
                    description: None,
                    status: Status::Todo,
                    assignee: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    last_status_hlc: None,
                    last_title_hlc: None,
                    last_type_hlc: None,
                    last_description_hlc: None,
                    last_assignee_hlc: None,
                };
                self.create_issue(&issue)?;

                let event = Event::new(id.clone(), Action::Created);
                self.log_event(&event)?;

                Ok(true)
            }

            OpPayload::SetStatus {
                issue_id,
                status,
                reason,
            } => self.apply_set_status(issue_id, *status, reason.clone(), op.id),

            OpPayload::SetTitle { issue_id, title } => self.apply_set_title(issue_id, title, op.id),

            OpPayload::SetType {
                issue_id,
                issue_type,
            } => self.apply_set_type(issue_id, *issue_type, op.id),

            OpPayload::AddLabel { issue_id, label } => {
                // Add always succeeds (idempotent)
                if !self.issue_exists(issue_id)? {
                    return Ok(false);
                }
                // add_label is already idempotent (unique constraint)
                let _ = self.add_label(issue_id, label);

                let event = Event::new(issue_id.clone(), Action::Labeled)
                    .with_values(None, Some(label.clone()));
                self.log_event(&event)?;

                Ok(true)
            }

            OpPayload::RemoveLabel { issue_id, label } => {
                // Remove always succeeds (idempotent)
                if !self.issue_exists(issue_id)? {
                    return Ok(false);
                }
                let removed = self.remove_label(issue_id, label)?;
                if removed {
                    let event = Event::new(issue_id.clone(), Action::Unlabeled)
                        .with_values(Some(label.clone()), None);
                    self.log_event(&event)?;
                }
                Ok(true)
            }

            OpPayload::AddNote {
                issue_id,
                content,
                status,
            } => {
                // Always append
                if !self.issue_exists(issue_id)? {
                    return Ok(false);
                }
                self.add_note(issue_id, *status, content)?;

                let event = Event::new(issue_id.clone(), Action::Noted);
                self.log_event(&event)?;

                Ok(true)
            }

            OpPayload::AddDep {
                from_id,
                to_id,
                relation,
            } => {
                // Add always succeeds (idempotent)
                if !self.issue_exists(from_id)? || !self.issue_exists(to_id)? {
                    return Ok(false);
                }
                // May fail if already exists or would create cycle
                match self.add_dependency(from_id, to_id, *relation) {
                    Ok(()) => {
                        let event = Event::new(from_id.clone(), Action::Related)
                            .with_values(None, Some(format!("{relation} {to_id}")));
                        self.log_event(&event)?;
                        Ok(true)
                    }
                    Err(_) => Ok(false), // Already exists or would create cycle
                }
            }

            OpPayload::RemoveDep {
                from_id,
                to_id,
                relation,
            } => {
                // Remove always succeeds (idempotent)
                if !self.issue_exists(from_id)? {
                    return Ok(false);
                }
                match self.remove_dependency(from_id, to_id, *relation) {
                    Ok(()) => {
                        let event = Event::new(from_id.clone(), Action::Unrelated)
                            .with_values(Some(format!("{relation} {to_id}")), None);
                        self.log_event(&event)?;
                        Ok(true)
                    }
                    Err(_) => Ok(false), // Doesn't exist
                }
            }

            OpPayload::ConfigRename {
                old_prefix,
                new_prefix,
            } => self.apply_config_rename(old_prefix, new_prefix),
        }
    }
}

impl Database {
    fn apply_set_status(
        &mut self,
        issue_id: &str,
        status: Status,
        reason: Option<String>,
        hlc: Hlc,
    ) -> Result<bool> {
        let issue = match self.get_issue(issue_id) {
            Ok(i) => i,
            Err(_) => return Ok(false),
        };

        // Last HLC wins
        if let Some(last_hlc) = issue.last_status_hlc {
            if hlc <= last_hlc {
                return Ok(false);
            }
        }

        let old_status = issue.status;
        self.update_issue_status(issue_id, status)?;
        self.update_issue_status_hlc(issue_id, hlc)?;

        let action = match status {
            Status::InProgress if old_status.is_terminal() => Action::Reopened,
            Status::InProgress => Action::Started,
            Status::Todo => Action::Stopped,
            Status::Done => Action::Done,
            Status::Closed => Action::Closed,
        };

        let event = Event::new(issue_id.to_string(), action)
            .with_values(Some(old_status.to_string()), Some(status.to_string()))
            .with_reason(reason);
        self.log_event(&event)?;

        Ok(true)
    }

    fn apply_set_title(&mut self, issue_id: &str, title: &str, hlc: Hlc) -> Result<bool> {
        let issue = match self.get_issue(issue_id) {
            Ok(i) => i,
            Err(_) => return Ok(false),
        };

        // Last HLC wins
        if let Some(last_hlc) = issue.last_title_hlc {
            if hlc <= last_hlc {
                return Ok(false);
            }
        }

        let old_title = issue.title;
        self.update_issue_title(issue_id, title)?;
        self.update_issue_title_hlc(issue_id, hlc)?;

        let event = Event::new(issue_id.to_string(), Action::Edited)
            .with_values(Some(old_title), Some(title.to_string()));
        self.log_event(&event)?;

        Ok(true)
    }

    fn apply_set_type(
        &mut self,
        issue_id: &str,
        issue_type: crate::issue::IssueType,
        hlc: Hlc,
    ) -> Result<bool> {
        let issue = match self.get_issue(issue_id) {
            Ok(i) => i,
            Err(_) => return Ok(false),
        };

        // Last HLC wins
        if let Some(last_hlc) = issue.last_type_hlc {
            if hlc <= last_hlc {
                return Ok(false);
            }
        }

        let old_type = issue.issue_type;
        self.update_issue_type(issue_id, issue_type)?;
        self.update_issue_type_hlc(issue_id, hlc)?;

        let event = Event::new(issue_id.to_string(), Action::Edited)
            .with_values(Some(old_type.to_string()), Some(issue_type.to_string()));
        self.log_event(&event)?;

        Ok(true)
    }

    /// Apply a config rename operation to update all issue IDs with the old prefix.
    ///
    /// This is idempotent: applying the same rename twice has no additional effect.
    fn apply_config_rename(&self, old_prefix: &str, new_prefix: &str) -> Result<bool> {
        let old_pattern = format!("{}-", old_prefix);
        let new_pattern = format!("{}-", new_prefix);
        let like_pattern = format!("{}%", old_pattern);

        // Disable foreign keys for batch update
        self.conn.execute("PRAGMA foreign_keys = OFF", [])?;

        let result = (|| -> Result<()> {
            // Update issues table (primary)
            self.conn.execute(
                "UPDATE issues SET id = replace(id, ?1, ?2) WHERE id LIKE ?3",
                [&old_pattern, &new_pattern, &like_pattern],
            )?;

            // Update deps table (both columns)
            self.conn.execute(
                "UPDATE deps SET from_id = replace(from_id, ?1, ?2) WHERE from_id LIKE ?3",
                [&old_pattern, &new_pattern, &like_pattern],
            )?;
            self.conn.execute(
                "UPDATE deps SET to_id = replace(to_id, ?1, ?2) WHERE to_id LIKE ?3",
                [&old_pattern, &new_pattern, &like_pattern],
            )?;

            // Update labels, notes, events tables
            self.conn.execute(
                "UPDATE labels SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
                [&old_pattern, &new_pattern, &like_pattern],
            )?;
            self.conn.execute(
                "UPDATE notes SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
                [&old_pattern, &new_pattern, &like_pattern],
            )?;
            self.conn.execute(
                "UPDATE events SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
                [&old_pattern, &new_pattern, &like_pattern],
            )?;

            Ok(())
        })();

        // Re-enable foreign keys regardless of success/failure
        self.conn.execute("PRAGMA foreign_keys = ON", [])?;

        result?;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "merge_tests.rs"]
mod tests;
