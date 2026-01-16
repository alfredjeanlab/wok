// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use yare::parameterized;

#[parameterized(
    issue_not_found = { Error::IssueNotFound("test-123".into()), "test-123" },
    cycle_detected = { Error::CycleDetected, "cycle" },
    self_dependency = { Error::SelfDependency, "self-dependency" },
)]
fn error_display_contains(err: Error, expected: &str) {
    assert!(err.to_string().contains(expected));
}

#[test]
fn error_invalid_transition_display() {
    let err = Error::InvalidTransition {
        from: "todo".into(),
        to: "done".into(),
        valid_targets: "in_progress".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("todo"));
    assert!(msg.contains("done"));
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn error_from_json() {
    let json_err = serde_json::from_str::<()>("invalid").unwrap_err();
    let err: Error = json_err.into();
    assert!(matches!(err, Error::Json(_)));
}
