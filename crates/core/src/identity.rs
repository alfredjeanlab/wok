// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! User identity detection for audit purposes.
//!
//! This module provides utilities to determine the current user's display name
//! for auto-populating reasons in status transitions.

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;

use std::process::Command;

/// Returns the current user's display name for audit purposes.
///
/// Resolution order:
/// 1. Git config user.name (display name only, never email)
/// 2. Unix username from USER or LOGNAME env var (if not system account)
/// 3. Fallback to "human"
pub fn get_user_name() -> String {
    // Try git config user.name first
    if let Some(name) = get_git_user_name() {
        return name;
    }

    // Try Unix username
    if let Some(name) = get_unix_username() {
        if !is_system_account(&name) {
            return name;
        }
    }

    // Fallback
    "human".to_string()
}

fn get_git_user_name() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn get_unix_username() -> Option<String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .ok()
        .filter(|s| !s.is_empty())
}

fn is_system_account(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "root" | "system" | "administrator" | "admin" | "daemon" | "nobody"
    )
}
