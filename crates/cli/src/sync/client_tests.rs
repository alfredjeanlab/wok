// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for the sync client module.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::client::{ConnectionState, SyncClient, SyncConfig};
use super::test_helpers::make_test_op;
use super::transport_tests::MockTransport;
use tempfile::tempdir;
use wk_core::protocol::ServerMessage;
use wk_core::Hlc;

fn make_client_with_mock(dir: &tempfile::TempDir) -> SyncClient<MockTransport> {
    let config = SyncConfig::default();
    let transport = MockTransport::new();
    let queue_path = dir.path().join("queue.jsonl");
    SyncClient::with_transport(config, transport, &queue_path).unwrap()
}

#[tokio::test]
async fn test_client_connect_disconnect() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    assert_eq!(client.state(), ConnectionState::Disconnected);
    assert!(!client.is_connected());

    client.connect().await.unwrap();

    assert_eq!(client.state(), ConnectionState::Connected);
    assert!(client.is_connected());

    client.disconnect().await.unwrap();

    assert_eq!(client.state(), ConnectionState::Disconnected);
    assert!(!client.is_connected());
}

#[tokio::test]
async fn test_client_send_op_when_connected() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    client.connect().await.unwrap();

    let op = make_test_op(1000);
    client.send_op(op.clone()).await.unwrap();

    // Op should have been sent, not queued
    assert_eq!(client.pending_ops_count().unwrap(), 0);
}

#[tokio::test]
async fn test_client_queue_op_when_disconnected() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    // Don't connect - stay disconnected
    let op = make_test_op(1000);
    client.send_op(op).await.unwrap();

    // Op should be queued
    assert_eq!(client.pending_ops_count().unwrap(), 1);
}

#[tokio::test]
async fn test_client_flush_queue() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    // Queue some ops while disconnected
    client.send_op(make_test_op(1000)).await.unwrap();
    client.send_op(make_test_op(2000)).await.unwrap();

    assert_eq!(client.pending_ops_count().unwrap(), 2);

    // Connect and flush
    client.connect().await.unwrap();
    let flushed = client.flush_queue().await.unwrap();

    // flush_queue sends ops but does NOT clear queue (for reliability)
    // Queue is only cleared after receiving sync response
    assert_eq!(flushed, 2);
    assert_eq!(client.pending_ops_count().unwrap(), 2);

    // Explicit clear_queue() is needed after confirming server receipt
    client.clear_queue().unwrap();
    assert_eq!(client.pending_ops_count().unwrap(), 0);
}

#[tokio::test]
async fn test_client_recv() {
    let dir = tempdir().unwrap();
    let config = SyncConfig::default();
    let transport = MockTransport::new();

    // Queue a message before creating client
    transport.queue_incoming(ServerMessage::pong(42));

    let queue_path = dir.path().join("queue.jsonl");
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    client.connect().await.unwrap();

    let msg = client.recv().await.unwrap();
    assert!(matches!(msg, Some(ServerMessage::Pong { id: 42 })));
}

#[tokio::test]
async fn test_client_last_hlc_updates() {
    let dir = tempdir().unwrap();
    let config = SyncConfig::default();
    let transport = MockTransport::new();

    // Queue an op message
    let incoming_op = make_test_op(5000);
    transport.queue_incoming(ServerMessage::op(incoming_op));

    let queue_path = dir.path().join("queue.jsonl");
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    assert!(client.last_hlc().is_none());

    client.connect().await.unwrap();

    // Send an op - should update last_hlc
    client.send_op(make_test_op(1000)).await.unwrap();
    assert_eq!(client.last_hlc().unwrap().wall_ms, 1000);

    // Send another op - should update last_hlc
    client.send_op(make_test_op(2000)).await.unwrap();
    assert_eq!(client.last_hlc().unwrap().wall_ms, 2000);

    // Receive an op with higher HLC - should update last_hlc
    let _ = client.recv().await.unwrap();
    assert_eq!(client.last_hlc().unwrap().wall_ms, 5000);
}

#[tokio::test]
async fn test_client_ping() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    // Should fail when disconnected
    let result = client.ping(42).await;
    assert!(result.is_err());

    // Should succeed when connected
    client.connect().await.unwrap();
    client.ping(42).await.unwrap();
}

#[tokio::test]
async fn test_client_request_sync() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    client.connect().await.unwrap();

    let since = Hlc::new(1000, 0, 1);
    client.request_sync(since).await.unwrap();
}

#[tokio::test]
async fn test_client_request_snapshot() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    client.connect().await.unwrap();
    client.request_snapshot().await.unwrap();
}

#[tokio::test]
async fn test_client_sync_on_connect_with_hlc() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    // Set a last HLC by sending an op while disconnected
    client.send_op(make_test_op(1000)).await.unwrap();

    client.connect().await.unwrap();
    client.sync_on_connect().await.unwrap();

    // Queue ops are sent but not cleared (queue is cleared after sync response)
    // sync_on_connect doesn't wait for response, so queue still has ops
    assert_eq!(client.pending_ops_count().unwrap(), 1);
}

#[tokio::test]
async fn test_client_sync_on_connect_without_hlc() {
    let dir = tempdir().unwrap();
    let mut client = make_client_with_mock(&dir);

    // Don't set any HLC
    assert!(client.last_hlc().is_none());

    client.connect().await.unwrap();
    client.sync_on_connect().await.unwrap();

    // Should request snapshot since no HLC
}
