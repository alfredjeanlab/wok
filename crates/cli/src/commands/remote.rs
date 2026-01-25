// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Remote sync management commands.
//!
//! Commands for controlling remote synchronization in remote mode.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::config::{find_work_dir, get_daemon_dir, Config};
use crate::daemon::{
    detect_daemon, ensure_compatible_daemon, get_daemon_status, request_sync,
    stop_daemon_forcefully, wait_daemon_connected,
};
use crate::error::{Error, Result};
use crate::mode::OperatingMode;

/// Show remote sync status.
pub fn status() -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let daemon_dir = get_daemon_dir(&work_dir, &config);
    let mode = OperatingMode::detect(&config);

    match mode {
        OperatingMode::Local => {
            println!("Status: not applicable (no remote configured)");
            println!();
            println!("To enable remote sync, add a [remote] section to .work/config.toml:");
            println!();
            println!("  [remote]");
            println!("  url = \"ws://your-server:7890\"");
        }
        OperatingMode::Remote => {
            let remote = config.remote.as_ref().ok_or(Error::Config(
                "remote mode detected but no remote config".to_string(),
            ))?;

            // Try to get status from running daemon
            match get_daemon_status(&daemon_dir) {
                Ok(Some(status)) => {
                    let status_str = if status.connected {
                        "connected"
                    } else {
                        "disconnected"
                    };
                    println!("Status: daemon running ({})", status_str);
                    println!("PID: {}", status.pid);
                    println!("Remote: {}", status.remote_url);
                    println!("Pending ops: {}", status.pending_ops);
                    if let Some(ts) = status.last_sync {
                        let dt = DateTime::<Utc>::from_timestamp(i64::try_from(ts).unwrap_or(0), 0);
                        if let Some(dt) = dt {
                            println!("Last sync: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
                        } else {
                            println!("Last sync: {}", ts);
                        }
                    } else {
                        println!("Last sync: never");
                    }
                    println!("Uptime: {}s", status.uptime_secs);
                }
                Ok(None) => {
                    println!("Status: daemon not running");
                    println!("Remote: {}", remote.url);
                    println!();
                    println!("Run 'wok remote sync' to start syncing.");
                }
                Err(e) => {
                    println!("Status: error checking daemon");
                    println!("Error: {}", e);
                    println!("Remote: {}", remote.url);
                }
            }
        }
    }

    Ok(())
}

/// Stop the sync daemon.
pub fn stop() -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let daemon_dir = get_daemon_dir(&work_dir, &config);
    let mode = OperatingMode::detect(&config);

    match mode {
        OperatingMode::Local => {
            println!("Not in remote mode - no daemon to stop.");
        }
        OperatingMode::Remote => {
            // Check if daemon is running
            if detect_daemon(&daemon_dir)?.is_none() {
                println!("Daemon is not running.");
                return Ok(());
            }

            // Stop the daemon (forcefully if graceful shutdown times out)
            match stop_daemon_forcefully(&daemon_dir) {
                Ok(()) => {
                    println!("Daemon stopped.");
                }
                Err(e) => {
                    println!("Failed to stop daemon: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Sync now with remote server.
pub fn sync(force: bool, quiet: bool) -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let daemon_dir = get_daemon_dir(&work_dir, &config);
    let mode = OperatingMode::detect(&config);

    match mode {
        OperatingMode::Local => {
            if !quiet {
                println!("Not in remote mode - nothing to sync.");
                println!();
                println!("To enable remote sync, add a [remote] section to .work/config.toml:");
                println!();
                println!("  [remote]");
                println!("  url = \"ws://your-server:7890\"");
            }
        }
        OperatingMode::Remote => {
            let remote = config.remote.as_ref().ok_or(Error::Config(
                "remote mode detected but no remote config".to_string(),
            ))?;

            // Validate the remote URL before attempting to sync
            if let Some(error_msg) = remote.validate_url() {
                return Err(Error::Config(error_msg));
            }

            // Ensure a compatible daemon is running (handles version mismatch)
            let was_running = detect_daemon(&daemon_dir)?.is_some();
            match ensure_compatible_daemon(&daemon_dir, &work_dir) {
                Ok(info) => {
                    if !was_running {
                        println!("Daemon started (PID: {})", info.pid);
                        // Wait for daemon to establish connection before requesting sync.
                        // The daemon signals READY as soon as IPC is available, but the
                        // WebSocket connection is established asynchronously.
                        let timeout = Duration::from_secs(remote.connect_timeout_secs);
                        if !wait_daemon_connected(&daemon_dir, timeout)? {
                            // Daemon couldn't connect within timeout - let sync proceed anyway
                            // since it will queue ops for later if still disconnected
                            eprintln!("Warning: daemon still connecting to server...");
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to start daemon: {}", e);
                    return Ok(());
                }
            }

            // Request sync from daemon
            if force {
                println!("Forcing full resync with {}...", remote.url);
            } else {
                println!("Syncing with {}...", remote.url);
            }
            match request_sync(&daemon_dir) {
                Ok(ops_synced) => {
                    println!("Sync complete. {} operations synced.", ops_synced);
                }
                Err(e) => {
                    return Err(Error::Sync(format!("Sync failed: {}", e)));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "remote_tests.rs"]
mod tests;
