// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Tests for remote sync management commands.
//!
//! These tests cover the `status()`, `stop()`, and `sync()` functions
//! in `remote.rs`, testing both local and remote mode behaviors.

use std::fs;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;

use tempfile::TempDir;

use super::*;
use crate::config::{Config, RemoteConfig};
use crate::daemon::{
    framing, get_pid_path, get_socket_path, DaemonRequest, DaemonResponse, DaemonStatus,
};

// Global lock for tests that change the current directory.
// This prevents test parallelism from causing issues.
static CWD_LOCK: Mutex<()> = Mutex::new(());

/// Helper to create a local-mode config (no remote section).
fn create_local_config(work_dir: &Path) {
    let config = Config::new("test".to_string()).unwrap();
    config.save(work_dir).unwrap();
}

/// Helper to create a remote-mode config.
fn create_remote_config(work_dir: &Path) {
    let config = Config {
        prefix: "test".to_string(),
        workspace: None,
        remote: Some(RemoteConfig {
            url: "ws://localhost:7890".to_string(),
            branch: "wk/oplog".to_string(),
            worktree: None,
            reconnect_max_retries: 10,
            reconnect_max_delay_secs: 30,
            heartbeat_interval_ms: 30_000,
            heartbeat_timeout_ms: 10_000,
            connect_timeout_secs: 2,
        }),
    };
    config.save(work_dir).unwrap();
}

/// Helper to set up a test directory with .wok subdirectory.
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();
    (temp, work_dir)
}

/// Run a test with the current directory set to the temp directory.
/// Uses a lock to prevent parallel test interference.
fn with_cwd<F, R>(temp_path: &Path, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _lock = CWD_LOCK.lock().unwrap();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();
    let result = f();
    std::env::set_current_dir(original_cwd).unwrap();
    result
}

#[test]
fn test_status_local_mode() {
    let (temp, work_dir) = setup_test_env();
    create_local_config(&work_dir);

    with_cwd(temp.path(), || {
        // Should not error, just print informational message
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_stop_local_mode() {
    let (temp, work_dir) = setup_test_env();
    create_local_config(&work_dir);

    with_cwd(temp.path(), || {
        let result = stop();
        assert!(result.is_ok());
    });
}

#[test]
fn test_sync_local_mode() {
    let (temp, work_dir) = setup_test_env();
    create_local_config(&work_dir);

    with_cwd(temp.path(), || {
        let result = sync(false, false);
        assert!(result.is_ok());
    });
}

#[test]
fn test_sync_local_mode_quiet() {
    let (temp, work_dir) = setup_test_env();
    create_local_config(&work_dir);

    with_cwd(temp.path(), || {
        // With quiet=true, should silently succeed without printing instructions
        let result = sync(false, true);
        assert!(result.is_ok());
    });
}

#[test]
fn test_status_remote_no_daemon() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    // No socket file exists - daemon not running
    with_cwd(temp.path(), || {
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_stop_remote_no_daemon() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    // No socket file exists - daemon not running
    with_cwd(temp.path(), || {
        let result = stop();
        assert!(result.is_ok());
    });
}

#[test]
fn test_sync_remote_no_daemon() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    // No socket file exists - will try to spawn daemon (which will fail, but that's ok)
    with_cwd(temp.path(), || {
        let result = sync(false, false);
        // The result should be Ok because sync() handles spawn errors gracefully
        assert!(result.is_ok());
    });
}

#[test]
fn test_status_remote_stale_socket() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    // Create a stale socket file (not connected to anything)
    let socket_path = get_socket_path(&work_dir);
    fs::write(&socket_path, "stale").unwrap();

    with_cwd(temp.path(), || {
        // This should handle the connection error gracefully
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_stop_remote_stale_socket() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    // Create a stale socket file
    let socket_path = get_socket_path(&work_dir);
    fs::write(&socket_path, "stale").unwrap();

    with_cwd(temp.path(), || {
        // detect_daemon will fail to connect and return None
        let result = stop();
        assert!(result.is_ok());
    });
}

#[test]
fn test_sync_remote_stale_socket() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    // Create a stale socket file
    let socket_path = get_socket_path(&work_dir);
    fs::write(&socket_path, "stale").unwrap();

    with_cwd(temp.path(), || {
        // detect_daemon will fail to connect and return None, then spawn will be attempted
        let result = sync(false, false);
        assert!(result.is_ok());
    });
}

/// Helper to create mock daemon that accepts multiple connections.
fn start_mock_daemon_multi(
    work_dir: &Path,
    status_response: DaemonStatus,
    connection_count: usize,
) -> thread::JoinHandle<()> {
    let socket_path = get_socket_path(work_dir);
    let pid_path = get_pid_path(work_dir);

    // Remove existing socket if present
    let _ = fs::remove_file(&socket_path);

    // Create socket listener
    let listener = UnixListener::bind(&socket_path).unwrap();

    // Write PID file
    fs::write(&pid_path, "12345").unwrap();

    let status_response_clone = status_response;
    thread::spawn(move || {
        for _ in 0..connection_count {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));

                if let Ok(request) = framing::read_request(&mut stream) {
                    let response = match request {
                        DaemonRequest::Ping => DaemonResponse::Pong,
                        DaemonRequest::Status => {
                            DaemonResponse::Status(status_response_clone.clone())
                        }
                        DaemonRequest::Shutdown => DaemonResponse::ShuttingDown,
                        DaemonRequest::SyncNow => DaemonResponse::SyncComplete { ops_synced: 0 },
                        DaemonRequest::Hello { .. } => DaemonResponse::Hello {
                            version: env!("CARGO_PKG_VERSION").to_string(),
                        },
                    };
                    let _ = framing::write_response(&mut stream, &response);
                }
            }
        }
    })
}

