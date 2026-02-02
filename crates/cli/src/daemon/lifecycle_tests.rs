// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for daemon lifecycle management.

#![allow(clippy::unwrap_used)]

use tempfile::tempdir;

use super::lifecycle::*;

#[test]
fn test_get_socket_path() {
    let dir = tempdir().unwrap();
    let socket_path = get_socket_path(dir.path());
    assert!(socket_path.ends_with("daemon.sock"));
}

#[test]
fn test_get_pid_path() {
    let dir = tempdir().unwrap();
    let pid_path = get_pid_path(dir.path());
    assert!(pid_path.ends_with("daemon.pid"));
}

#[test]
fn test_get_lock_path() {
    let dir = tempdir().unwrap();
    let lock_path = get_lock_path(dir.path());
    assert!(lock_path.ends_with("daemon.lock"));
}

#[test]
fn test_detect_daemon_no_socket() {
    let dir = tempdir().unwrap();
    let result = detect_daemon(dir.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_daemon_info_fields() {
    let info = DaemonInfo { pid: 12345 };
    assert_eq!(info.pid, 12345);
}

// Crash recovery tests

#[test]
fn test_detect_daemon_cleans_up_stale_socket() {
    let dir = tempdir().unwrap();
    let socket_path = get_socket_path(dir.path());

    // Create a stale socket file (not a real socket)
    std::fs::write(&socket_path, "stale").unwrap();
    assert!(socket_path.exists());

    // detect_daemon should clean up the stale socket
    let result = detect_daemon(dir.path()).unwrap();
    assert!(result.is_none());

    // Socket should be cleaned up
    assert!(!socket_path.exists());
}

#[test]
fn test_detect_daemon_cleans_up_stale_pid() {
    let dir = tempdir().unwrap();
    let pid_path = get_pid_path(dir.path());

    // Create a stale PID file without a socket
    std::fs::write(&pid_path, "12345").unwrap();
    assert!(pid_path.exists());

    // detect_daemon should clean up the stale PID
    let result = detect_daemon(dir.path()).unwrap();
    assert!(result.is_none());

    // PID file should be cleaned up
    assert!(!pid_path.exists());
}

#[test]
fn test_detect_daemon_cleans_up_both_stale_files() {
    let dir = tempdir().unwrap();
    let socket_path = get_socket_path(dir.path());
    let pid_path = get_pid_path(dir.path());

    // Create stale files
    std::fs::write(&socket_path, "stale").unwrap();
    std::fs::write(&pid_path, "12345").unwrap();
    assert!(socket_path.exists());
    assert!(pid_path.exists());

    // detect_daemon should clean up both
    let result = detect_daemon(dir.path()).unwrap();
    assert!(result.is_none());

    // Both should be cleaned up
    assert!(!socket_path.exists());
    assert!(!pid_path.exists());
}

// Spawn race tests

#[test]
fn test_lock_path_unique_per_directory() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    let lock1 = get_lock_path(dir1.path());
    let lock2 = get_lock_path(dir2.path());

    // Lock paths should be different for different directories
    assert_ne!(lock1, lock2);
}

#[test]
fn test_get_daemon_status_not_running() {
    let dir = tempdir().unwrap();

    // Getting status when no daemon is running should return None
    let result = get_daemon_status(dir.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_cli_version_is_set() {
    // CLI_VERSION should be set from Cargo.toml
    assert!(!CLI_VERSION.is_empty());
    // Should be a valid semver-like string
    let parts: Vec<&str> = CLI_VERSION.split('.').collect();
    assert!(!parts.is_empty());
}

#[test]
fn test_stop_daemon_forcefully_no_daemon() {
    let dir = tempdir().unwrap();

    // Should succeed even when no daemon is running
    let result = stop_daemon_forcefully(dir.path());
    assert!(result.is_ok());
}

#[test]
fn test_stop_daemon_forcefully_cleans_stale_files() {
    let dir = tempdir().unwrap();
    let socket_path = get_socket_path(dir.path());
    let pid_path = get_pid_path(dir.path());

    // Create stale files
    std::fs::write(&socket_path, "stale").unwrap();
    std::fs::write(&pid_path, "12345").unwrap();

    // Should clean up stale files
    let result = stop_daemon_forcefully(dir.path());
    assert!(result.is_ok());
    assert!(!socket_path.exists());
    assert!(!pid_path.exists());
}
