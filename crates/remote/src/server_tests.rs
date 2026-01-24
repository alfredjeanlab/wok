// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Test server utilities for integration testing.
//!
//! Provides a TestServer that runs on a random port with fault injection
//! capabilities for testing client behavior under various conditions.

#![cfg(test)]
#![allow(dead_code)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::server;
use crate::state::ServerState;

/// A test server that runs on a random port and can be controlled.
pub struct TestServer {
    addr: SocketAddr,
    shutdown_tx: oneshot::Sender<()>,
    state: ServerState,
    /// Keep the temp directory alive for the lifetime of the test server.
    #[allow(dead_code)]
    _temp_dir: tempfile::TempDir,
}

impl TestServer {
    /// Start a new test server on a random available port.
    pub async fn start() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let state = ServerState::new(temp_dir.path(), None).unwrap();

        // Bind to port 0 to get a random available port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let state_clone = state.clone();
        tokio::spawn(async move {
            tokio::select! {
                result = accept_loop(listener, state_clone) => {
                    if let Err(e) = result {
                        eprintln!("Test server error: {}", e);
                    }
                }
                _ = shutdown_rx => {
                    // Shutdown requested
                }
            }
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        TestServer {
            addr,
            shutdown_tx,
            state,
            _temp_dir: temp_dir,
        }
    }

    /// Get the address the server is listening on.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get the WebSocket URL for connecting to this server.
    pub fn ws_url(&self) -> String {
        format!("ws://{}", self.addr)
    }

    /// Get access to the server state for verification.
    pub fn state(&self) -> &ServerState {
        &self.state
    }

    /// Shutdown the test server.
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Accept loop that uses the actual server::handle_connection.
/// Note: spawns tasks which may not show coverage correctly with LLVM instrumentation.
async fn accept_loop(
    listener: TcpListener,
    state: ServerState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            let _ = server::handle_connection(stream, peer_addr, state).await;
        });
    }
}

