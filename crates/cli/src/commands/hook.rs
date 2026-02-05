// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Commands for managing issue hooks.

use crate::models::{Action, Event};

use super::open_db;
use crate::cli::OutputFormat;
use crate::config::find_work_dir;
use crate::error::Result;
use crate::hooks::{execute_hook, load_hooks_config, HookEvent, HookFilter, HookPayload};

/// List configured hooks.
pub fn list(output: OutputFormat) -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = load_hooks_config(&work_dir)?;

    match config {
        None => {
            if matches!(output, OutputFormat::Json) {
                println!("[]");
            } else {
                println!("No hooks configured");
                println!();
                println!("Create .wok/hooks.toml or .wok/hooks.json to configure hooks.");
            }
        }
        Some(config) => {
            if config.hooks.is_empty() {
                if matches!(output, OutputFormat::Json) {
                    println!("[]");
                } else {
                    println!("No hooks configured");
                }
                return Ok(());
            }

            if matches!(output, OutputFormat::Json) {
                let json = serde_json::to_string_pretty(&config.hooks)
                    .map_err(|e| crate::error::Error::Config(format!("JSON error: {}", e)))?;
                println!("{}", json);
            } else {
                for hook in &config.hooks {
                    println!("{}:", hook.name);
                    println!("  events: {}", hook.events.join(", "));
                    if let Some(ref filter) = hook.filter {
                        println!("  filter: {}", filter);
                    }
                    println!("  run: {}", hook.run);
                    println!();
                }
            }
        }
    }

    Ok(())
}

/// Test a hook by simulating an event.
pub fn test(hook_name: &str, issue_id: &str) -> Result<()> {
    let work_dir = find_work_dir()?;
    let (db, _config, _work_dir) = open_db()?;

    // Load hooks config
    let config = load_hooks_config(&work_dir)?
        .ok_or_else(|| crate::error::Error::Config("no hooks configured".to_string()))?;

    // Find the named hook
    let hook = config
        .hooks
        .iter()
        .find(|h| h.name == hook_name)
        .ok_or_else(|| crate::error::Error::Config(format!("hook '{}' not found", hook_name)))?;

    // Resolve the issue ID
    let resolved_id = db.resolve_id(issue_id)?;
    let issue = db.get_issue(&resolved_id)?;
    let labels = db.get_labels(&resolved_id)?;

    // Determine the event to simulate (use first event from hook config, or issue.created)
    let event_name = hook
        .events
        .first()
        .map(String::as_str)
        .unwrap_or("issue.created");
    let action = match event_name {
        "issue.created" | "issue.*" => Action::Created,
        "issue.edited" => Action::Edited,
        "issue.started" => Action::Started,
        "issue.stopped" => Action::Stopped,
        "issue.done" => Action::Done,
        "issue.closed" => Action::Closed,
        "issue.reopened" => Action::Reopened,
        "issue.labeled" => Action::Labeled,
        "issue.unlabeled" => Action::Unlabeled,
        "issue.assigned" => Action::Assigned,
        "issue.unassigned" => Action::Unassigned,
        "issue.noted" => Action::Noted,
        "issue.linked" => Action::Linked,
        "issue.unlinked" => Action::Unlinked,
        "issue.related" => Action::Related,
        "issue.unrelated" => Action::Unrelated,
        "issue.unblocked" => Action::Unblocked,
        _ => Action::Created,
    };

    // Create a simulated event
    let event = Event::new(resolved_id.clone(), action);
    let hook_event: HookEvent = event.action.into();

    println!("Testing hook '{}'", hook_name);
    println!("  Issue: {} ({})", issue.id, issue.title);
    println!("  Event: {}", hook_event.as_event_name());

    // Check filter if present
    if let Some(ref filter_str) = hook.filter {
        let filter = HookFilter::parse(filter_str)?;
        let matches = filter.matches(&issue, &labels);
        println!(
            "  Filter: {} ({})",
            filter_str,
            if matches { "MATCH" } else { "NO MATCH" }
        );
        if !matches {
            println!();
            println!("Hook would NOT trigger - filter does not match");
            return Ok(());
        }
    }

    // Build payload and execute
    let payload = HookPayload::build(&event, &issue, &labels);
    println!("  Command: {}", hook.run);
    println!();
    println!("Executing hook...");

    execute_hook(hook, &payload, &work_dir)?;

    println!("Hook triggered (fire-and-forget)");

    Ok(())
}

#[cfg(test)]
#[path = "hook_tests.rs"]
mod tests;
