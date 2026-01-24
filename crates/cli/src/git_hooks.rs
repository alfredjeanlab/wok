// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Git hooks management for wk remote sync.
//!
//! Installs post-push and post-merge hooks that trigger wk remote sync
//! when working with git remotes.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// Marker comment to identify wk hooks.
const WK_HOOK_MARKER: &str = "# wk-remote-sync";

/// The post-push hook script.
const POST_PUSH_HOOK: &str = r#"#!/bin/sh
# wk-remote-sync
# Trigger wk remote sync after pushing to remote
wk remote sync --quiet 2>/dev/null || true
"#;

/// The post-merge hook script.
const POST_MERGE_HOOK: &str = r#"#!/bin/sh
# wk-remote-sync
# Trigger wk remote sync after merging from remote
wk remote sync --quiet 2>/dev/null || true
"#;

/// Find the .git directory for a repository.
pub fn find_git_dir(from: &Path) -> Result<PathBuf> {
    // Try git rev-parse to find the git dir
    let output = Command::new("git")
        .current_dir(from)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| Error::Config(format!("failed to run git: {}", e)))?;

    if output.status.success() {
        let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let git_path = if git_dir.starts_with('/') {
            PathBuf::from(git_dir)
        } else {
            from.join(git_dir)
        };
        return Ok(git_path);
    }

    Err(Error::Config("not a git repository".to_string()))
}

/// Install wk git hooks in a repository.
///
/// This installs post-push and post-merge hooks that trigger wk remote sync.
/// Existing hooks are preserved by appending the wk hook code.
pub fn install_hooks(repo_path: &Path) -> Result<()> {
    let git_dir = find_git_dir(repo_path)?;
    let hooks_dir = git_dir.join("hooks");

    // Ensure hooks directory exists
    fs::create_dir_all(&hooks_dir)?;

    // Install each hook
    install_hook(&hooks_dir, "post-push", POST_PUSH_HOOK)?;
    install_hook(&hooks_dir, "post-merge", POST_MERGE_HOOK)?;

    Ok(())
}

/// Install a single hook.
fn install_hook(hooks_dir: &Path, name: &str, content: &str) -> Result<()> {
    let hook_path = hooks_dir.join(name);

    // Read existing content if hook exists
    let existing = if hook_path.exists() {
        fs::read_to_string(&hook_path)?
    } else {
        String::new()
    };

    // Check if wk hook is already installed
    if existing.contains(WK_HOOK_MARKER) {
        return Ok(());
    }

    // Create new hook content
    let new_content = if existing.is_empty() {
        content.to_string()
    } else {
        // Append to existing hook
        format!("{}\n\n{}", existing.trim(), content)
    };

    // Write hook
    fs::write(&hook_path, new_content)?;

    // Make executable
    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)?;

    Ok(())
}

#[cfg(test)]
#[path = "git_hooks_tests.rs"]
mod tests;