#[test]
fn test_status_remote_connected() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let status_response = DaemonStatus::new(
        true,  // connected
        false, // connecting
        "ws://localhost:7890".to_string(),
        5,                // pending_ops
        Some(1700000000), // last_sync timestamp
        12345,            // pid
        3600,             // uptime_secs
    );

    let _handle = start_mock_daemon_multi(&work_dir, status_response, 1);

    // Give the mock daemon time to start
    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_status_remote_disconnected() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let status_response = DaemonStatus::new(
        false, // disconnected
        false, // connecting
        "ws://localhost:7890".to_string(),
        0,
        None, // no last sync
        12345,
        100,
    );

    let _handle = start_mock_daemon_multi(&work_dir, status_response, 1);

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_status_last_sync_never() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let status_response = DaemonStatus::new(
        true,
        false, // connecting
        "ws://localhost:7890".to_string(),
        0,
        None, // last_sync = None triggers "never" branch
        12345,
        100,
    );

    let _handle = start_mock_daemon_multi(&work_dir, status_response, 1);

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_status_last_sync_timestamp_overflow() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    // Use a timestamp that causes DateTime::from_timestamp to return None
    // This happens when the timestamp is out of range
    let status_response = DaemonStatus::new(
        true,
        false, // connecting
        "ws://localhost:7890".to_string(),
        0,
        Some(u64::MAX), // Very large timestamp
        12345,
        100,
    );

    let _handle = start_mock_daemon_multi(&work_dir, status_response, 1);

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_stop_remote_success() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let status_response = DaemonStatus::new(
        true,
        false,
        "ws://localhost:7890".to_string(),
        0,
        None,
        12345,
        100,
    );

    // Need 2 connections: one for detect_daemon (ping) and one for stop_daemon (shutdown)
    let _handle = start_mock_daemon_multi(&work_dir, status_response, 2);

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        let result = stop();
        assert!(result.is_ok());
    });
}

#[test]
fn test_sync_remote_with_running_daemon() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let status_response = DaemonStatus::new(
        true,
        false,
        "ws://localhost:7890".to_string(),
        0,
        None,
        12345,
        100,
    );

    // Need 2 connections: one for detect_daemon (ping) and one for request_sync (SyncNow)
    let _handle = start_mock_daemon_multi(&work_dir, status_response, 2);

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        let result = sync(false, false);
        assert!(result.is_ok());
    });
}

/// Helper to start a mock daemon that returns error responses.
fn start_mock_daemon_error(work_dir: &Path, error_message: &str) -> thread::JoinHandle<()> {
    let socket_path = get_socket_path(work_dir);
    let pid_path = get_pid_path(work_dir);

    let _ = fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).unwrap();
    fs::write(&pid_path, "12345").unwrap();

    let error_msg = error_message.to_string();
    thread::spawn(move || {
        // Handle multiple connections for detect + actual command
        for _ in 0..2 {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));

                if let Ok(request) = framing::read_request(&mut stream) {
                    let response = match request {
                        DaemonRequest::Ping => DaemonResponse::Pong,
                        DaemonRequest::Hello { .. } => DaemonResponse::Hello {
                            version: env!("CARGO_PKG_VERSION").to_string(),
                        },
                        _ => DaemonResponse::Error {
                            message: error_msg.clone(),
                        },
                    };
                    let _ = framing::write_response(&mut stream, &response);
                }
            }
        }
    })
}

#[test]
fn test_status_remote_error_response() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let _handle = start_mock_daemon_error(&work_dir, "internal error");

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        // get_daemon_status returns Err, status() handles this in the Err branch
        let result = status();
        assert!(result.is_ok());
    });
}

#[test]
fn test_stop_remote_error_response() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let _handle = start_mock_daemon_error(&work_dir, "cannot stop");

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        // stop_daemon returns Err, stop() prints error but returns Ok
        let result = stop();
        assert!(result.is_ok());
    });
}

#[test]
fn test_sync_remote_error_response() {
    let (temp, work_dir) = setup_test_env();
    create_remote_config(&work_dir);

    let _handle = start_mock_daemon_error(&work_dir, "sync failed");

    thread::sleep(std::time::Duration::from_millis(50));

    with_cwd(temp.path(), || {
        // request_sync returns Err, sync() prints error but returns Ok
        let result = sync(false, false);
        assert!(result.is_ok());
    });
}
