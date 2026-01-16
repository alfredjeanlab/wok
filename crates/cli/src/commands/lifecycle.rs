// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use wk_core::detect::is_human_interactive;
use wk_core::identity::get_user_name;
use wk_core::OpPayload;

use crate::config::{find_work_dir, get_db_path, Config};
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{Action, Event, Status};
use crate::validate::validate_and_trim_reason;

use super::queue_op;

pub fn start(ids: &[String]) -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
    start_impl(&db, &config, &work_dir, ids)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn start_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
) -> Result<()> {
    for id in ids {
        start_single(db, config, work_dir, id)?;
    }
    Ok(())
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

    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
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
    for id in ids {
        done_single(db, config, work_dir, id, reason)?;
    }
    Ok(())
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
        let effective_reason = resolve_reason(None, "Completed")?;
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
    let effective_reason = resolve_reason(reason, "Closed")?;

    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
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
    for id in ids {
        close_single(db, config, work_dir, id, reason)?;
    }
    Ok(())
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

    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
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
    for id in ids {
        reopen_single(db, config, work_dir, id, reason)?;
    }
    Ok(())
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
        let effective_reason = resolve_reason(None, "Reopened")?;
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
        return Ok(format!("{} by {}", action, name));
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
