// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for daemon IPC protocol.

#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use super::ipc::*;
use yare::parameterized;

#[parameterized(
    status = { DaemonRequest::Status },
    shutdown = { DaemonRequest::Shutdown },
    ping = { DaemonRequest::Ping },
    hello = { DaemonRequest::Hello { version: "0.1.0".to_string() } },
)]
fn daemon_request_serialization(request: DaemonRequest) {
    let json = serde_json::to_string(&request).unwrap();
    let parsed: DaemonRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request, parsed);
}

#[parameterized(
    status = { DaemonResponse::Status(DaemonStatus::new(1234, 3600)) },
    shutting_down = { DaemonResponse::ShuttingDown },
    pong = { DaemonResponse::Pong },
    error = { DaemonResponse::Error { message: "test error".to_string() } },
    hello = { DaemonResponse::Hello { version: "0.1.0".to_string() } },
)]
fn daemon_response_serialization(response: DaemonResponse) {
    let json = serde_json::to_string(&response).unwrap();
    let parsed: DaemonResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(response, parsed);
}

#[test]
fn daemon_status_new() {
    let status = DaemonStatus::new(5678, 7200);

    assert_eq!(status.pid, 5678);
    assert_eq!(status.uptime_secs, 7200);
}

#[parameterized(
    status = { DaemonRequest::Status },
    shutdown = { DaemonRequest::Shutdown },
    ping = { DaemonRequest::Ping },
    hello = { DaemonRequest::Hello { version: "0.1.0".to_string() } },
)]
fn framing_roundtrip_request(request: DaemonRequest) {
    let mut buf = Vec::new();
    framing::write_request(&mut buf, &request).unwrap();

    let mut cursor = Cursor::new(buf);
    let decoded = framing::read_request(&mut cursor).unwrap();
    assert_eq!(request, decoded);
}

#[parameterized(
    status = { DaemonResponse::Status(DaemonStatus::new(1000, 100)) },
    shutting_down = { DaemonResponse::ShuttingDown },
    pong = { DaemonResponse::Pong },
    error = { DaemonResponse::Error { message: "test".to_string() } },
    hello = { DaemonResponse::Hello { version: "0.1.0".to_string() } },
)]
fn framing_roundtrip_response(response: DaemonResponse) {
    let mut buf = Vec::new();
    framing::write_response(&mut buf, &response).unwrap();

    let mut cursor = Cursor::new(buf);
    let decoded = framing::read_response(&mut cursor).unwrap();
    assert_eq!(response, decoded);
}
