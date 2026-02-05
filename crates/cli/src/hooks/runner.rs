// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook runner orchestration.

use std::path::Path;

use crate::db::Database;
use crate::error::Result;
use crate::models::Event;

use super::config::{load_hooks_config, HookConfig};
use super::event::HookEvent;
use super::executor::execute_hook;
use super::filter::HookFilter;
use super::payload::HookPayload;

/// Find and execute matching hooks for an event.
///
/// Loads hook configuration, filters matching hooks, and executes them.
/// Errors during individual hook execution are logged but don't fail the operation.
pub fn run_hooks_for_event(db: &Database, work_dir: &Path, event: &Event) -> Result<()> {
    // Load hooks config (if exists)
    let config = match load_hooks_config(work_dir)? {
        Some(c) => c,
        None => return Ok(()), // No hooks configured
    };

    // Get issue from database
    let issue = match db.get_issue(&event.issue_id) {
        Ok(issue) => issue,
        Err(e) => {
            eprintln!("warning: failed to get issue for hooks: {}", e);
            return Ok(());
        }
    };

    // Get labels for issue
    let labels = db.get_labels(&event.issue_id).unwrap_or_default();

    // Convert action to hook event
    let hook_event: HookEvent = event.action.into();

    // Process each hook
    for hook in &config.hooks {
        // Check if event matches this hook's patterns
        if !matches_hook_events(hook, hook_event) {
            continue;
        }

        // Check filter if present
        if let Some(filter_str) = &hook.filter {
            match HookFilter::parse(filter_str) {
                Ok(filter) => {
                    if !filter.matches(&issue, &labels) {
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("warning: invalid filter for hook '{}': {}", hook.name, e);
                    continue;
                }
            }
        }

        // Build payload and execute
        let payload = HookPayload::from_event(event, &issue, labels.clone());
        if let Err(e) = execute_hook(hook, &payload, work_dir) {
            eprintln!("warning: failed to execute hook '{}': {}", hook.name, e);
        }
    }

    Ok(())
}

/// Check if a hook's event patterns match the given event.
fn matches_hook_events(hook: &HookConfig, event: HookEvent) -> bool {
    hook.events
        .iter()
        .any(|pattern| event.matches_pattern(pattern))
}

/// Test a specific hook by name against an issue.
///
/// Returns true if the hook would fire for the given issue with the specified event.
pub fn test_hook(
    db: &Database,
    work_dir: &Path,
    hook_name: &str,
    issue_id: &str,
    test_event: HookEvent,
) -> Result<Option<bool>> {
    // Load hooks config
    let config = match load_hooks_config(work_dir)? {
        Some(c) => c,
        None => return Ok(None), // No hooks configured
    };

    // Find the named hook
    let hook = match config.hooks.iter().find(|h| h.name == hook_name) {
        Some(h) => h,
        None => return Ok(None), // Hook not found
    };

    // Check event match
    if !matches_hook_events(hook, test_event) {
        return Ok(Some(false));
    }

    // Get issue
    let issue = match db.get_issue(issue_id) {
        Ok(i) => i,
        Err(_) => return Ok(None), // Issue not found
    };

    // Get labels
    let labels = db.get_labels(issue_id).unwrap_or_default();

    // Check filter if present
    if let Some(filter_str) = &hook.filter {
        let filter = HookFilter::parse(filter_str)?;
        Ok(Some(filter.matches(&issue, &labels)))
    } else {
        Ok(Some(true))
    }
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
