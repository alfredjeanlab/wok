// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Daemon lifecycle management: spawn, detect, cleanup.
//!
//! The daemon is spawned as a background process and communicates via Unix socket.
//! PID and socket files are stored in the daemon directory (same directory as the database).

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
                            // or crashed. Return None to trigger retry.
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

/// Wait for the daemon to establish a connection to the remote server.
///
/// Polls the daemon status until connected, disconnected (gave up), or timeout.
/// Returns Ok(true) if connected, Ok(false) if timed out or disconnected.
pub fn wait_daemon_connected(daemon_dir: &Path, timeout: Duration) -> Result<bool> {
    use std::time::Instant;

    let start = Instant::now();
    let poll_interval = Duration::from_millis(50);

    while start.elapsed() < timeout {
        if let Ok(Some(status)) = get_daemon_status(daemon_dir) {
            if status.connected {
                return Ok(true);
            }
            // If not connected and not connecting, daemon has given up - stop waiting
            if !status.connecting {
                return Ok(false);
            }
        }
        std::thread::sleep(poll_interval);
    }

    Ok(false)
}

/// Send a shutdown request to the daemon.
pub fn stop_daemon(daemon_dir: &Path) -> Result<()> {
    let socket_path = get_socket_path(daemon_dir);

    if !socket_path.exists() {
        return Err(Error::Io(std::io::Error::other(
            "daemon is not running".to_string(),
        )));
    }

    let mut stream = UnixStream::connect(&socket_path)?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    framing::write_request(&mut stream, &DaemonRequest::Shutdown)?;

    match framing::read_response(&mut stream)? {
        DaemonResponse::ShuttingDown => Ok(()),
        DaemonResponse::Error { message } => Err(Error::Io(std::io::Error::other(message))),
        _ => Err(Error::Io(std::io::Error::other(
            "unexpected response".to_string(),
        ))),
    }
}

/// Notify daemon that queue has changed (fire-and-forget).
/// Best-effort: silently ignores all errors.
pub fn notify_daemon_sync(daemon_dir: &Path) {
    let socket_path = get_socket_path(daemon_dir);
    if !socket_path.exists() {
        return;
    }
    // Non-blocking connect, send SyncNow, ignore response
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
        let _ = framing::write_request(&mut stream, &DaemonRequest::SyncNow);
    }
}

/// Request immediate sync from the daemon.
pub fn request_sync(daemon_dir: &Path) -> Result<usize> {
    let socket_path = get_socket_path(daemon_dir);

    if !socket_path.exists() {
        return Err(Error::Io(std::io::Error::other(
            "daemon is not running".to_string(),
        )));
    }

    let mut stream = UnixStream::connect(&socket_path)?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    framing::write_request(&mut stream, &DaemonRequest::SyncNow)?;

    match framing::read_response(&mut stream)? {
        DaemonResponse::SyncComplete { ops_synced } => Ok(ops_synced),
        DaemonResponse::Error { message } => Err(Error::Io(std::io::Error::other(message))),
        _ => Err(Error::Io(std::io::Error::other(
            "unexpected response".to_string(),
        ))),
    }
}

