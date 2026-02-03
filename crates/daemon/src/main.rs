// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! wokd - The wok daemon.
//!
//! Manages a shared user-level SQLite database at `~/.local/state/wok/`.
//! Listens on a Unix socket for IPC from `wk` CLI processes.
//!
//! Usage:
//!   wokd --state-dir <path>

use std::fs;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod env;
mod ipc;

use ipc::{framing, DaemonRequest, DaemonResponse, DaemonStatus};

/// Socket filename within daemon directory.
const SOCKET_NAME: &str = "daemon.sock";
/// PID filename within daemon directory.
const PID_NAME: &str = "daemon.pid";
/// Lock filename for single instance guarantee.
const LOCK_NAME: &str = "daemon.lock";

fn main() {
    // Parse args
    let args: Vec<String> = std::env::args().collect();
    let state_dir = parse_state_dir(&args);

    // Set up logging
    let log_path = state_dir.join("daemon.log");
    setup_logging(&log_path);

    tracing::info!("wokd starting, state_dir={}", state_dir.display());

    // Acquire file lock for single instance
    let lock_path = state_dir.join(LOCK_NAME);
    let lock_file = match acquire_lock(&lock_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("failed to acquire lock: {}", e);
            std::process::exit(1);
        }
    };

    // Write PID file
    let pid_path = state_dir.join(PID_NAME);
    if let Err(e) = write_pid_file(&pid_path) {
        tracing::error!("failed to write PID file: {}", e);
        std::process::exit(1);
    }

    // Bind Unix socket
    let socket_path = state_dir.join(SOCKET_NAME);
    // Remove stale socket if it exists
    let _ = fs::remove_file(&socket_path);

    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("failed to bind socket: {}", e);
            cleanup(&pid_path, &socket_path);
            std::process::exit(1);
        }
    };

    tracing::info!("listening on {}", socket_path.display());

    // Signal readiness to parent process
    println!("READY");
    // Flush stdout so parent sees READY immediately
    let _ = std::io::stdout().flush();

    let start_time = Instant::now();

    // Accept connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));

                match framing::read_request(&mut stream) {
                    Ok(request) => {
                        let response = handle_request(request, &start_time);
                        let should_shutdown = matches!(response, DaemonResponse::ShuttingDown);
                        let _ = framing::write_response(&mut stream, &response);
                        if should_shutdown {
                            tracing::info!("shutting down");
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("failed to read request: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("failed to accept connection: {}", e);
            }
        }
    }

    // Cleanup
    cleanup(&pid_path, &socket_path);
    drop(lock_file);
    tracing::info!("wokd stopped");
}

fn handle_request(request: DaemonRequest, start_time: &Instant) -> DaemonResponse {
    match request {
        DaemonRequest::Ping => DaemonResponse::Pong,
        DaemonRequest::Status => {
            let pid = std::process::id();
            let uptime_secs = start_time.elapsed().as_secs();
            DaemonResponse::Status(DaemonStatus::new(pid, uptime_secs))
        }
        DaemonRequest::Shutdown => DaemonResponse::ShuttingDown,
        DaemonRequest::Hello { version: _ } => DaemonResponse::Hello {
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    }
}

fn parse_state_dir(args: &[String]) -> PathBuf {
    for i in 0..args.len() {
        if args[i] == "--state-dir" {
            if let Some(dir) = args.get(i + 1) {
                return PathBuf::from(dir);
            }
        }
    }
    // Default to XDG state directory
    if let Some(dir) = env::state_dir() {
        return dir;
    }
    if let Some(dir) = env::xdg_state_home() {
        return dir.join("wok");
    }
    dirs::home_dir()
        .map(|h| h.join(".local/state/wok"))
        .unwrap_or_else(|| PathBuf::from(".local/state/wok"))
}

fn setup_logging(log_path: &Path) {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Try to open log file, fall back to stderr
    if let Ok(file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(file)
            .with_ansi(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .init();
    }
}

fn acquire_lock(lock_path: &Path) -> std::io::Result<fs::File> {
    use fs2::FileExt;

    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(lock_path)?;
    file.try_lock_exclusive()
        .map_err(|_| std::io::Error::other("another daemon instance is already running"))?;
    Ok(file)
}

fn write_pid_file(pid_path: &Path) -> std::io::Result<()> {
    fs::write(pid_path, format!("{}", std::process::id()))
}

fn cleanup(pid_path: &Path, socket_path: &Path) {
    let _ = fs::remove_file(pid_path);
    let _ = fs::remove_file(socket_path);
}
