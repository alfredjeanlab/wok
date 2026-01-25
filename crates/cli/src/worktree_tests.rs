// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_resolve_oplog_path_same_repo() {
    use std::process::Command;

    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    // Initialize a git repository
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let remote = RemoteConfig {
        url: "git:.".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };

    // For same-repo mode, path should be .git/wk/oplog
    let path = resolve_oplog_path(&work_dir, &remote).unwrap();
    assert_eq!(path, temp.path().join(".git").join("wk").join("oplog"));
}

#[test]
fn test_resolve_oplog_path_separate_repo_local_worktree() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    let remote = RemoteConfig {
        url: "git:~/tracker".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: Some(true), // Force local worktree for separate repos
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };

    let path = resolve_oplog_path(&work_dir, &remote).unwrap();
    assert_eq!(path, work_dir.join("oplog"));
}

#[test]
fn test_resolve_oplog_path_websocket_error() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    let remote = RemoteConfig {
        url: "ws://localhost:8080".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };

    let result = resolve_oplog_path(&work_dir, &remote);
    assert!(result.is_err());
}

#[test]
fn test_compute_repo_hash_deterministic() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    let hash1 = compute_repo_hash(&work_dir).unwrap();
    let hash2 = compute_repo_hash(&work_dir).unwrap();

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 16); // 8 bytes = 16 hex chars
}

#[test]
fn test_remote_config_is_same_repo() {
    let remote_dot = RemoteConfig {
        url: "git:.".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(remote_dot.is_same_repo());

    let remote_bare_dot = RemoteConfig {
        url: ".".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(remote_bare_dot.is_same_repo());

    let remote_path = RemoteConfig {
        url: "git:~/tracker".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(!remote_path.is_same_repo());
}

#[test]
fn test_remote_config_git_url() {
    let remote_same = RemoteConfig {
        url: "git:.".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert_eq!(remote_same.git_url(), None);

    let remote_path = RemoteConfig {
        url: "git:~/tracker".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert_eq!(remote_path.git_url(), Some("~/tracker"));

    let remote_ssh = RemoteConfig {
        url: "git@github.com:org/repo.git".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert_eq!(remote_ssh.git_url(), Some("git@github.com:org/repo.git"));
}

#[test]
fn test_remote_config_remote_type() {
    let git_remote = RemoteConfig {
        url: "git:.".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert_eq!(git_remote.remote_type(), RemoteType::Git);

    let ws_remote = RemoteConfig {
        url: "ws://localhost:8080".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert_eq!(ws_remote.remote_type(), RemoteType::WebSocket);

    let wss_remote = RemoteConfig {
        url: "wss://example.com/workspace".to_string(),
        branch: "wok/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert_eq!(wss_remote.remote_type(), RemoteType::WebSocket);
}
