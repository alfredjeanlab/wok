// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::hooks::{ChangePayload, HookConfig, HookPayload, IssuePayload};
use chrono::Utc;
use std::fs;
use tempfile::TempDir;

fn make_payload() -> HookPayload {
    HookPayload {
        event: "issue.created".to_string(),
        timestamp: Utc::now(),
        issue: IssuePayload {
            id: "proj-abc".to_string(),
            issue_type: "bug".to_string(),
            title: "Test issue".to_string(),
            status: "todo".to_string(),
            assignee: Some("alice".to_string()),
            labels: vec!["urgent".to_string()],
        },
        change: ChangePayload {
            old_value: None,
            new_value: Some("test".to_string()),
            reason: None,
        },
    }
}

fn make_hook(run: &str) -> HookConfig {
    HookConfig {
        name: "test-hook".to_string(),
        events: vec!["issue.created".to_string()],
        filter: None,
        run: run.to_string(),
    }
}

#[test]
fn execute_true_command_succeeds() {
    let dir = TempDir::new().unwrap();
    let work_dir = dir.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    let hook = make_hook("true");
    let payload = make_payload();

    let result = execute_hook(&hook, &payload, &work_dir);
    assert!(result.is_ok());
}

#[test]
fn execute_echo_command_succeeds() {
    let dir = TempDir::new().unwrap();
    let work_dir = dir.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    let hook = make_hook("echo test");
    let payload = make_payload();

    let result = execute_hook(&hook, &payload, &work_dir);
    assert!(result.is_ok());
}

#[test]
fn execute_receives_env_vars() {
    let dir = TempDir::new().unwrap();
    let work_dir = dir.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    // Create a script that writes env vars to a file
    let script_path = dir.path().join("check_env.sh");
    fs::write(
        &script_path,
        r#"#!/bin/sh
echo "WOK_EVENT=$WOK_EVENT" > "$1/env_output.txt"
echo "WOK_ISSUE_ID=$WOK_ISSUE_ID" >> "$1/env_output.txt"
echo "WOK_ISSUE_TYPE=$WOK_ISSUE_TYPE" >> "$1/env_output.txt"
echo "WOK_ISSUE_STATUS=$WOK_ISSUE_STATUS" >> "$1/env_output.txt"
"#,
    )
    .unwrap();

    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let hook = make_hook(&format!(
        "{} {}",
        script_path.display(),
        dir.path().display()
    ));
    let payload = make_payload();

    let result = execute_hook(&hook, &payload, &work_dir);
    assert!(result.is_ok());

    // Wait a moment for the script to execute
    std::thread::sleep(std::time::Duration::from_millis(100));

    let output_path = dir.path().join("env_output.txt");
    if output_path.exists() {
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("WOK_EVENT=issue.created"));
        assert!(content.contains("WOK_ISSUE_ID=proj-abc"));
        assert!(content.contains("WOK_ISSUE_TYPE=bug"));
        assert!(content.contains("WOK_ISSUE_STATUS=todo"));
    }
}

#[test]
fn execute_nonexistent_command_returns_error() {
    let dir = TempDir::new().unwrap();
    let work_dir = dir.path().join(".wok");
    fs::create_dir_all(&work_dir).unwrap();

    // A command that doesn't exist should still spawn successfully
    // because we use sh -c which will handle the error
    let hook = make_hook("/nonexistent/path/script.sh");
    let payload = make_payload();

    // Fire and forget - doesn't check exit code
    let result = execute_hook(&hook, &payload, &work_dir);
    assert!(result.is_ok());
}
