// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use wk_core::{HlcClock, Op, OpPayload};

use crate::cli::ConfigCommand;
use crate::config::{
    find_work_dir, get_daemon_dir, write_gitignore, Config, RemoteConfig, RemoteType,
};
use crate::daemon::{detect_daemon, request_sync, spawn_daemon};
use crate::db::Database;
use crate::error::{Error, Result};
use crate::git_hooks;
use crate::id::validate_prefix;
use crate::wal::Wal;

use super::open_db;

/// Execute a config subcommand.
pub fn run(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Rename {
            old_prefix,
            new_prefix,
        } => {
            let (db, config) = open_db()?;
            let work_dir = find_work_dir()?;
            run_rename_prefix(&db, &config, &work_dir, &old_prefix, &new_prefix)
        }
        ConfigCommand::Remote { url } => run_config_remote(&url),
    }
}

/// Configure remote sync for an existing local-mode tracker.
fn run_config_remote(url: &str) -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;

    // Workspace and remote are incompatible - workspace stores the database elsewhere,
    // but remote sync requires a single .wok/ location for the daemon to manage
    if config.workspace.is_some() {
        return Err(Error::WorkspaceRemoteIncompatible);
    }

    // Case 1: Already in remote mode
    if config.is_remote_mode() {
        let existing_url = config.remote_url().unwrap_or("");
        let normalized_url = normalize_remote_url(url);

        if existing_url == normalized_url {
            println!("Remote already configured as '{}'", existing_url);
            return Ok(());
        }

        // Case 2: Changing remotes - not yet supported
        println!("Changing remotes is not currently supported.");
        println!("Current remote: {}", existing_url);
        println!("Requested remote: {}", normalized_url);
        return Ok(());
    }

    // Case 3: Local → Remote (supported)
    let repo_path = work_dir.parent().unwrap_or(&work_dir);
    setup_remote(&work_dir, repo_path, url)?;

    // Update .gitignore (remote mode doesn't ignore config.toml)
    write_gitignore(&work_dir, false)?;

    // Trigger immediate sync
    trigger_sync(&work_dir)?;

    Ok(())
}

/// Normalize a remote URL to canonical form.
fn normalize_remote_url(url: &str) -> String {
    if url == "." {
        "git:.".to_string()
    } else if url.starts_with("ws://") || url.starts_with("wss://") || url.starts_with("git:") {
        url.to_string()
    } else {
        format!("git:{}", url)
    }
}

