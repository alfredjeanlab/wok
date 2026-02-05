// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook execution in fire-and-forget mode.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use super::{HookConfig, HookPayload};
use crate::error::Result;

/// Execute a hook script with the given payload.
///
/// This is fire-and-forget: the process is spawned, stdin is written, and the
/// function returns without waiting for completion. Exit codes are not checked.
///
/// # Environment Variables
///
/// The following environment variables are set for the hook script:
/// - `WOK_EVENT`: Event type (e.g., "issue.created")
/// - `WOK_ISSUE_ID`: Issue ID (e.g., "proj-a1b2")
/// - `WOK_ISSUE_TYPE`: Issue type (e.g., "bug")
/// - `WOK_ISSUE_STATUS`: Issue status (e.g., "in_progress")
/// - `WOK_CHANGE_VALUE`: New value from the change (if any)
///
/// # Errors
///
/// Returns an error if the process cannot be spawned or stdin cannot be written.
pub fn execute_hook(hook: &HookConfig, payload: &HookPayload, work_dir: &Path) -> Result<()> {
    // Get the project root (parent of .wok)
    let project_root = work_dir.parent().unwrap_or(Path::new("."));

    // Serialize payload to JSON
    let json = serde_json::to_string(payload)
        .map_err(|e| crate::error::Error::Config(format!("failed to serialize payload: {}", e)))?;

    // Build the command
    // Use shell to allow for more complex command strings
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&hook.run)
        .current_dir(project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("WOK_EVENT", &payload.event)
        .env("WOK_ISSUE_ID", &payload.issue.id)
        .env("WOK_ISSUE_TYPE", &payload.issue.issue_type)
        .env("WOK_ISSUE_STATUS", &payload.issue.status)
        .env(
            "WOK_CHANGE_VALUE",
            payload.change.new_value.as_deref().unwrap_or(""),
        )
        .spawn()
        .map_err(|e| crate::error::Error::Config(format!("failed to spawn hook: {}", e)))?;

    // Write JSON to stdin, then drop to close the pipe
    if let Some(mut stdin) = child.stdin.take() {
        // Ignore write errors - fire and forget
        let _ = stdin.write_all(json.as_bytes());
    }

    // Don't wait for completion - fire and forget
    // The child process will be orphaned and run independently

    Ok(())
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod tests;
