// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Integration tests for the sync module.
//!
//! These tests verify the complete sync flow including:
//! - Client sync against server
//! - Queue flush on reconnect
//! - Multiple clients syncing

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::client::{ConnectionState, SyncClient, SyncConfig};
use super::queue::OfflineQueue;
use super::test_helpers::make_test_op_with_node;
use super::transport_tests::MockTransport;
use tempfile::tempdir;
use wk_core::issue::IssueType;
use wk_core::protocol::ServerMessage;
use wk_core::Hlc;

/// Test the complete sync-on-connect flow:
/// 1. Client queues ops while offline
/// 2. Client connects
/// 3. Client flushes queue
/// 4. Client requests sync
/// 5. Server responds with missed ops
#[tokio::test]
async fn test_full_sync_flow() {
    let dir = tempdir().unwrap();
    let config = SyncConfig::default();
    let transport = MockTransport::new();

    let queue_path = dir.path().join("queue.jsonl");
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    // Queue operations while offline
    assert_eq!(client.state(), ConnectionState::Disconnected);

    client
        .send_op(make_test_op_with_node(1000, 1))
        .await
        .unwrap();
    client
        .send_op(make_test_op_with_node(2000, 1))
        .await
        .unwrap();

    assert_eq!(client.pending_ops_count().unwrap(), 2);
    assert_eq!(client.last_hlc().unwrap().wall_ms, 2000);

    // Connect
    client.connect().await.unwrap();
    assert_eq!(client.state(), ConnectionState::Connected);

    // Flush queue and sync
    client.sync_on_connect().await.unwrap();

    // Queue should be empty after flush
    assert_eq!(client.pending_ops_count().unwrap(), 0);
}

/// Test receiving operations from server updates local state
#[tokio::test]
async fn test_receive_server_ops() {
    let dir = tempdir().unwrap();
    let config = SyncConfig::default();
    let transport = MockTransport::new();

    // Queue server ops before client connects
    let server_op1 = make_test_op_with_node(3000, 2);
    let server_op2 = make_test_op_with_node(4000, 2);
    transport.queue_incoming(ServerMessage::op(server_op1.clone()));
    transport.queue_incoming(ServerMessage::op(server_op2.clone()));

    let queue_path = dir.path().join("queue.jsonl");
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    client.connect().await.unwrap();

    // Receive first op
    let msg1 = client.recv().await.unwrap().unwrap();
    if let ServerMessage::Op(op) = msg1 {
        assert_eq!(op.id, server_op1.id);
    } else {
        panic!("Expected Op message");
    }

    // last_hlc should update
    assert_eq!(client.last_hlc().unwrap().wall_ms, 3000);

    // Receive second op
    let msg2 = client.recv().await.unwrap().unwrap();
    if let ServerMessage::Op(op) = msg2 {
        assert_eq!(op.id, server_op2.id);
    } else {
        panic!("Expected Op message");
    }

    // last_hlc should update to higher value
    assert_eq!(client.last_hlc().unwrap().wall_ms, 4000);
}

/// Test sync response processing
#[tokio::test]
async fn test_sync_response() {
    let dir = tempdir().unwrap();
    let config = SyncConfig::default();
    let transport = MockTransport::new();

    // Queue a sync response
    let ops = vec![
        make_test_op_with_node(5000, 2),
        make_test_op_with_node(6000, 2),
    ];
    transport.queue_incoming(ServerMessage::sync_response(ops.clone()));

    let queue_path = dir.path().join("queue.jsonl");
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    // Set initial HLC
    client
        .send_op(make_test_op_with_node(1000, 1))
        .await
        .unwrap();

    client.connect().await.unwrap();

    // Flush queue first
    client.flush_queue().await.unwrap();

    // Request sync
    client.request_sync(Hlc::new(1000, 0, 1)).await.unwrap();

    // Receive sync response
    let msg = client.recv().await.unwrap().unwrap();
    if let ServerMessage::SyncResponse { ops: recv_ops } = msg {
        assert_eq!(recv_ops.len(), 2);
        assert_eq!(recv_ops[0].id.wall_ms, 5000);
        assert_eq!(recv_ops[1].id.wall_ms, 6000);
    } else {
        panic!("Expected SyncResponse");
    }
}

