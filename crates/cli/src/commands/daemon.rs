// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Daemon management commands.
//!
//! Commands for controlling the wokd daemon that manages the shared
//! user-level database.

use crate::config::wok_state_dir;
use crate::daemon;
use crate::error::{Error, Result};

/// Show daemon status.
pub fn status() -> Result<()> {
    let daemon_dir = wok_state_dir();

    match daemon::get_daemon_status(&daemon_dir) {
        Ok(Some(status)) => {
            println!("Status: running");
            println!("PID: {}", status.pid);
            println!("Uptime: {}s", status.uptime_secs);
        }
        Ok(None) => {
            println!("Status: not running");
        }
        Err(e) => {
            println!("Status: error ({})", e);
        }
    }

    Ok(())
}

/// Stop the daemon.
pub fn stop() -> Result<()> {
    let daemon_dir = wok_state_dir();

    if daemon::detect_daemon(&daemon_dir)?.is_none() {
        println!("Daemon is not running.");
        return Ok(());
    }

    match daemon::stop_daemon_forcefully(&daemon_dir) {
        Ok(()) => {
            println!("Daemon stopped.");
        }
        Err(e) => {
            println!("Failed to stop daemon: {}", e);
        }
    }

    Ok(())
}

/// Start the daemon.
pub fn start(foreground: bool) -> Result<()> {
    let daemon_dir = wok_state_dir();

    if foreground {
        // Run daemon in foreground (for debugging)
        // This would normally call wokd directly, but for now just spawn
        println!("Starting daemon in foreground...");
        println!("State directory: {}", daemon_dir.display());
        return Err(Error::Daemon(
            "foreground mode requires running wokd directly".to_string(),
        ));
    }

    match daemon::detect_daemon(&daemon_dir)? {
        Some(info) => {
            println!("Daemon is already running (PID: {})", info.pid);
        }
        None => match daemon::spawn_daemon(&daemon_dir) {
            Ok(info) => {
                println!("Daemon started (PID: {})", info.pid);
            }
            Err(e) => {
                return Err(Error::Daemon(format!("failed to start daemon: {}", e)));
            }
        },
    }

    Ok(())
}

/// View daemon logs.
pub fn logs(follow: bool) -> Result<()> {
    let daemon_dir = wok_state_dir();
    let log_path = daemon_dir.join("daemon.log");

    if !log_path.exists() {
        println!("No daemon logs found at {}", log_path.display());
        return Ok(());
    }

    if follow {
        // Tail -f the log file
        let status = std::process::Command::new("tail")
            .arg("-f")
            .arg(&log_path)
            .status()?;

        if !status.success() {
            return Err(Error::Io(std::io::Error::other("tail command failed")));
        }
    } else {
        // Read and print the log file
        let content = std::fs::read_to_string(&log_path)?;
        print!("{}", content);
    }

    Ok(())
}