/// Spawn a new daemon process for the given daemon directory.
///
/// Returns the DaemonInfo for the spawned daemon.
/// Uses flock to ensure only one daemon instance per daemon directory.
///
/// # Arguments
/// * `daemon_dir` - Directory for daemon files (socket, pid, lock)
/// * `work_dir` - Work directory for loading config (.work directory)
pub fn spawn_daemon(daemon_dir: &Path, work_dir: &Path) -> Result<DaemonInfo> {
    // Check if daemon is already running
    if let Some(info) = detect_daemon(daemon_dir)? {
        return Ok(info);
    }

    // Get the path to the current executable
    let exe_path = std::env::current_exe()?;

    // Spawn daemon process
    // The daemon will:
    // 1. Acquire flock on lock file
    // 2. Write PID file
    // 3. Create Unix socket
    // 4. Start main loop
    let mut child = Command::new(&exe_path)
        .arg("remote")
        .arg("run")
        .arg("--daemon-dir")
        .arg(daemon_dir)
        .arg("--work-dir")
        .arg(work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

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
    // Use 10ms intervals with up to 150 attempts (1.5s total timeout)
    for _ in 0..150 {
        // Check if daemon process has exited (indicates failure)
        if let Ok(Some(status)) = child.try_wait() {
            // Read stderr for error message
            let stderr_output = if let Some(mut stderr) = child.stderr.take() {
                use std::io::Read;
                let mut output = String::new();
                let _ = stderr.read_to_string(&mut output);
                output
            } else {
                String::new()
            };
            return Err(Error::Io(std::io::Error::other(format!(
                "daemon process exited with status: {}\nstderr: {}",
                status, stderr_output
            ))));
        }

        if let Some(info) = detect_daemon(daemon_dir)? {
            return Ok(info);
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    // Daemon failed to start - return error instead of Ok with invalid pid
    Err(Error::Io(std::io::Error::other(
        "daemon failed to start: could not connect after multiple attempts".to_string(),
    )))
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
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result of version handshake with daemon.
#[derive(Debug)]
pub enum HandshakeResult {
    /// Versions match, daemon is compatible.
    Compatible,
    /// Daemon is older than CLI, should restart.
    DaemonOlder { daemon_version: String },
    /// CLI is older than daemon, warn but continue.
    CliOlder { daemon_version: String },
    /// Daemon doesn't understand Hello (old protocol).
    OldProtocol,
    /// Connection or communication failed.
    Failed(String),
}

/// Perform version handshake with the daemon.
///
/// Returns the handshake result indicating compatibility status.
pub fn handshake_daemon(daemon_dir: &Path) -> HandshakeResult {
    let socket_path = get_socket_path(daemon_dir);

    if !socket_path.exists() {
        return HandshakeResult::Failed("socket does not exist".to_string());
    }

    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(e) => return HandshakeResult::Failed(format!("connect failed: {}", e)),
    };

    // Set short timeout for handshake
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    // Send Hello request
    let request = DaemonRequest::Hello {
        version: CLI_VERSION.to_string(),
    };
    if let Err(e) = framing::write_request(&mut stream, &request) {
        return HandshakeResult::Failed(format!("write failed: {}", e));
    }

    // Read response
    match framing::read_response(&mut stream) {
        Ok(DaemonResponse::Hello { version }) => {
            // Compare versions
            match compare_versions(CLI_VERSION, &version) {
                std::cmp::Ordering::Equal => HandshakeResult::Compatible,
                std::cmp::Ordering::Greater => HandshakeResult::DaemonOlder {
                    daemon_version: version,
                },
                std::cmp::Ordering::Less => HandshakeResult::CliOlder {
                    daemon_version: version,
                },
            }
        }
        Ok(DaemonResponse::Error { message }) => {
            // Old daemon returns Error for unknown request types
            if message.contains("unknown") || message.contains("deserialize") {
                HandshakeResult::OldProtocol
            } else {
                HandshakeResult::Failed(message)
            }
        }
        Ok(_) => {
            // Unexpected response type - likely old daemon
            HandshakeResult::OldProtocol
        }
        Err(e) => {
            let err_str = e.to_string();
            // Check if it's a deserialization error (old daemon can't parse Hello)
            if err_str.contains("deserialize") {
                HandshakeResult::OldProtocol
            } else {
                HandshakeResult::Failed(err_str)
            }
        }
    }
}

/// Compare two semver-like version strings.
///
/// Returns Ordering based on version comparison.
fn compare_versions(v1: &str, v2: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|p| p.parse().ok()).collect() };

    let parts1 = parse(v1);
    let parts2 = parse(v2);

    for i in 0..3 {
        let p1 = parts1.get(i).copied().unwrap_or(0);
        let p2 = parts2.get(i).copied().unwrap_or(0);
        match p1.cmp(&p2) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    std::cmp::Ordering::Equal
}

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
                wait_for_process_exit(pid, Duration::from_secs(3));
            }
            cleanup_stale_files(daemon_dir);
            return Ok(());
        }
        Err(_) => {
            // Graceful shutdown failed, try SIGKILL
        }
    }

    // If we have a PID, send SIGKILL via external kill command
    if let Some(pid) = pid {
        let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();
        // Give kernel time to clean up
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
        // Check if process still exists by sending signal 0 via kill command
        let result = Command::new("kill").arg("-0").arg(pid.to_string()).output();

        match result {
            Ok(output) if !output.status.success() => {
                // Process doesn't exist (kill -0 failed)
                return;
            }
            Err(_) => {
                // Command failed, assume process doesn't exist
                return;
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Ensure a compatible daemon is running, restarting if necessary.
///
/// This function:
/// 1. Checks if a daemon is running
/// 2. If running, performs version handshake
/// 3. If version mismatch or old protocol, restarts daemon
/// 4. If no daemon running, spawns one
///
/// Returns DaemonInfo for the running (compatible) daemon.
pub fn ensure_compatible_daemon(daemon_dir: &Path, work_dir: &Path) -> Result<DaemonInfo> {
    // First check if any daemon is running
    if detect_daemon(daemon_dir)?.is_none() {
        // No daemon running, just spawn one
        return spawn_daemon(daemon_dir, work_dir);
    }

    // Daemon is running, check version
    match handshake_daemon(daemon_dir) {
        HandshakeResult::Compatible => {
            // Version matches, use existing daemon
            detect_daemon(daemon_dir)?
                .ok_or_else(|| Error::Io(std::io::Error::other("daemon disappeared")))
        }
        HandshakeResult::DaemonOlder { daemon_version } => {
            // Daemon is older, restart it
            eprintln!(
                "Restarting daemon: version mismatch (daemon v{}, CLI v{})",
                daemon_version, CLI_VERSION
            );
            stop_daemon_forcefully(daemon_dir)?;
            spawn_daemon(daemon_dir, work_dir)
        }
        HandshakeResult::OldProtocol => {
            // Old daemon doesn't understand Hello, restart it
            eprintln!("Restarting daemon: old protocol version detected");
            stop_daemon_forcefully(daemon_dir)?;
            spawn_daemon(daemon_dir, work_dir)
        }
        HandshakeResult::CliOlder { daemon_version } => {
            // CLI is older than daemon - warn but continue
            eprintln!(
                "Warning: daemon v{} is newer than CLI v{}",
                daemon_version, CLI_VERSION
            );
            detect_daemon(daemon_dir)?
                .ok_or_else(|| Error::Io(std::io::Error::other("daemon disappeared")))
        }
        HandshakeResult::Failed(msg) => {
            // Communication failed, try to restart
            eprintln!("Daemon communication failed ({}), restarting", msg);
            stop_daemon_forcefully(daemon_dir)?;
            spawn_daemon(daemon_dir, work_dir)
        }
    }
}