/// Test snapshot response processing
#[tokio::test]
async fn test_snapshot_response() {
    let dir = tempdir().unwrap();
    let config = SyncConfig::default();
    let transport = MockTransport::new();

    // Queue a snapshot response
    let issues = vec![wk_core::Issue {
        id: "test-1".to_string(),
        issue_type: IssueType::Task,
        title: "Test Issue".to_string(),
        status: wk_core::Status::Todo,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
    }];
    let since = Hlc::new(10000, 0, 0);
    transport.queue_incoming(ServerMessage::snapshot_response(
        issues.clone(),
        vec![],
        since,
    ));

    let queue_path = dir.path().join("queue.jsonl");
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    client.connect().await.unwrap();
    client.request_snapshot().await.unwrap();

    let msg = client.recv().await.unwrap().unwrap();
    if let ServerMessage::SnapshotResponse {
        issues: recv_issues,
        since: recv_since,
        ..
    } = msg
    {
        assert_eq!(recv_issues.len(), 1);
        assert_eq!(recv_issues[0].id, "test-1");
        assert_eq!(recv_since, since);
    } else {
        panic!("Expected SnapshotResponse");
    }
}

/// Test queue persistence across client restarts
#[tokio::test]
async fn test_queue_persistence_across_restarts() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("queue.jsonl");

    // First client instance queues ops
    {
        let config = SyncConfig::default();
        let transport = MockTransport::new();
        let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

        client
            .send_op(make_test_op_with_node(1000, 1))
            .await
            .unwrap();
        client
            .send_op(make_test_op_with_node(2000, 1))
            .await
            .unwrap();

        // Don't connect - ops stay queued
        assert_eq!(client.pending_ops_count().unwrap(), 2);
    }

    // Second client instance sees queued ops
    {
        let config = SyncConfig::default();
        let transport = MockTransport::new();
        let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

        // Queue should still have the ops
        assert_eq!(client.pending_ops_count().unwrap(), 2);

        // Connect and flush
        client.connect().await.unwrap();
        let flushed = client.flush_queue().await.unwrap();

        assert_eq!(flushed, 2);
        assert_eq!(client.pending_ops_count().unwrap(), 0);
    }
}

/// Test queue operations directly
#[tokio::test]
async fn test_offline_queue_operations() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("queue.jsonl");

    let mut queue = OfflineQueue::open(&queue_path).unwrap();

    // Enqueue multiple ops
    let op1 = make_test_op_with_node(1000, 1);
    let op2 = make_test_op_with_node(2000, 1);
    let op3 = make_test_op_with_node(3000, 1);

    queue.enqueue(&op1).unwrap();
    queue.enqueue(&op2).unwrap();
    queue.enqueue(&op3).unwrap();

    assert_eq!(queue.len().unwrap(), 3);

    // Peek without removing
    let ops = queue.peek_all().unwrap();
    assert_eq!(ops.len(), 3);
    assert_eq!(queue.len().unwrap(), 3); // Still there

    // Remove first two
    queue.remove_first(2).unwrap();
    assert_eq!(queue.len().unwrap(), 1);

    let remaining = queue.peek_all().unwrap();
    assert_eq!(remaining[0].id.wall_ms, 3000);

    // Clear remaining
    queue.clear().unwrap();
    assert!(queue.is_empty().unwrap());
}

/// Test ping/pong for connection health check
#[tokio::test]
async fn test_ping_pong_health_check() {
    let dir = tempdir().unwrap();
    let config = SyncConfig::default();
    let transport = MockTransport::new();

    // Queue pong response
    transport.queue_incoming(ServerMessage::pong(12345));

    let queue_path = dir.path().join("queue.jsonl");
    let mut client = SyncClient::with_transport(config, transport, &queue_path).unwrap();

    client.connect().await.unwrap();

    // Send ping
    client.ping(12345).await.unwrap();

    // Receive pong
    let msg = client.recv().await.unwrap().unwrap();
    assert!(matches!(msg, ServerMessage::Pong { id: 12345 }));
}
