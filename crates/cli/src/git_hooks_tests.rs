// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(path: &Path) {
    Command::new("git")
        .current_dir(path)
        .args(["init"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(path)
        .args(["config", "user.email", "test@test.com"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(path)
        .args(["config", "user.name", "Test"])
        .output()
        .unwrap();
}

#[test]
fn test_find_git_dir() {
    let temp = TempDir::new().unwrap();
    init_git_repo(temp.path());

    let git_dir = find_git_dir(temp.path()).unwrap();
    assert!(git_dir.ends_with(".git"));
    assert!(git_dir.exists());
}

#[test]
fn test_find_git_dir_not_repo() {
    let temp = TempDir::new().unwrap();
    let result = find_git_dir(temp.path());
    assert!(result.is_err());
}

#[test]
fn test_install_hooks() {
    let temp = TempDir::new().unwrap();
    init_git_repo(temp.path());

    install_hooks(temp.path()).unwrap();

    let git_dir = find_git_dir(temp.path()).unwrap();
    let post_push = git_dir.join("hooks/post-push");
    let post_merge = git_dir.join("hooks/post-merge");

    assert!(post_push.exists());
    assert!(post_merge.exists());

    let post_push_content = fs::read_to_string(&post_push).unwrap();
    assert!(post_push_content.contains(WK_HOOK_MARKER));
    assert!(post_push_content.contains("wk remote sync"));

    let post_merge_content = fs::read_to_string(&post_merge).unwrap();
    assert!(post_merge_content.contains(WK_HOOK_MARKER));
    assert!(post_merge_content.contains("wk remote sync"));

    // Check executable permission
    let perms = fs::metadata(&post_push).unwrap().permissions();
    assert_eq!(perms.mode() & 0o111, 0o111);
}

#[test]
fn test_install_hooks_idempotent() {
    let temp = TempDir::new().unwrap();
    init_git_repo(temp.path());

    install_hooks(temp.path()).unwrap();
    install_hooks(temp.path()).unwrap();

    let git_dir = find_git_dir(temp.path()).unwrap();
    let post_push = git_dir.join("hooks/post-push");
    let content = fs::read_to_string(&post_push).unwrap();

    // Should only have one marker
    let marker_count = content.matches(WK_HOOK_MARKER).count();
    assert_eq!(marker_count, 1);
}

#[test]
fn test_install_hooks_preserves_existing() {
    let temp = TempDir::new().unwrap();
    init_git_repo(temp.path());

    let git_dir = find_git_dir(temp.path()).unwrap();
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    let post_push = hooks_dir.join("post-push");
    fs::write(&post_push, "#!/bin/sh\necho 'existing hook'\n").unwrap();

    install_hooks(temp.path()).unwrap();

    let content = fs::read_to_string(&post_push).unwrap();
    assert!(content.contains("existing hook"));
    assert!(content.contains(WK_HOOK_MARKER));
}
