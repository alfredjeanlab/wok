// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Daemon lifecycle management: spawn, detect, cleanup.
//!
//! The daemon (wokd) is spawned as a background process and communicates via Unix socket.
//! PID and socket files are stored in the state directory (~/.local/state/wok/).

use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::{Error, Result};

use super::ipc::{framing, DaemonRequest, DaemonResponse, DaemonStatus};

/// Socket filename within daemon directory.
const SOCKET_NAME: &str = "daemon.sock";
/// PID filename within daemon directory.
const PID_NAME: &str = "daemon.pid";
/// Lock filename for single instance guarantee.
#[cfg(test)]
const LOCK_NAME: &str = "daemon.lock";

/// Information about a running daemon.
#[derive(Debug, Clone)]
pub struct DaemonInfo {
    /// Process ID of the daemon.
    pub pid: u32,
}

/// Get the socket path for the given daemon directory.
pub fn get_socket_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join(SOCKET_NAME)
}

/// Get the PID file path for the given daemon directory.
pub fn get_pid_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join(PID_NAME)
}

/// Get the lock file path for the given daemon directory.
#[cfg(test)]
pub fn get_lock_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join(LOCK_NAME)
}

/// Detect if a daemon is running for the given daemon directory.
///
/// Returns Some(DaemonInfo) if a daemon is running and responding,
/// None otherwise. Cleans up stale PID/socket files if found.
pub fn detect_daemon(daemon_dir: &Path) -> Result<Option<DaemonInfo>> {
    let socket_path = get_socket_path(daemon_dir);
    let pid_path = get_pid_path(daemon_dir);

    // Check if socket exists
    if !socket_path.exists() {
        // No socket, clean up stale PID file if it exists
        if pid_path.exists() {
            let _ = fs::remove_file(&pid_path);
        }
        return Ok(None);
    }

    // Try to connect and ping
    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            // Set a short timeout for the ping
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

            // Send ping request
            if framing::write_request(&mut stream, &DaemonRequest::Ping).is_err() {
                // Failed to write, daemon is dead
                cleanup_stale_files(daemon_dir);
                return Ok(None);
            }

            // Read response
            match framing::read_response(&mut stream) {
                Ok(DaemonResponse::Pong) => {
                    // Daemon is alive, read PID
                    match read_pid_file(&pid_path) {
                        Some(pid) if pid > 0 => Ok(Some(DaemonInfo { pid })),
                        _ => {
                            // PID file missing or invalid - daemon may be starting up
                            Ok(None)
                        }
                    }
                }
                _ => {
                    // Unexpected response or error
                    cleanup_stale_files(daemon_dir);
                    Ok(None)
                }
            }
        }
        Err(_) => {
            // Cannot connect, clean up stale files
            cleanup_stale_files(daemon_dir);
            Ok(None)
        }
    }
}

/// Get daemon status by connecting to the daemon.
pub fn get_daemon_status(daemon_dir: &Path) -> Result<Option<DaemonStatus>> {
    let socket_path = get_socket_path(daemon_dir);

    if !socket_path.exists() {
        return Ok(None);
    }

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

            framing::write_request(&mut stream, &DaemonRequest::Status)?;

            match framing::read_response(&mut stream)? {
                DaemonResponse::Status(status) => Ok(Some(status)),
                DaemonResponse::Error { message } => Err(Error::Io(std::io::Error::other(message))),
                _ => Err(Error::Io(std::io::Error::other(
                    "unexpected response".to_string(),
                ))),
            }
        }
        Err(e) => {
            // Cannot connect
            cleanup_stale_files(daemon_dir);
            Err(Error::Io(e))
        }
    }
}

/// Send a shutdown request to the daemon.
fn stop_daemon(daemon_dir: &Path) -> Result<()> {
    let socket_path = get_socket_path(daemon_dir);

    if !socket_path.exists() {
        return Err(Error::Io(std::io::Error::other(
            "daemon is not running".to_string(),
        )));
    }

    let mut stream = UnixStream::connect(&socket_path)?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    framing::write_request(&mut stream, &DaemonRequest::Shutdown)?;

    match framing::read_response(&mut stream)? {
        DaemonResponse::ShuttingDown => Ok(()),
        DaemonResponse::Error { message } => Err(Error::Io(std::io::Error::other(message))),
        _ => Err(Error::Io(std::io::Error::other(
            "unexpected response".to_string(),
        ))),
    }
}