/// Direct test helper that runs handle_connection in the same task for coverage.
/// Returns when the client disconnects.
async fn run_single_connection(
    listener: &TcpListener,
    state: ServerState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (stream, peer_addr) = listener.accept().await?;
    server::handle_connection(stream, peer_addr, state).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use wk_core::issue::IssueType;
    use wk_core::protocol::{ClientMessage, ServerMessage};
    use wk_core::{Hlc, Op, OpPayload};

    #[tokio::test]
    async fn test_server_starts() {
        let server = TestServer::start().await;
        assert!(server.addr().port() > 0);
        server.shutdown();
    }

    #[tokio::test]
    async fn test_ping_pong() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        let (ws_stream, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink, mut stream) = ws_stream.split();

        // Send ping
        let ping = ClientMessage::ping(42);
        sink.send(Message::Text(ping.to_json().unwrap()))
            .await
            .unwrap();

        // Receive pong (with timeout)
        let result = timeout(Duration::from_secs(5), stream.next()).await;
        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let response: ServerMessage = serde_json::from_str(&text).unwrap();
                assert!(matches!(response, ServerMessage::Pong { id: 42 }));
            }
            Ok(other) => panic!("Expected pong response, got {:?}", other),
            Err(_) => panic!("Timeout waiting for pong response"),
        }

        server.shutdown();
    }

    #[tokio::test]
    async fn test_op_broadcast() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        // Connect two clients
        let (ws1, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink1, mut stream1) = ws1.split();

        let (ws2, _) = connect_async(&server.ws_url()).await.unwrap();
        let (_sink2, mut stream2) = ws2.split();

        // Give connections time to establish
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Client 1 sends an op
        let op = Op::new(
            Hlc::new(1000, 0, 1),
            OpPayload::create_issue("test-1".into(), IssueType::Task, "Test Issue".into()),
        );
        let msg = ClientMessage::op(op.clone());
        sink1
            .send(Message::Text(msg.to_json().unwrap()))
            .await
            .unwrap();

        // Client 2 should receive the broadcast (with timeout)
        let result = timeout(Duration::from_secs(5), stream2.next()).await;
        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let response: ServerMessage = serde_json::from_str(&text).unwrap();
                if let ServerMessage::Op(received_op) = response {
                    assert_eq!(received_op.id, op.id);
                } else {
                    panic!("Expected Op broadcast, got {:?}", response);
                }
            }
            Ok(other) => panic!("Unexpected message: {:?}", other),
            Err(_) => panic!("Timeout waiting for broadcast to client 2"),
        }

        // Client 1 should also receive the broadcast (echo)
        let result = timeout(Duration::from_secs(5), stream1.next()).await;
        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let response: ServerMessage = serde_json::from_str(&text).unwrap();
                assert!(matches!(response, ServerMessage::Op(_)));
            }
            Ok(other) => panic!("Unexpected message: {:?}", other),
            Err(_) => panic!("Timeout waiting for broadcast echo to client 1"),
        }

        server.shutdown();
    }

    #[tokio::test]
    async fn test_snapshot() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        let (ws, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink, mut stream) = ws.split();

        // First create an issue
        let op = Op::new(
            Hlc::new(1000, 0, 1),
            OpPayload::create_issue("test-1".into(), IssueType::Task, "Test".into()),
        );
        sink.send(Message::Text(ClientMessage::op(op).to_json().unwrap()))
            .await
            .unwrap();

        // Consume the broadcast (with timeout)
        let _ = timeout(Duration::from_secs(5), stream.next()).await;

        // Request snapshot
        sink.send(Message::Text(ClientMessage::snapshot().to_json().unwrap()))
            .await
            .unwrap();

        // Receive snapshot response (with timeout)
        let result = timeout(Duration::from_secs(5), stream.next()).await;
        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let response: ServerMessage = serde_json::from_str(&text).unwrap();
                if let ServerMessage::SnapshotResponse { issues, .. } = response {
                    assert_eq!(issues.len(), 1);
                    assert_eq!(issues[0].id, "test-1");
                } else {
                    panic!("Expected SnapshotResponse, got {:?}", response);
                }
            }
            Ok(other) => panic!("Expected snapshot response, got {:?}", other),
            Err(_) => panic!("Timeout waiting for snapshot response"),
        }

        server.shutdown();
    }

    #[tokio::test]
    async fn test_sync() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        let (ws, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink, mut stream) = ws.split();

        // Create two issues
        let op1 = Op::new(
            Hlc::new(1000, 0, 1),
            OpPayload::create_issue("test-1".into(), IssueType::Task, "Test 1".into()),
        );
        let op2 = Op::new(
            Hlc::new(2000, 0, 1),
            OpPayload::create_issue("test-2".into(), IssueType::Task, "Test 2".into()),
        );

        sink.send(Message::Text(ClientMessage::op(op1).to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), stream.next()).await; // Consume broadcast

        sink.send(Message::Text(ClientMessage::op(op2).to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), stream.next()).await; // Consume broadcast

        // Request sync since before op2
        let sync_msg = ClientMessage::sync(Hlc::new(1500, 0, 0));
        sink.send(Message::Text(sync_msg.to_json().unwrap()))
            .await
            .unwrap();

        // Receive sync response (with timeout)
        let result = timeout(Duration::from_secs(5), stream.next()).await;
        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let response: ServerMessage = serde_json::from_str(&text).unwrap();
                if let ServerMessage::SyncResponse { ops } = response {
                    assert_eq!(ops.len(), 1);
                    assert_eq!(ops[0].id.wall_ms, 2000);
                } else {
                    panic!("Expected SyncResponse, got {:?}", response);
                }
            }
            Ok(other) => panic!("Expected sync response, got {:?}", other),
            Err(_) => panic!("Timeout waiting for sync response"),
        }

        server.shutdown();
    }

    #[tokio::test]
    async fn test_malformed_json_returns_error() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        let (ws_stream, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink, mut stream) = ws_stream.split();

        // Send invalid JSON
        sink.send(Message::Text("not valid json".into()))
            .await
            .unwrap();

        // Should receive an error response
        let result = timeout(Duration::from_secs(5), stream.next()).await;
        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let response: ServerMessage = serde_json::from_str(&text).unwrap();
                assert!(matches!(response, ServerMessage::Error { .. }));
            }
            Ok(other) => panic!("Expected error response, got {:?}", other),
            Err(_) => panic!("Timeout waiting for error response"),
        }

        server.shutdown();
    }

    #[tokio::test]
    async fn test_websocket_ping_frame() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        let (ws_stream, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink, mut stream) = ws_stream.split();

        // Send raw WebSocket Ping frame (not ClientMessage::Ping)
        sink.send(Message::Ping(vec![1, 2, 3])).await.unwrap();

        // Should receive a Pong frame
        let result = timeout(Duration::from_secs(5), stream.next()).await;
        match result {
            Ok(Some(Ok(Message::Pong(data)))) => {
                assert_eq!(data, vec![1, 2, 3]);
            }
            Ok(other) => panic!("Expected Pong frame, got {:?}", other),
            Err(_) => panic!("Timeout waiting for Pong frame"),
        }

        server.shutdown();
    }

    #[tokio::test]
    async fn test_client_close_frame() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        let (ws_stream, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink, mut stream) = ws_stream.split();

        // Send Close frame
        sink.send(Message::Close(None)).await.unwrap();

        // The server should close the connection
        // We may receive a Close frame back, or the stream may end, or we may get
        // a protocol error (ResetWithoutClosingHandshake) - all are acceptable
        let result = timeout(Duration::from_secs(5), stream.next()).await;
        match result {
            Ok(Some(Ok(Message::Close(_)))) => {
                // Got close response
            }
            Ok(None) => {
                // Stream ended cleanly
            }
            Ok(Some(Err(_))) => {
                // Protocol error on close is acceptable
            }
            Ok(other) => panic!("Expected close or end, got {:?}", other),
            Err(_) => panic!("Timeout waiting for close"),
        }

        server.shutdown();
    }

    #[tokio::test]
    async fn test_duplicate_op_skipped() {
        use tokio::time::{timeout, Duration};

        let server = TestServer::start().await;

        let (ws_stream, _) = connect_async(&server.ws_url()).await.unwrap();
        let (mut sink, mut stream) = ws_stream.split();

        // Send the same op twice
        let op = Op::new(
            Hlc::new(1000, 0, 1),
            OpPayload::create_issue("dup-test".into(), IssueType::Task, "Test".into()),
        );
        let msg = ClientMessage::op(op.clone());

        // First send
        sink.send(Message::Text(msg.to_json().unwrap()))
            .await
            .unwrap();

        // Consume broadcast
        let _ = timeout(Duration::from_secs(5), stream.next()).await;

        // Second send (duplicate)
        sink.send(Message::Text(msg.to_json().unwrap()))
            .await
            .unwrap();

        // The duplicate should be silently skipped - no broadcast
        // Send a ping to verify connection is still alive and no extra messages
        let ping = ClientMessage::ping(999);
        sink.send(Message::Text(ping.to_json().unwrap()))
            .await
            .unwrap();

        // Should get pong, not another op broadcast
        let result = timeout(Duration::from_secs(5), stream.next()).await;
        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let response: ServerMessage = serde_json::from_str(&text).unwrap();
                assert!(matches!(response, ServerMessage::Pong { id: 999 }));
            }
            Ok(other) => panic!("Expected pong, got {:?}", other),
            Err(_) => panic!("Timeout waiting for pong"),
        }

        server.shutdown();
    }

    #[test]
    fn test_server_state_new_creates_database() {
        let temp = tempfile::tempdir().unwrap();
        let _state = ServerState::new(temp.path(), None).unwrap();

        // Verify the database file was created
        assert!(temp.path().join("issues.db").exists());
        // Note: oplog.jsonl is created lazily on first append
    }

    #[test]
    fn test_server_state_subscribe_returns_receiver() {
        let temp = tempfile::tempdir().unwrap();
        let state = ServerState::new(temp.path(), None).unwrap();

        // Should be able to subscribe multiple times
        let _rx1 = state.subscribe();
        let _rx2 = state.subscribe();
    }

    #[tokio::test]
    async fn test_apply_op_updates_high_water_mark() {
        let temp = tempfile::tempdir().unwrap();
        let state = ServerState::new(temp.path(), None).unwrap();

        // Initial snapshot should have min HLC
        let (_, _, initial_hlc) = state.snapshot().await.unwrap();

        // Apply an op with a specific HLC
        let op = Op::new(
            Hlc::new(5000, 0, 1),
            OpPayload::create_issue("hwm-test".into(), IssueType::Task, "Test".into()),
        );
        state.apply_op(op).await.unwrap();

        // Snapshot should now reflect the op's HLC
        let (_, _, new_hlc) = state.snapshot().await.unwrap();
        assert!(new_hlc > initial_hlc, "HLC should be updated after op");
        assert_eq!(new_hlc.wall_ms, 5000);
    }

    // These tests use run_single_connection to ensure handle_connection is covered.
    #[tokio::test]
    async fn test_direct_ping_pong_coverage() {
        use tokio::net::TcpStream;
        use tokio::time::{timeout, Duration};

        let temp = tempfile::tempdir().unwrap();
        let state = ServerState::new(temp.path(), None).unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Run server and client concurrently using select! instead of spawn
        // This keeps everything in the same task for coverage
        tokio::select! {
            // Server accepts and handles one connection
            result = async {
                let (stream, peer_addr) = listener.accept().await.unwrap();
                server::handle_connection(stream, peer_addr, state.clone()).await
            } => {
                // Server finished handling connection
                result.ok();
            }
            // Client connects and sends ping
            _ = async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                let stream = TcpStream::connect(addr).await.unwrap();
                let (ws_stream, _) = tokio_tungstenite::client_async(
                    format!("ws://{}", addr),
                    stream
                ).await.unwrap();
                let (mut sink, mut stream) = ws_stream.split();

                // Send ping
                let ping = ClientMessage::ping(777);
                sink.send(Message::Text(ping.to_json().unwrap()))
                    .await
                    .unwrap();

                // Receive pong
                let result = timeout(Duration::from_secs(5), stream.next()).await;
                match result {
                    Ok(Some(Ok(Message::Text(text)))) => {
                        let response: ServerMessage = serde_json::from_str(&text).unwrap();
                        assert!(matches!(response, ServerMessage::Pong { id: 777 }));
                    }
                    other => panic!("Expected pong, got {:?}", other),
                }

                // Close connection
                sink.send(Message::Close(None)).await.ok();
            } => {}
        }
    }

    #[tokio::test]
    async fn test_direct_op_broadcast_coverage() {
        use tokio::net::TcpStream;
        use tokio::time::{timeout, Duration};

        let temp = tempfile::tempdir().unwrap();
        let state = ServerState::new(temp.path(), None).unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::select! {
            result = async {
                let (stream, peer_addr) = listener.accept().await.unwrap();
                server::handle_connection(stream, peer_addr, state.clone()).await
            } => {
                result.ok();
            }
            _ = async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                let stream = TcpStream::connect(addr).await.unwrap();
                let (ws_stream, _) = tokio_tungstenite::client_async(
                    format!("ws://{}", addr),
                    stream
                ).await.unwrap();
                let (mut sink, mut stream) = ws_stream.split();

                // Send an op
                let op = Op::new(
                    Hlc::new(2000, 0, 1),
                    OpPayload::create_issue("direct-test".into(), IssueType::Task, "Direct Test".into()),
                );
                let msg = ClientMessage::op(op.clone());
                sink.send(Message::Text(msg.to_json().unwrap()))
                    .await
                    .unwrap();

                // Receive broadcast
                let result = timeout(Duration::from_secs(5), stream.next()).await;
                match result {
                    Ok(Some(Ok(Message::Text(text)))) => {
                        let response: ServerMessage = serde_json::from_str(&text).unwrap();
                        if let ServerMessage::Op(received_op) = response {
                            assert_eq!(received_op.id, op.id);
                        } else {
                            panic!("Expected Op broadcast, got {:?}", response);
                        }
                    }
                    other => panic!("Unexpected: {:?}", other),
                }

                sink.send(Message::Close(None)).await.ok();
            } => {}
        }
    }
}
