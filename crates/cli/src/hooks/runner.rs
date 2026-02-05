// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook runner for orchestrating hook execution.

use std::path::Path;

use crate::models::Event;

use super::{execute_hook, load_hooks_config, HookEvent, HookFilter, HookPayload};
use crate::db::Database;
use crate::error::Result;

/// Find and execute matching hooks for an event.
///
/// This function:
/// 1. Loads hooks configuration from `.wok/hooks.toml` and/or `.wok/hooks.json`
/// 2. Retrieves the issue and its labels from the database
/// 3. For each configured hook:
///    - Checks if the event matches the hook's event patterns
///    - Parses and applies the hook's filter (if any)
///    - Executes the hook if all conditions match
///
/// Errors during hook execution are logged but don't fail the overall operation.
pub fn run_hooks_for_event(db: &Database, work_dir: &Path, event: &Event) -> Result<()> {
    // Load hooks config
    let config = match load_hooks_config(work_dir)? {
        Some(config) => config,
        None => return Ok(()), // No hooks configured
    };

    if config.hooks.is_empty() {
        return Ok(());
    }

    // Get the issue from database
    let issue = db.get_issue(&event.issue_id)?;
    let labels = db.get_labels(&event.issue_id)?;

    // Convert action to hook event
    let hook_event: HookEvent = event.action.into();

    // Build payload once for all hooks
    let payload = HookPayload::build(event, &issue, &labels);

    // Process each hook
    for hook in &config.hooks {
        // Check if event matches hook's event patterns
        let event_matches = hook
            .events
            .iter()
            .any(|pattern| hook_event.matches_pattern(pattern));

        if !event_matches {
            continue;
        }

        // Check filter if present
        if let Some(ref filter_str) = hook.filter {
            match HookFilter::parse(filter_str) {
                Ok(filter) => {
                    if !filter.matches(&issue, &labels) {
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("warning: hook '{}' has invalid filter: {}", hook.name, e);
                    continue;
                }
            }
        }

        // Execute the hook
        if let Err(e) = execute_hook(hook, &payload, work_dir) {
            eprintln!("warning: failed to execute hook '{}': {}", hook.name, e);
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
