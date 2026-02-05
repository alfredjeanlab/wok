// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Issue hooks management commands.
//!
//! Commands for listing configured hooks and testing them.

use crate::cli::OutputFormat;
use crate::error::Result;
use crate::hooks::{load_hooks_config, test_hook, HookEvent};

use super::open_db;

/// Run the hook list command.
pub fn list(output: OutputFormat) -> Result<()> {
    let (_, _, work_dir) = open_db()?;
    let config = load_hooks_config(&work_dir)?;

    match config {
        None => {
            if matches!(output, OutputFormat::Json) {
                println!("[]");
            } else {
                println!("No hooks configured.");
                println!();
                println!("To configure hooks, create .wok/hooks.toml:");
                println!("  [[hooks]]");
                println!("  name = \"my-hook\"");
                println!("  events = [\"issue.created\"]");
                println!("  run = \"./scripts/notify.sh\"");
            }
        }
        Some(config) => {
            if matches!(output, OutputFormat::Json) {
                let json = serde_json::to_string_pretty(&config.hooks)
                    .map_err(|e| crate::error::Error::Config(e.to_string()))?;
                println!("{}", json);
            } else {
                println!("Configured hooks:");
                for hook in &config.hooks {
                    println!();
                    println!("  {}", hook.name);
                    println!("    Events: {}", hook.events.join(", "));
                    if let Some(filter) = &hook.filter {
                        println!("    Filter: {}", filter);
                    }
                    println!("    Run: {}", hook.run);
                }
            }
        }
    }

    Ok(())
}

/// Run the hook test command.
pub fn test(name: String, id: String, event: Option<String>) -> Result<()> {
    let (db, _, work_dir) = open_db()?;

    // Resolve the issue ID
    let resolved_id = db.resolve_id(&id)?;

    // Parse the event if provided, default to "issue.created"
    let test_event = match event.as_deref() {
        Some(e) => parse_event(e)?,
        None => HookEvent::Created,
    };

    match test_hook(&db, &work_dir, &name, &resolved_id, test_event)? {
        None => {
            println!(
                "Hook '{}' not found or issue '{}' not found.",
                name, resolved_id
            );
        }
        Some(true) => {
            println!(
                "Hook '{}' would fire for issue '{}' on event '{}'.",
                name,
                resolved_id,
                test_event.as_event_name()
            );
        }
        Some(false) => {
            println!(
                "Hook '{}' would NOT fire for issue '{}' on event '{}' (filter/event mismatch).",
                name,
                resolved_id,
                test_event.as_event_name()
            );
        }
    }

    Ok(())
}

/// Parse an event name into a HookEvent.
fn parse_event(event: &str) -> Result<HookEvent> {
    match event {
        "issue.created" | "created" => Ok(HookEvent::Created),
        "issue.edited" | "edited" => Ok(HookEvent::Edited),
        "issue.started" | "started" => Ok(HookEvent::Started),
        "issue.stopped" | "stopped" => Ok(HookEvent::Stopped),
        "issue.done" | "done" => Ok(HookEvent::Done),
        "issue.closed" | "closed" => Ok(HookEvent::Closed),
        "issue.reopened" | "reopened" => Ok(HookEvent::Reopened),
        "issue.labeled" | "labeled" => Ok(HookEvent::Labeled),
        "issue.unlabeled" | "unlabeled" => Ok(HookEvent::Unlabeled),
        "issue.assigned" | "assigned" => Ok(HookEvent::Assigned),
        "issue.unassigned" | "unassigned" => Ok(HookEvent::Unassigned),
        "issue.noted" | "noted" => Ok(HookEvent::Noted),
        "issue.linked" | "linked" => Ok(HookEvent::Linked),
        "issue.unlinked" | "unlinked" => Ok(HookEvent::Unlinked),
        "issue.related" | "related" => Ok(HookEvent::Related),
        "issue.unrelated" | "unrelated" => Ok(HookEvent::Unrelated),
        "issue.unblocked" | "unblocked" => Ok(HookEvent::Unblocked),
        _ => Err(crate::error::Error::Config(format!(
            "unknown event: {}",
            event
        ))),
    }
}

#[cfg(test)]
#[path = "hook_tests.rs"]
mod tests;
