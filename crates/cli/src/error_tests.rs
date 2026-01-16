// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

#[test]
fn test_error_not_initialized_display() {
    let err = Error::NotInitialized;
    assert!(err.to_string().contains("not initialized"));
    assert!(err.to_string().contains("wk init"));
}

#[test]
fn test_error_already_initialized_display() {
    let err = Error::AlreadyInitialized("/path/to/work".to_string());
    assert!(err.to_string().contains("already initialized"));
    assert!(err.to_string().contains("/path/to/work"));
}

#[test]
fn test_error_issue_not_found_display() {
    let err = Error::IssueNotFound("wk-abc123".to_string());
    assert!(err.to_string().contains("issue not found"));
    assert!(err.to_string().contains("wk-abc123"));
}

#[test]
fn test_error_invalid_transition_display() {
    let err = Error::InvalidTransition {
        from: "todo".to_string(),
        to: "done".to_string(),
        valid_targets: "in_progress, closed (with reason)".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid status transition"));
    assert!(msg.contains("todo"));
    assert!(msg.contains("done"));
    assert!(msg.contains("in_progress"));
}

#[test]
fn test_error_cycle_detected_display() {
    let err = Error::CycleDetected;
    assert!(err.to_string().contains("cycle"));
}

#[test]
fn test_error_self_dependency_display() {
    let err = Error::SelfDependency;
    assert!(err.to_string().contains("self-dependency"));
}

#[test]
fn test_error_dependency_not_found_display() {
    let err = Error::DependencyNotFound {
        from: "issue-a".to_string(),
        rel: "blocks".to_string(),
        to: "issue-b".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("dependency not found"));
    assert!(msg.contains("issue-a"));
    assert!(msg.contains("blocks"));
    assert!(msg.contains("issue-b"));
}

#[test]
fn test_error_invalid_issue_type_display() {
    let err = Error::InvalidIssueType("invalid".to_string());
    let msg = err.to_string();
    assert!(msg.contains("invalid issue type"));
    assert!(msg.contains("invalid"));
    assert!(msg.contains("feature, task, bug"));
}

#[test]
fn test_error_invalid_status_display() {
    let err = Error::InvalidStatus("badstatus".to_string());
    let msg = err.to_string();
    assert!(msg.contains("invalid status"));
    assert!(msg.contains("badstatus"));
    assert!(msg.contains("todo, in_progress, done, closed"));
}

#[test]
fn test_error_invalid_relation_display() {
    let err = Error::InvalidRelation("depends".to_string());
    let msg = err.to_string();
    assert!(msg.contains("invalid relation"));
    assert!(msg.contains("depends"));
    assert!(msg.contains("blocks, blocked-by, tracks, tracked-by"));
}

#[test]
fn test_error_invalid_prefix_display() {
    let err = Error::InvalidPrefix;
    assert!(err.to_string().contains("invalid prefix"));
    assert!(err.to_string().contains("2+ lowercase"));
}

#[test]
fn test_error_invalid_input_display() {
    let err = Error::InvalidInput("custom error message".to_string());
    assert_eq!(err.to_string(), "custom error message");
}

#[test]
fn test_error_config_display() {
    let err = Error::Config("missing field".to_string());
    assert!(err.to_string().contains("config error"));
    assert!(err.to_string().contains("missing field"));
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: Error = io_err.into();
    assert!(err.to_string().contains("io error"));
}

#[test]
fn test_error_from_json() {
    // Create a JSON parsing error
    let result: std::result::Result<i32, serde_json::Error> = serde_json::from_str("invalid");
    let json_err = result.unwrap_err();
    let err: Error = json_err.into();
    assert!(err.to_string().contains("json error"));
}
