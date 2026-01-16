// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::hlc::Hlc;
use crate::issue::IssueType;
use crate::op::OpPayload;
use chrono::Utc;
use yare::parameterized;

fn test_op() -> Op {
    Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".to_string(), IssueType::Task, "Title".to_string()),
    )
}

fn test_issue() -> Issue {
    Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Test Issue".to_string(),
        Utc::now(),
    )
}

// ClientMessage roundtrip tests
#[parameterized(
    op = { ClientMessage::op(Op::new(Hlc::new(1000, 0, 1), OpPayload::create_issue("test-1".to_string(), IssueType::Task, "Title".to_string()))) },
    sync = { ClientMessage::sync(Hlc::new(1000, 5, 42)) },
    snapshot = { ClientMessage::snapshot() },
    ping = { ClientMessage::ping(12345) },
)]
fn client_message_roundtrip(msg: ClientMessage) {
    let json = msg.to_json().unwrap();
    let parsed = ClientMessage::from_json(&json).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn server_message_op_roundtrip() {
    let msg = ServerMessage::op(test_op());
    let json = msg.to_json().unwrap();
    let parsed = ServerMessage::from_json(&json).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn server_message_sync_response_roundtrip() {
    let ops = vec![test_op()];
    let msg = ServerMessage::sync_response(ops);
    let json = msg.to_json().unwrap();
    let parsed = ServerMessage::from_json(&json).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn server_message_snapshot_response_roundtrip() {
    let issues = vec![test_issue()];
    let tags = vec![("test-1".to_string(), "urgent".to_string())];
    let msg = ServerMessage::snapshot_response(issues, tags, Hlc::new(5000, 0, 1));
    let json = msg.to_json().unwrap();
    let parsed = ServerMessage::from_json(&json).unwrap();
    assert_eq!(msg, parsed);
}

#[parameterized(
    pong = { ServerMessage::pong(12345) },
    error = { ServerMessage::error("Something went wrong") },
)]
fn server_message_simple_roundtrip(msg: ServerMessage) {
    let json = msg.to_json().unwrap();
    let parsed = ServerMessage::from_json(&json).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn message_json_format() {
    let msg = ClientMessage::snapshot();
    let json = msg.to_json().unwrap();
    assert!(json.contains("\"type\":\"snapshot\""));

    let msg = ClientMessage::sync(Hlc::new(1000, 0, 1));
    let json = msg.to_json().unwrap();
    assert!(json.contains("\"type\":\"sync\""));
    assert!(json.contains("\"since\""));

    let msg = ServerMessage::error("test error");
    let json = msg.to_json().unwrap();
    assert!(json.contains("\"type\":\"error\""));
    assert!(json.contains("\"message\":\"test error\""));
}