/// Set up remote configuration.
fn setup_remote(work_dir: &Path, repo_path: &Path, remote_url: &str) -> Result<()> {
    let url = normalize_remote_url(remote_url);

    let remote_config = RemoteConfig {
        url: url.clone(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
    };

    let mut config = Config::load(work_dir)?;
    config.remote = Some(remote_config.clone());
    config.save(work_dir)?;

    println!("Remote configured: {}", url);

    // Install git hooks if this is a git remote
    if remote_config.remote_type() == RemoteType::Git {
        if let Err(e) = git_hooks::install_hooks(repo_path) {
            eprintln!("Warning: failed to install git hooks: {}", e);
        } else {
            println!("Installed git hooks (post-push, post-merge)");
        }
    }

    Ok(())
}

/// Spawn daemon if needed and request sync.
fn trigger_sync(work_dir: &Path) -> Result<()> {
    let config = Config::load(work_dir)?;
    let daemon_dir = get_daemon_dir(work_dir, &config);

    // Spawn daemon if not running
    if detect_daemon(&daemon_dir)?.is_none() {
        println!("Starting sync daemon...");
        match spawn_daemon(&daemon_dir, work_dir) {
            Ok(info) => {
                println!("Daemon started (PID: {})", info.pid);
            }
            Err(e) => {
                eprintln!("Warning: failed to start daemon: {}", e);
                return Ok(()); // Continue anyway
            }
        }
        // spawn_daemon already waits for "READY" and verifies with detect_daemon
    }

    // Request sync
    println!("Syncing...");
    match request_sync(&daemon_dir) {
        Ok(ops_synced) => {
            println!("Sync complete. {} operations synced.", ops_synced);
        }
        Err(e) => {
            eprintln!("Initial sync failed: {}", e);
            println!("Daemon will retry automatically.");
        }
    }

    Ok(())
}

/// Rename the issue ID prefix across all issues and config.
pub(crate) fn run_rename_prefix(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    old_prefix: &str,
    new_prefix: &str,
) -> Result<()> {
    // 1. Validate old prefix
    if !validate_prefix(old_prefix) {
        return Err(Error::InvalidPrefix);
    }

    // 2. Validate new prefix
    if !validate_prefix(new_prefix) {
        return Err(Error::InvalidPrefix);
    }

    // 3. Check if prefix is unchanged
    if old_prefix == new_prefix {
        println!("Prefix is already '{}'", new_prefix);
        return Ok(());
    }

    // 4. Create operation for sync
    let op = create_config_rename_op(old_prefix, new_prefix);

    // 5. Update all issue IDs in database (within transaction)
    rename_all_issue_ids(db, old_prefix, new_prefix)?;

    // 6. Write to WAL for sync if in remote mode
    if config.is_remote_mode() {
        let daemon_dir = get_daemon_dir(work_dir, config);
        write_op_to_wal(&daemon_dir, &op)?;
    }

    // 7. Update config file if old_prefix matches current config prefix
    if config.prefix == old_prefix {
        let mut new_config = config.clone();
        new_config.prefix = new_prefix.to_string();
        new_config.save(work_dir)?;
    }

    println!("Renamed prefix from '{}' to '{}'", old_prefix, new_prefix);
    Ok(())
}

/// Create a ConfigRename operation with a generated HLC.
fn create_config_rename_op(old_prefix: &str, new_prefix: &str) -> Op {
    // Generate a simple node_id from process ID
    // This ensures uniqueness per-process which is sufficient for CLI operations
    let node_id = std::process::id();
    let clock = HlcClock::new(node_id);
    let hlc = clock.now();

    Op::new(
        hlc,
        OpPayload::config_rename(old_prefix.to_string(), new_prefix.to_string()),
    )
}

/// Write an operation to the WAL for the daemon to sync.
fn write_op_to_wal(daemon_dir: &Path, op: &Op) -> Result<()> {
    let wal_path = daemon_dir.join("pending_ops.jsonl");
    let wal = Wal::open(&wal_path)?;
    wal.append(op)?;
    Ok(())
}

/// Rename all issue IDs in the database from old_prefix to new_prefix.
/// Uses a transaction to ensure atomicity.
fn rename_all_issue_ids(db: &Database, old_prefix: &str, new_prefix: &str) -> Result<()> {
    let old_pattern = format!("{}-", old_prefix);
    let new_pattern = format!("{}-", new_prefix);
    let like_pattern = format!("{}%", old_pattern);

    // Disable foreign keys, perform updates, then re-enable
    // Note: PRAGMA foreign_keys cannot be changed inside a transaction,
    // so we handle this carefully.
    db.conn.execute("PRAGMA foreign_keys = OFF", [])?;

    let result = (|| -> Result<()> {
        let tx = db.conn.unchecked_transaction()?;

        // Update issues table (primary)
        tx.execute(
            "UPDATE issues SET id = replace(id, ?1, ?2) WHERE id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;

        // Update deps table (both columns)
        tx.execute(
            "UPDATE deps SET from_id = replace(from_id, ?1, ?2) WHERE from_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE deps SET to_id = replace(to_id, ?1, ?2) WHERE to_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;

        // Update labels, notes, events, links tables
        tx.execute(
            "UPDATE labels SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE notes SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE events SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;
        tx.execute(
            "UPDATE links SET issue_id = replace(issue_id, ?1, ?2) WHERE issue_id LIKE ?3",
            [&old_pattern, &new_pattern, &like_pattern],
        )?;

        tx.commit()?;
        Ok(())
    })();

    // Re-enable foreign keys regardless of success/failure
    db.conn.execute("PRAGMA foreign_keys = ON", [])?;

    result
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
