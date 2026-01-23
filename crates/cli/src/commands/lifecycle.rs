// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use wk_core::detect::is_human_interactive;
use wk_core::identity::get_user_name;
use wk_core::OpPayload;

use crate::config::Config;
use crate::db::Database;

use super::open_db;
use crate::error::{Error, Result};
use crate::models::{Action, Event, Status};
use crate::validate::validate_and_trim_reason;

use super::queue_op;

/// Result of a bulk lifecycle operation
#[derive(Default)]
pub(crate) struct BulkResult {
    /// Number of issues successfully transitioned
    pub success_count: usize,
    /// IDs that were not found in the database
    pub unknown_ids: Vec<String>,
    /// IDs that failed due to invalid transitions (with error message)
    pub transition_failures: Vec<(String, String)>,
}

impl BulkResult {
    /// Returns true if all operations succeeded
    pub fn is_success(&self) -> bool {
        self.unknown_ids.is_empty() && self.transition_failures.is_empty()
    }

    /// Returns the total number of failures
    pub fn failure_count(&self) -> usize {
        self.unknown_ids.len() + self.transition_failures.len()
    }
}

/// How to handle an error in bulk operations
enum BulkErrorKind {
    /// ID was not found - add to unknown_ids
    NotFound(String),
    /// Invalid transition - add to transition_failures
    /// Contains (from, to, valid_targets, message)
    TransitionFailure {
        from: String,
        to: String,
        valid_targets: String,
        message: String,
    },
    /// InvalidInput that should be treated as transition failure
    InvalidInput(String),
    /// Unexpected error - fail fast
    Fatal(Error),
}

impl BulkErrorKind {
    /// Classify an error for bulk operation handling (consumes the error)
    fn classify(error: Error) -> Self {
        match error {
            Error::IssueNotFound(unknown_id) => BulkErrorKind::NotFound(unknown_id),
            Error::InvalidTransition {
                from,
                to,
                valid_targets,
            } => {
                let message = format!("cannot go from {} to {}", from, to);
                BulkErrorKind::TransitionFailure {
                    from,
                    to,
                    valid_targets,
                    message,
                }
            }
            Error::InvalidInput(msg) if msg.contains("required for agent") => {
                BulkErrorKind::InvalidInput(msg)
            }
            e => BulkErrorKind::Fatal(e),
        }
    }

    /// Reconstruct the Error from this classification
    fn into_error(self) -> Error {
        match self {
            BulkErrorKind::NotFound(id) => Error::IssueNotFound(id),
            BulkErrorKind::TransitionFailure {
                from,
                to,
                valid_targets,
                ..
            } => Error::InvalidTransition {
                from,
                to,
                valid_targets,
            },
            BulkErrorKind::InvalidInput(msg) => Error::InvalidInput(msg),
            BulkErrorKind::Fatal(e) => e,
        }
    }
}

/// Print summary for bulk operations
fn print_bulk_summary(result: &BulkResult, action_verb: &str) {
    // Only print summary if there were multiple items OR failures
    if result.success_count + result.failure_count() <= 1 && result.is_success() {
        return;
    }

    // Summary line
    let total = result.success_count + result.failure_count();
    // Capitalize first letter of action verb
    let capitalized = format!(
        "{}{}",
        action_verb
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default(),
        &action_verb[1..]
    );
    println!(
        "{} {} of {} issues",
        capitalized, result.success_count, total
    );

    // List unknown IDs
    if !result.unknown_ids.is_empty() {
        println!("Unknown IDs: {}", result.unknown_ids.join(", "));
    }

    // List transition failures
    for (id, reason) in &result.transition_failures {
        eprintln!("  {}: {}", id, reason);
    }
}

