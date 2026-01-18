// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::{Path, PathBuf};

use crate::completions;
use crate::config::{
    init_work_dir, init_workspace_link, write_gitignore, Config, RemoteConfig, RemoteType,
};
use crate::db::Database;
use crate::error::{Error, Result};
use crate::git_hooks;
use crate::id::validate_prefix;
use crate::worktree;

pub fn run(
    prefix: Option<String>,
    path: Option<String>,
    workspace: Option<String>,
    remote: Option<String>,
    local: bool,
) -> Result<()> {
    let target_path = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()?,
    };

    // If workspace is specified, create workspace-link config only
    if let Some(ws) = workspace {
        // Validate prefix if provided
        if let Some(ref p) = prefix {
            if !validate_prefix(p) {
                return Err(Error::InvalidPrefix);
            }
        }

        let work_dir = init_workspace_link(&target_path, &ws, prefix.as_deref())?;

        // Workspace links are always local (no remote config), so include config.toml in gitignore
        write_gitignore(&work_dir, true)?;

        println!("Initialized workspace link at {}", work_dir.display());
        println!("Workspace: {}", ws);
        if let Some(p) = prefix {
            println!("Prefix: {}", p);
        }

        return Ok(());
    }

    // Original behavior: full init with prefix and database
    let prefix = match prefix {
        Some(p) => p,
        None => derive_prefix_from_path(&target_path)?,
    };

    // Validate the prefix
    if !validate_prefix(&prefix) {
        return Err(Error::InvalidPrefix);
    }

    let work_dir = init_work_dir(&target_path, &prefix)?;

    // Initialize the database
    let db_path = work_dir.join("issues.db");
    Database::open(&db_path)?;

    // Create .gitignore (include config.toml if local mode)
    write_gitignore(&work_dir, local)?;

    println!("Initialized issue tracker at {}", work_dir.display());
    println!("Prefix: {}", prefix);

    // Set up remote: default to "." (git orphan branch) unless --local is specified
    if !local {
        let remote_url = remote.as_deref().unwrap_or(".");
        setup_remote(&work_dir, &target_path, remote_url)?;
    }

    // Install shell completions
    if let Err(e) = completions::install_all() {
        eprintln!("Warning: failed to install shell completions: {}", e);
    }

    Ok(())
}

/// Set up a remote for the issue tracker.
fn setup_remote(work_dir: &Path, repo_path: &Path, remote_url: &str) -> Result<()> {
    // Determine the URL format
    let url = if remote_url == "." {
        "git:.".to_string()
    } else if remote_url.starts_with("ws://")
        || remote_url.starts_with("wss://")
        || remote_url.starts_with("git:")
    {
        // WebSocket or already prefixed git URL
        remote_url.to_string()
    } else {
        // SSH URL or path - treat as git
        format!("git:{}", remote_url)
    };

    // Create remote config
    let remote_config = RemoteConfig {
        url: url.clone(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };

    // Load and update config
    let mut config = Config::load(work_dir)?;
    config.remote = Some(remote_config.clone());
    config.save(work_dir)?;

    println!("Remote: {}", url);

    // For git remotes, set up the oplog worktree
    if remote_config.remote_type() == RemoteType::Git {
        // Initialize the oplog worktree (creates .git/wk/oplog for same-repo mode)
        if remote_config.is_same_repo() {
            match worktree::init_oplog_worktree(work_dir, &remote_config) {
                Ok(wt) => {
                    println!("Created oplog worktree at {}", wt.path.display());
                }
                Err(e) => {
                    eprintln!("Warning: failed to create oplog worktree: {}", e);
                }
            }
        }

        // Install git hooks
        if let Err(e) = git_hooks::install_hooks(repo_path) {
            eprintln!("Warning: failed to install git hooks: {}", e);
        } else {
            println!("Installed git hooks (post-push, post-merge)");
        }
    }

    Ok(())
}

/// Derive a prefix from the directory path.
/// Uses the directory name, converted to lowercase, keeping letters and digits.
fn derive_prefix_from_path(path: &Path) -> Result<String> {
    let dir_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::InvalidInput("Cannot derive prefix from path".to_string()))?;

    // Convert to lowercase and keep only ASCII alphanumeric characters
    let prefix: String = dir_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    // Must have at least 2 characters and contain at least one letter
    if prefix.len() < 2 || !prefix.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(Error::InvalidInput(
            "Cannot derive prefix from directory name (need 2+ chars with at least one letter)"
                .to_string(),
        ));
    }

    Ok(prefix)
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
