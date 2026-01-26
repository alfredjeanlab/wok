// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! End-to-end integration tests for WebSocket transport.
//!
//! These tests exercise `WebSocketTransport` against a real `wk-remote` server,
//! validating the full client-server communication path.
//!
//! # Requirements
//!
//! The `wk-remote` binary must be built and available in the same target directory.
//! Run `cargo build -p wk-remote` before running these tests.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use wk_core::protocol::{ClientMessage, ServerMessage};
use wk_core::{Hlc, Op, OpPayload};
use wkrs::sync::{SyncClient, SyncConfig, Transport, WebSocketTransport};

/// Helper macro to skip tests when wk-remote binary is not available.
/// Prints a message and returns early instead of failing.
macro_rules! require_server {
    () => {
        match TestServer::spawn() {
            Some(server) => server,
            None => {
                eprintln!(
                    "SKIPPED: wk-remote binary not found. Run `cargo build -p wk-remote` first."
                );
                return;
            }
        }
    };
}

/// Returns timeout duration, longer for CI environments.
fn timeout() -> Duration {
    if std::env::var("CI").is_ok() {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(5)
    }
}

/// Find the wk-remote binary path.
///
/// Searches in order:
/// 1. WK_REMOTE_BIN environment variable
/// 2. Sibling package target directory (../remote/target/{debug,release}/wk-remote)
/// 3. Same target directory (for workspace builds)
///
/// Returns `None` if the binary is not found, allowing tests to skip gracefully.
fn find_wk_remote() -> Option<PathBuf> {
    let binary_name = if cfg!(windows) {
        "wk-remote.exe"
    } else {
        "wk-remote"
    };

    // Check environment variable first
    if let Ok(path) = std::env::var("WK_REMOTE_BIN") {
        let binary_path = PathBuf::from(path);
        if binary_path.exists() {
            return Some(binary_path);
        }
    }

    // Get the path to the current test executable
    let test_exe = std::env::current_exe().expect("cannot get current exe path");

    // Try sibling package target directory
    // test_exe is at: bin/cli/target/{debug,release}/deps/transport-*
    // wk-remote is at: bin/remote/target/{debug,release}/wk-remote
    if let Some(deps_dir) = test_exe.parent() {
        if let Some(profile_dir) = deps_dir.parent() {
            let profile_name = profile_dir.file_name().and_then(|s| s.to_str());
            if let Some(profile) = profile_name {
                // Navigate to sibling remote package
                if let Some(target_dir) = profile_dir.parent() {
                    if let Some(cli_dir) = target_dir.parent() {
                        if let Some(bin_dir) = cli_dir.parent() {
                            let sibling_path = bin_dir
                                .join("remote")
                                .join("target")
                                .join(profile)
                                .join(binary_name);
                            if sibling_path.exists() {
                                return Some(sibling_path);
                            }
                        }
                    }
                }
            }
        }
    }

    // For workspace builds, test_exe is at: wok/target/{debug,release}/deps/transport-*
    // wk-remote is at: wok/target/{debug,release}/wk-remote
    if let Some(deps_dir) = test_exe.parent() {
        if let Some(profile_dir) = deps_dir.parent() {
            let binary_path = profile_dir.join(binary_name);
            if binary_path.exists() {
                return Some(binary_path);
            }
        }
    }

    None
}

/// Helper to spawn `wk-remote` server and clean up on drop.
struct TestServer {
    child: Child,
    port: u16,
    _data_dir: TempDir,
}

