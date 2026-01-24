// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_test_repo() -> (TempDir, GitBacking) {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first, configure user, and checkout orphan branch
    // before GitBacking::new (init_repo skips setup if .git exists)
    Command::new("git")
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(temp.path())
        .args(["config", "user.email", "test@test.local"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(temp.path())
        .args(["config", "user.name", "Test User"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(temp.path())
        .args(["checkout", "--orphan", "wk/oplog"])
        .output()
        .unwrap();

    // Create an oplog file
    fs::write(temp.path().join("oplog.jsonl"), "").unwrap();

    let config = GitBackingConfig {
        repo_path: temp.path().to_path_buf(),
        branch: "wk/oplog".to_string(),
        commit_interval: Duration::from_secs(60),
        remote: None,
    };

    let backing = GitBacking::new(config).unwrap();
    (temp, backing)
}

#[test]
fn test_git_backing_init() {
    let (temp, _backing) = setup_test_repo();

    // Verify git repo was created
    assert!(temp.path().join(".git").exists());
}

#[tokio::test]
async fn test_mark_dirty_and_commit() {
    let (temp, backing) = setup_test_repo();

    // Initially not dirty, so commit should do nothing
    let committed = backing.commit_if_dirty().await.unwrap();
    assert!(!committed);

    // Add some content to oplog
    fs::write(temp.path().join("oplog.jsonl"), "{\"test\":true}\n").unwrap();

    // Mark dirty
    backing.mark_dirty().await;

    // Now commit should work
    let committed = backing.commit_if_dirty().await.unwrap();
    assert!(committed);

    // Second commit should do nothing (not dirty anymore)
    let committed = backing.commit_if_dirty().await.unwrap();
    assert!(!committed);
}

#[test]
fn test_default_config() {
    let config = GitBackingConfig::default();
    assert_eq!(config.branch, "wk/oplog");
    assert_eq!(config.commit_interval, Duration::from_secs(90));
    assert!(config.remote.is_none());
}
