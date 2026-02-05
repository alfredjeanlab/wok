// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook execution in fire-and-forget mode.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::Result;

use super::config::HookConfig;
use super::payload::HookPayload;

/// Execute a hook script with the given payload.
///
/// Fire-and-forget: spawns process, writes stdin, doesn't wait for completion.
/// Does not check exit codes or handle timeouts.
pub fn execute_hook(hook: &HookConfig, payload: &HookPayload, work_dir: &Path) -> Result<()> {
    // Serialize payload to JSON
    let json = payload.to_json().map_err(|e| {
        crate::error::Error::Config(format!("failed to serialize hook payload: {}", e))
    })?;

    // Get the project root (parent of .wok/)
    let project_root = work_dir.parent().unwrap_or(work_dir);

    // Build the command
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&hook.run)
        .current_dir(project_root)
        .env("WOK_EVENT", &payload.event)
        .env("WOK_ISSUE_ID", &payload.issue.id)
        .env("WOK_ISSUE_TYPE", &payload.issue.r#type)
        .env("WOK_ISSUE_STATUS", &payload.issue.status)
        .env(
            "WOK_CHANGE_VALUE",
            payload.change.new_value.as_deref().unwrap_or(""),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            crate::error::Error::Config(format!("failed to spawn hook '{}': {}", hook.name, e))
        })?;

    // Write JSON to stdin
    if let Some(mut stdin) = child.stdin.take() {
        // Best effort write, ignore errors (fire-and-forget)
        let _ = stdin.write_all(json.as_bytes());
    }

    // Don't wait for completion - fire and forget
    // The process will be adopted by init when we drop the handle
    drop(child);

    Ok(())
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod tests;