impl TestServer {
    /// Spawn a new test server, trying multiple ports if needed.
    /// Returns `None` if wk-remote binary is not found (test should be skipped).
    fn spawn() -> Option<Self> {
        let binary = find_wk_remote()?;

        for attempt in 0..5 {
            let port = Self::random_port(attempt);

            // Check port is available before spawning
            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(listener) => {
                    drop(listener); // Release for server to use

                    if let Ok(server) = Self::try_spawn(&binary, port) {
                        return Some(server);
                    }
                    // Spawn failed, try next port
                }
                Err(_) => {
                    // Port in use, try next
                }
            }
        }
        panic!("wk-remote binary found but failed to start server after 5 port attempts");
    }

    /// Attempt to spawn server on a specific port.
    fn try_spawn(binary: &PathBuf, port: u16) -> Result<Self, std::io::Error> {
        let data_dir = TempDir::new()?;

        let child = Command::new(binary)
            .arg("--bind")
            .arg(format!("127.0.0.1:{}", port))
            .arg("--data")
            .arg(data_dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(TestServer {
            child,
            port,
            _data_dir: data_dir,
        })
    }

    /// Generate a random port in the ephemeral range.
    fn random_port(attempt: u32) -> u16 {
        let mut hasher = DefaultHasher::new();
        std::process::id().hash(&mut hasher);
        std::thread::current().id().hash(&mut hasher);
        Instant::now().hash(&mut hasher);
        attempt.hash(&mut hasher);

        // Ephemeral port range: 49152-65535
        49152 + (hasher.finish() % 16383) as u16
    }

    /// Get the WebSocket URL for this server.
    fn url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// Wait for server to be ready, with retries.
    async fn wait_ready(&self) -> Result<(), &'static str> {
        for _ in 0..50 {
            match tokio::net::TcpStream::connect(("127.0.0.1", self.port)).await {
                Ok(_) => return Ok(()),
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
        Err("server did not become ready")
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Kill the process
        let _ = self.child.kill();
        // Wait to reap zombie (prevents resource leak)
        let _ = self.child.wait();
        // TempDir cleans itself up via its own Drop
    }
}

#[tokio::test]
async fn test_websocket_transport_connect_disconnect() {
    let server = require_server!();
    server.wait_ready().await.unwrap();

    let mut transport = WebSocketTransport::new();
    assert!(!transport.is_connected());

    transport.connect(&server.url()).await.unwrap();
    assert!(transport.is_connected());

    transport.disconnect().await.unwrap();
    assert!(!transport.is_connected());
}

#[tokio::test]
async fn test_websocket_transport_ping_pong() {
    let server = require_server!();
    server.wait_ready().await.unwrap();

    let mut transport = WebSocketTransport::new();
    transport.connect(&server.url()).await.unwrap();

    // Send ping
    transport.send(ClientMessage::ping(42)).await.unwrap();

    // Receive pong with timeout
    let msg = tokio::time::timeout(timeout(), transport.recv())
        .await
        .expect("recv timed out")
        .unwrap();

    assert!(matches!(msg, Some(ServerMessage::Pong { id: 42 })));

    transport.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_sync_client_with_real_transport() {
    let server = require_server!();
    server.wait_ready().await.unwrap();

    let client_dir = TempDir::new().unwrap();
    let queue_path = client_dir.path().join("queue.jsonl");

    let config = SyncConfig {
        url: server.url(),
        max_retries: 3,
        max_delay_secs: 1,
        initial_delay_ms: 100,
        heartbeat_interval_ms: 0,
        heartbeat_timeout_ms: 0,
    };
    let transport = WebSocketTransport::new();
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    // Connect
    client.connect().await.unwrap();
    assert!(client.is_connected());

    // Request snapshot (start with fresh state, before sending any ops)
    client.request_snapshot().await.unwrap();

    // Receive snapshot response with timeout
    let msg = tokio::time::timeout(timeout(), client.recv())
        .await
        .expect("recv timed out")
        .unwrap();

    assert!(matches!(msg, Some(ServerMessage::SnapshotResponse { .. })));

    // Now send an operation
    let op = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue(
            "test-001".to_string(),
            wk_core::IssueType::Task,
            "Test issue".to_string(),
        ),
    );
    client.send_op(op.clone()).await.unwrap();

    // Server broadcasts the op back to all clients (including sender)
    let msg = tokio::time::timeout(timeout(), client.recv())
        .await
        .expect("recv timed out")
        .unwrap();

    assert!(matches!(msg, Some(ServerMessage::Op(_))));

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_websocket_transport_connection_refused() {
    let mut transport = WebSocketTransport::new();

    // Connect to a port with nothing listening (port 1 is privileged and unused)
    let result = transport.connect("ws://127.0.0.1:1").await;

    assert!(result.is_err());
    assert!(!transport.is_connected());
}