/// Execute a bulk operation on multiple IDs with consistent error handling.
///
/// - `ids`: The issue IDs to process
/// - `action_verb`: Past tense verb for summary (e.g., "started", "completed")
/// - `operation`: Closure that performs the single-item operation
fn bulk_operation<F>(ids: &[String], action_verb: &str, operation: F) -> Result<()>
where
    F: Fn(&str) -> Result<()>,
{
    let mut result = BulkResult::default();
    let mut last_error: Option<BulkErrorKind> = None;

    for id in ids {
        match operation(id) {
            Ok(()) => result.success_count += 1,
            Err(e) => match BulkErrorKind::classify(e) {
                BulkErrorKind::NotFound(unknown_id) => {
                    result.unknown_ids.push(unknown_id.clone());
                    last_error = Some(BulkErrorKind::NotFound(unknown_id));
                }
                BulkErrorKind::TransitionFailure {
                    from,
                    to,
                    valid_targets,
                    message,
                } => {
                    result
                        .transition_failures
                        .push((id.clone(), message.clone()));
                    last_error = Some(BulkErrorKind::TransitionFailure {
                        from,
                        to,
                        valid_targets,
                        message,
                    });
                }
                BulkErrorKind::InvalidInput(msg) => {
                    result.transition_failures.push((id.clone(), msg.clone()));
                    last_error = Some(BulkErrorKind::InvalidInput(msg));
                }
                BulkErrorKind::Fatal(fatal_error) => {
                    return Err(fatal_error);
                }
            },
        }
    }

    // For single ID, return original error for backward compatibility
    if ids.len() == 1 {
        if result.is_success() {
            return Ok(());
        }
        return Err(last_error
            .map(|k| k.into_error())
            .unwrap_or_else(|| Error::InvalidInput("internal error: expected error".to_string())));
    }

    print_bulk_summary(&result, action_verb);

    if result.is_success() {
        Ok(())
    } else {
        Err(Error::PartialBulkFailure {
            succeeded: result.success_count,
            failed: result.failure_count(),
            unknown_ids: result.unknown_ids,
            transition_failures: result.transition_failures,
        })
    }
}

pub fn start(ids: &[String]) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    start_impl(&db, &config, &work_dir, ids)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn start_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
) -> Result<()> {
    bulk_operation(ids, "started", |id| start_single(db, config, work_dir, id))
}

fn start_single(db: &Database, config: &Config, work_dir: &Path, id: &str) -> Result<()> {
    let issue = db.get_issue(id)?;

    // Start only works from todo status; use reopen for other states
    if issue.status != Status::Todo {
        return Err(Error::InvalidTransition {
            from: issue.status.to_string(),
            to: "in_progress".to_string(),
            valid_targets: "todo (use 'reopen' for other states)".to_string(),
        });
    }

    db.update_issue_status(id, Status::InProgress)?;

    let event = Event::new(id.to_string(), Action::Started).with_values(
        Some(issue.status.to_string()),
        Some("in_progress".to_string()),
    );
    db.log_event(&event)?;

    // Queue SetStatus op for sync
    queue_op(
        work_dir,
        config,
        OpPayload::set_status(id.to_string(), wk_core::Status::InProgress, None),
    )?;

    println!("Started {}", id);

    Ok(())
}

pub fn done(ids: &[String], reason: Option<&str>) -> Result<()> {
    // Validate and trim reason if provided
    let trimmed_reason = if let Some(r) = reason {
        Some(validate_and_trim_reason(r)?)
    } else {
        None
    };

    let (db, config, work_dir) = open_db()?;
    done_impl(&db, &config, &work_dir, ids, trimmed_reason.as_deref())
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn done_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    reason: Option<&str>,
) -> Result<()> {
    bulk_operation(ids, "completed", |id| {
        done_single(db, config, work_dir, id, reason)
    })
}

