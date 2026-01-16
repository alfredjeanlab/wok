// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Claude Code hooks installation and management.
//!
//! This module provides utilities to install, uninstall, and check status
//! of Claude Code hooks that integrate wk with AI assistants.

use std::fs;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod tests;

/// Scope for hooks installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookScope {
    /// ./.claude/settings.local.json (git-ignored, per-project)
    Local,
    /// ./.claude/settings.json (per-project, committed)
    Project,
    /// ~/.claude/settings.json (per-machine)
    User,
}

impl HookScope {
    /// Parse scope from string argument.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "local" => Some(HookScope::Local),
            "project" => Some(HookScope::Project),
            "user" => Some(HookScope::User),
            _ => None,
        }
    }

    /// Get the settings file path for this scope.
    pub fn settings_path(&self) -> io::Result<PathBuf> {
        match self {
            HookScope::Local => Ok(PathBuf::from(".claude/settings.local.json")),
            HookScope::Project => Ok(PathBuf::from(".claude/settings.json")),
            HookScope::User => {
                let home = dirs::home_dir().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Could not determine home directory",
                    )
                })?;
                Ok(home.join(".claude/settings.json"))
            }
        }
    }

    /// Human-readable name for display.
    pub fn display_name(&self) -> &'static str {
        match self {
            HookScope::Local => "local",
            HookScope::Project => "project",
            HookScope::User => "user",
        }
    }
}

/// Result of checking hooks status for a scope.
#[derive(Debug)]
pub struct HookStatus {
    pub scope: HookScope,
    pub installed: bool,
    pub path: PathBuf,
}

/// The events where wk hooks should be installed.
const WK_HOOK_EVENTS: &[&str] = &["PreCompact", "SessionStart"];

/// Check if a hook entry contains a "wk prime" command.
///
/// A hook entry is considered a wk hook if any command in its hooks array
/// contains "wk prime". This handles:
/// - `wk prime` (plain command)
/// - `/path/to/wk prime` (full path)
/// - `wk prime --args` (with arguments)
pub fn is_wk_hook(hook_entry: &serde_json::Value) -> bool {
    hook_entry
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|cmd| {
                        let trimmed = cmd.trim();
                        trimmed.starts_with("wk prime") || trimmed.contains("/wk prime")
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Create a wk hook entry for installation.
fn create_wk_hook_entry() -> serde_json::Value {
    serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "wk prime"}]
    })
}

/// Merge wk hooks into existing configuration.
///
/// This function:
/// - Preserves all non-wk hooks
/// - Only adds wk hooks if not already present (idempotent)
/// - Maintains existing hook order, appends wk hooks at end
fn merge_wk_hooks(settings: &mut serde_json::Value) {
    // Ensure hooks object exists
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }

    let hooks = match settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        Some(h) => h,
        None => return,
    };

    for event in WK_HOOK_EVENTS {
        // Ensure event array exists
        if !hooks.contains_key(*event) {
            hooks.insert(event.to_string(), serde_json::json!([]));
        }

        if let Some(event_hooks) = hooks.get_mut(*event).and_then(|e| e.as_array_mut()) {
            // Check if wk hook already exists
            let has_wk = event_hooks.iter().any(is_wk_hook);
            if !has_wk {
                event_hooks.push(create_wk_hook_entry());
            }
        }
    }
}

/// Install hooks to the specified scope.
///
/// Uses smart merging to:
/// - Preserve existing hooks that don't match "wk prime"
/// - Only add wk hooks if not already present (idempotent)
/// - Maintain existing hook order
pub fn install_hooks(scope: HookScope) -> io::Result<PathBuf> {
    let path = scope.settings_path()?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read existing settings or start fresh
    let mut settings: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Smart merge wk hooks
    merge_wk_hooks(&mut settings);

    // Write back
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(&path, content)?;

    Ok(path)
}

/// Remove wk hooks from configuration while preserving others.
///
/// This function:
/// - Only removes hooks containing "wk prime"
/// - Preserves other hooks in the same event array
/// - Removes event key only if array becomes empty
/// - Removes hooks key only if object becomes empty
fn remove_wk_hooks(settings: &mut serde_json::Value) {
    let hooks = match settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        Some(h) => h,
        None => return,
    };

    // Remove wk hooks from each event
    let mut empty_events = Vec::new();
    for (event, event_hooks) in hooks.iter_mut() {
        if let Some(arr) = event_hooks.as_array_mut() {
            arr.retain(|hook| !is_wk_hook(hook));
            if arr.is_empty() {
                empty_events.push(event.clone());
            }
        }
    }

    // Remove empty event arrays
    for event in empty_events {
        hooks.remove(&event);
    }

    // Remove hooks key if empty
    if hooks.is_empty() {
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("hooks");
        }
    }
}

/// Uninstall hooks from the specified scope.
///
/// Uses smart removal to:
/// - Only remove hooks containing "wk prime"
/// - Preserve other hooks in the same event array
/// - Remove event key only if array becomes empty
/// - Remove hooks key only if object becomes empty
pub fn uninstall_hooks(scope: HookScope) -> io::Result<()> {
    let path = scope.settings_path()?;

    if !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;
    let mut settings: serde_json::Value =
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));

    // Smart remove wk hooks
    remove_wk_hooks(&mut settings);

    // If empty, remove the file; otherwise write back
    if settings.as_object().is_none_or(|o| o.is_empty()) {
        fs::remove_file(&path)?;
    } else {
        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, content)?;
    }

    Ok(())
}

/// Check if hooks are installed in the specified scope.
pub fn check_hooks(scope: HookScope) -> io::Result<HookStatus> {
    let path = scope.settings_path()?;
    let installed = if path.exists() {
        let content = fs::read_to_string(&path)?;
        let settings: serde_json::Value =
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
        settings.get("hooks").is_some()
    } else {
        false
    };

    Ok(HookStatus {
        scope,
        installed,
        path,
    })
}

/// Check hooks status for all scopes.
pub fn check_all_hooks() -> Vec<HookStatus> {
    let scopes = [HookScope::Local, HookScope::Project, HookScope::User];
    scopes
        .iter()
        .filter_map(|&scope| check_hooks(scope).ok())
        .collect()
}

/// Determine if we should use interactive mode.
///
/// Returns false if:
/// - stdout is not a TTY
/// - Running under an AI assistant (detected via environment)
/// - CI environment detected
/// - Process is running in the background (e.g., `cmd &`)
pub fn should_use_interactive() -> bool {
    // Not a TTY
    if !std::io::stdout().is_terminal() {
        return false;
    }

    // Running under AI assistant
    if crate::detect::is_ai_subprocess() {
        return false;
    }

    // CI environment
    if std::env::var_os("CI").is_some() {
        return false;
    }

    // Background process
    if !crate::detect::is_foreground_process() {
        return false;
    }

    true
}