/// Find the wokd binary.
fn find_wokd_binary() -> Result<PathBuf> {
    // 1. Check WOK_DAEMON_BINARY env var
    if let Ok(path) = std::env::var("WOK_DAEMON_BINARY") {
        return Ok(PathBuf::from(path));
    }

    // 2. Look next to the current executable
    if let Ok(exe) = std::env::current_exe() {
        let wokd = exe.with_file_name("wokd");
        if wokd.exists() {
            return Ok(wokd);
        }
    }

    // 3. Fall back to PATH
    Ok(PathBuf::from("wokd"))
}

/// Spawn a new daemon process for the given daemon directory.
///
/// Returns the DaemonInfo for the spawned daemon.
/// Uses flock to ensure only one daemon instance per daemon directory.
pub fn spawn_daemon(daemon_dir: &Path) -> Result<DaemonInfo> {
    // Check if daemon is already running
    if let Some(info) = detect_daemon(daemon_dir)? {
        return Ok(info);
    }

    // Ensure daemon directory exists
    fs::create_dir_all(daemon_dir)?;

    // Find wokd binary
    let wokd_path = find_wokd_binary()?;

    // Spawn daemon process
    let mut child = Command::new(&wokd_path)
        .arg("--state-dir")
        .arg(daemon_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            Error::Daemon(format!(
                "failed to start wokd ({}): {}",
                wokd_path.display(),
                e
            ))
        })?;

    // Wait for daemon to signal it's ready (writes "READY" to stdout)
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) if line == "READY" => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    }

    // Verify daemon is running with short polling
    for _ in 0..150 {
        // Check if daemon process has exited (indicates failure)
        if let Ok(Some(status)) = child.try_wait() {
            let stderr_output = if let Some(mut stderr) = child.stderr.take() {
                use std::io::Read;
                let mut output = String::new();
                let _ = stderr.read_to_string(&mut output);
                output
            } else {
                String::new()
            };
            return Err(Error::Daemon(format!(
                "daemon process exited with status: {}\n{}",
                status,
                stderr_output.trim()
            )));
        }

        if let Some(info) = detect_daemon(daemon_dir)? {
            return Ok(info);
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    Err(Error::Daemon(
        "daemon failed to start: could not connect after multiple attempts".to_string(),
    ))
}

/// Clean up stale socket and PID files.
fn cleanup_stale_files(daemon_dir: &Path) {
    let socket_path = get_socket_path(daemon_dir);
    let pid_path = get_pid_path(daemon_dir);

    let _ = fs::remove_file(&socket_path);
    let _ = fs::remove_file(&pid_path);
}

/// Read PID from the PID file.
fn read_pid_file(pid_path: &Path) -> Option<u32> {
    fs::read_to_string(pid_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// CLI version for handshake.
#[cfg(test)]
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stop the daemon forcefully if graceful shutdown fails.
///
/// Tries graceful shutdown first, then sends SIGKILL if needed.
pub fn stop_daemon_forcefully(daemon_dir: &Path) -> Result<()> {
    let pid_path = get_pid_path(daemon_dir);

    // Read PID before attempting shutdown
    let pid = read_pid_file(&pid_path);

    // Try graceful shutdown first
    match stop_daemon(daemon_dir) {
        Ok(()) => {
            // Wait for daemon to actually exit
            if let Some(pid) = pid {
                wait_for_process_exit(pid, Duration::from_secs(1));
            }
            cleanup_stale_files(daemon_dir);
            return Ok(());
        }
        Err(_) => {
            // Graceful shutdown failed, try SIGKILL
        }
    }

    // If we have a PID, send SIGKILL
    if let Some(pid) = pid {
        let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();
        std::thread::sleep(Duration::from_millis(100));
    }

    // Clean up stale files
    cleanup_stale_files(daemon_dir);

    Ok(())
}

/// Wait for a process to exit, with timeout.
fn wait_for_process_exit(pid: u32, timeout: Duration) {
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let result = Command::new("kill").arg("-0").arg(pid.to_string()).output();

        match result {
            Ok(output) if !output.status.success() => return,
            Err(_) => return,
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