fn done_single(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let issue = db.get_issue(id)?;

    // Require reason when transitioning from todo (skipping in_progress)
    if issue.status == Status::Todo && reason.is_none() {
        // Try to resolve a reason (auto-generate for humans, error for agents)
        let effective_reason = resolve_reason(None, "complete")?;
        return done_single_with_reason(db, config, work_dir, id, &issue, &effective_reason);
    }

    if !issue.status.can_transition_to(Status::Done) {
        return Err(Error::InvalidTransition {
            from: issue.status.to_string(),
            to: "done".to_string(),
            valid_targets: issue.status.valid_targets(),
        });
    }

    db.update_issue_status(id, Status::Done)?;

    let mut event = Event::new(id.to_string(), Action::Done)
        .with_values(Some(issue.status.to_string()), Some("done".to_string()));

    if let Some(r) = reason {
        event = event.with_reason(Some(r.to_string()));
    }
    db.log_event(&event)?;

    // Add reason as note if provided (will appear in "Summary" section)
    if let Some(r) = reason {
        db.add_note(id, Status::Done, r)?;
    }

    // Log unblocked events for issues that are now unblocked
    log_unblocked_events(db, id)?;

    // Queue SetStatus op for sync
    queue_op(
        work_dir,
        config,
        OpPayload::set_status(
            id.to_string(),
            wk_core::Status::Done,
            reason.map(String::from),
        ),
    )?;

    if let Some(r) = reason {
        println!("Completed {} ({})", id, r);
    } else {
        println!("Completed {}", id);
    }

    Ok(())
}

fn done_single_with_reason(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    issue: &crate::models::Issue,
    reason: &str,
) -> Result<()> {
    if !issue.status.can_transition_to(Status::Done) {
        return Err(Error::InvalidTransition {
            from: issue.status.to_string(),
            to: "done".to_string(),
            valid_targets: issue.status.valid_targets(),
        });
    }

    db.update_issue_status(id, Status::Done)?;

    let event = Event::new(id.to_string(), Action::Done)
        .with_values(Some(issue.status.to_string()), Some("done".to_string()))
        .with_reason(Some(reason.to_string()));
    db.log_event(&event)?;

    // Add reason as note (will appear in "Summary" section)
    db.add_note(id, Status::Done, reason)?;

    // Log unblocked events for issues that are now unblocked
    log_unblocked_events(db, id)?;

    // Queue SetStatus op for sync
    queue_op(
        work_dir,
        config,
        OpPayload::set_status(
            id.to_string(),
            wk_core::Status::Done,
            Some(reason.to_string()),
        ),
    )?;

    println!("Completed {} ({})", id, reason);

    Ok(())
}

pub fn close(ids: &[String], reason: Option<&str>) -> Result<()> {
    let effective_reason = resolve_reason(reason, "closed")?;

    let (db, config, work_dir) = open_db()?;
    close_impl(&db, &config, &work_dir, ids, &effective_reason)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn close_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    reason: &str,
) -> Result<()> {
    bulk_operation(ids, "closed", |id| {
        close_single(db, config, work_dir, id, reason)
    })
}

fn close_single(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    reason: &str,
) -> Result<()> {
    let issue = db.get_issue(id)?;

    if !issue.status.can_transition_to(Status::Closed) {
        return Err(Error::InvalidTransition {
            from: issue.status.to_string(),
            to: "closed".to_string(),
            valid_targets: issue.status.valid_targets(),
        });
    }

    db.update_issue_status(id, Status::Closed)?;

    let event = Event::new(id.to_string(), Action::Closed)
        .with_values(Some(issue.status.to_string()), Some("closed".to_string()))
        .with_reason(Some(reason.to_string()));
    db.log_event(&event)?;

    // Add reason as note (will appear in "Close Reason" section)
    db.add_note(id, Status::Closed, reason)?;

    // Log unblocked events for issues that are now unblocked
    log_unblocked_events(db, id)?;

    // Queue SetStatus op for sync
    queue_op(
        work_dir,
        config,
        OpPayload::set_status(
            id.to_string(),
            wk_core::Status::Closed,
            Some(reason.to_string()),
        ),
    )?;

    println!("Closed {} ({})", id, reason);

    Ok(())
}

pub fn reopen(ids: &[String], reason: Option<&str>) -> Result<()> {
    // Validate and trim reason if provided
    let trimmed_reason = if let Some(r) = reason {
        Some(validate_and_trim_reason(r)?)
    } else {
        None
    };

    let (db, config, work_dir) = open_db()?;
    reopen_impl(&db, &config, &work_dir, ids, trimmed_reason.as_deref())
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn reopen_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    reason: Option<&str>,
) -> Result<()> {
    bulk_operation(ids, "reopened", |id| {
        reopen_single(db, config, work_dir, id, reason)
    })
}

fn reopen_single(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let issue = db.get_issue(id)?;

    // Reason is required when reopening from done/closed, but not from in_progress
    let requires_reason = issue.status == Status::Done || issue.status == Status::Closed;
    if requires_reason && reason.is_none() {
        // Try to resolve a reason (auto-generate for humans, error for agents)
        let effective_reason = resolve_reason(None, "reopened")?;
        return reopen_single_with_reason(db, config, work_dir, id, &issue, &effective_reason);
    }

    if !issue.status.can_transition_to(Status::Todo) {
        return Err(Error::InvalidTransition {
            from: issue.status.to_string(),
            to: "todo".to_string(),
            valid_targets: issue.status.valid_targets(),
        });
    }

    db.update_issue_status(id, Status::Todo)?;

    let mut event = Event::new(id.to_string(), Action::Reopened)
        .with_values(Some(issue.status.to_string()), Some("todo".to_string()));

    if let Some(r) = reason {
        event = event.with_reason(Some(r.to_string()));
        // Add reason as note (will appear in "Description" section)
        db.add_note(id, Status::Todo, r)?;
        println!("Reopened {} ({})", id, r);
    } else {
        println!("Reopened {}", id);
    }

    db.log_event(&event)?;

    // Queue SetStatus op for sync
    queue_op(
        work_dir,
        config,
        OpPayload::set_status(
            id.to_string(),
            wk_core::Status::Todo,
            reason.map(String::from),
        ),
    )?;

    Ok(())
}

fn reopen_single_with_reason(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    id: &str,
    issue: &crate::models::Issue,
    reason: &str,
) -> Result<()> {
    if !issue.status.can_transition_to(Status::Todo) {
        return Err(Error::InvalidTransition {
            from: issue.status.to_string(),
            to: "todo".to_string(),
            valid_targets: issue.status.valid_targets(),
        });
    }

    db.update_issue_status(id, Status::Todo)?;

    let event = Event::new(id.to_string(), Action::Reopened)
        .with_values(Some(issue.status.to_string()), Some("todo".to_string()))
        .with_reason(Some(reason.to_string()));
    db.log_event(&event)?;

    // Add reason as note (will appear in "Description" section)
    db.add_note(id, Status::Todo, reason)?;

    // Queue SetStatus op for sync
    queue_op(
        work_dir,
        config,
        OpPayload::set_status(
            id.to_string(),
            wk_core::Status::Todo,
            Some(reason.to_string()),
        ),
    )?;

    println!("Reopened {} ({})", id, reason);

    Ok(())
}

/// Resolves the effective reason for a status transition.
///
/// - If reason is provided, validates and returns it
/// - If human interactive, auto-generates "{action} by {name}"
/// - Otherwise, returns an error requiring explicit reason
pub(crate) fn resolve_reason(reason: Option<&str>, action: &str) -> Result<String> {
    if let Some(r) = reason {
        let trimmed = validate_and_trim_reason(r)?;
        if trimmed.is_empty() {
            return Err(Error::InvalidInput("Reason cannot be empty".to_string()));
        }
        return Ok(trimmed);
    }

    // Auto-generate for human interactive sessions
    if is_human_interactive() {
        let name = get_user_name();
        return Ok(format!("Marked as {} by {}", action, name));
    }

    // Require explicit reason for non-interactive/automation contexts
    Err(Error::InvalidInput(
        "--reason is required for agents".to_string(),
    ))
}

/// Log unblocked events for issues that become unblocked when a blocker is completed
fn log_unblocked_events(db: &crate::db::Database, completed_id: &str) -> Result<()> {
    // Get all issues that were blocked by this issue
    let blocked_issues = db.get_blocking(completed_id)?;

    for blocked_id in blocked_issues {
        // Check if this issue still has any open blockers
        let remaining_blockers = db.get_transitive_blockers(&blocked_id)?;

        // If no more open blockers, log an unblocked event
        if remaining_blockers.is_empty() {
            let event = Event::new(blocked_id, Action::Unblocked)
                .with_values(None, Some(completed_id.to_string()));
            db.log_event(&event)?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;
