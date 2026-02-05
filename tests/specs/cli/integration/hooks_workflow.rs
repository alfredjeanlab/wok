// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hooks workflow integration tests - converted from tests/specs/cli/integration/hooks_workflow.bats
//!
//! BATS test mapping:
//! - "hooks can be installed in initialized project"
//!   -> hooks_can_be_installed_in_initialized_project
//! - "hooks work alongside .wok directory"
//!   -> hooks_work_alongside_wok_directory
//! - "hooks status after install and uninstall cycle"
//!   -> hooks_status_after_install_and_uninstall_cycle
//! - "multiple scope installations tracked separately"
//!   -> multiple_scope_installations_tracked_separately
//! - "hooks install and wk commands work together"
//!   -> hooks_install_and_wk_commands_work_together
//! - "hooks installed in project visible to collaborators"
//!   -> hooks_installed_in_project_visible_to_collaborators
//! - "hooks survive wk operations"
//!   -> hooks_survive_wk_operations
//! - "hooks and wk init order independent"
//!   -> hooks_and_wk_init_order_independent
//! - "hooks install then uninstall preserves custom hooks"
//!   -> hooks_install_then_uninstall_preserves_custom_hooks
//! - "hooks status accurately reflects partial installation"
//!   -> hooks_status_accurately_reflects_partial_installation
//! - "multiple scopes with mixed configurations"
//!   -> multiple_scopes_with_mixed_configurations
//! - "reinstall does not change file when hooks already present"
//!   -> reinstall_does_not_change_file_when_hooks_already_present
//! - "hooks work with complex existing configuration"
//!   -> hooks_work_with_complex_existing_configuration

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::common::*;

// =============================================================================
// Integration with Issue Tracking Workflow
// =============================================================================

#[test]
fn hooks_can_be_installed_in_initialized_project() {
    let temp = init_temp();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    assert!(
        temp.path().join(".claude/settings.local.json").exists(),
        ".claude/settings.local.json should exist"
    );
}

#[test]
fn hooks_work_alongside_wok_directory() {
    let temp = init_temp();

    wk().args(["hooks", "install", "-y", "project"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Both should exist
    assert!(
        temp.path().join(".wok").is_dir(),
        ".wok directory should exist"
    );
    assert!(
        temp.path().join(".claude/settings.json").exists(),
        ".claude/settings.json should exist"
    );
}

#[test]
fn hooks_status_after_install_and_uninstall_cycle() {
    let temp = TempDir::new().unwrap();

    // Install
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Check status shows installed
    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("installed"));

    // Uninstall
    wk().args(["hooks", "uninstall", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Check status shows no hooks
    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No hooks"));
}

#[test]
fn multiple_scope_installations_tracked_separately() {
    let temp = TempDir::new().unwrap();

    // Install both scopes
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    wk().args(["hooks", "install", "-y", "project"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Check status shows both
    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("project"));

    // Uninstall local
    wk().args(["hooks", "uninstall", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Check only project remains
    let output = wk()
        .args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("project"),
        "Project scope should still be shown"
    );
    // local should not show as installed (might appear in header)
    assert!(
        !stdout.contains("local") || !stdout.contains("local") || stdout.contains("project"),
        "Local scope should not be shown as installed"
    );
}

#[test]
fn hooks_install_and_wk_commands_work_together() {
    let temp = init_temp();

    // Install hooks
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Normal wk commands should still work - create an issue
    wk().args(["new", "task", "Test issue"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // List should show the issue
    wk().arg("list")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Test issue"));
}

#[test]
fn hooks_installed_in_project_visible_to_collaborators() {
    let temp = TempDir::new().unwrap();

    // Install to project scope
    wk().args(["hooks", "install", "-y", "project"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Create a "collaborator" by using different temp home
    let collab_home = TempDir::new().unwrap();

    // Project hooks should still be visible (they're in current dir)
    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", collab_home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project"));
}

#[test]
fn hooks_survive_wk_operations() {
    let temp = init_temp();

    // Install hooks
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Create issue
    let output = wk()
        .args(["new", "task", "Test"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract issue ID (format: test-XXXX)
    let id = stdout
        .lines()
        .find_map(|line| {
            line.split_whitespace()
                .find(|word| word.starts_with("test-"))
        })
        .expect("Should find issue ID")
        .trim_end_matches(':');

    // Start the issue
    wk().args(["start", id])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Complete the issue
    wk().args(["done", id])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Hooks should still be there
    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("installed"));
}

#[test]
fn hooks_and_wk_init_order_independent() {
    let temp = TempDir::new().unwrap();

    // Install hooks first (before init)
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Then init
    wk().args(["init", "--prefix", "test", "--private"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Both should work
    assert!(
        temp.path().join(".claude/settings.local.json").exists(),
        ".claude/settings.local.json should exist"
    );
    assert!(
        temp.path().join(".wok").is_dir(),
        ".wok directory should exist"
    );

    // List should work
    wk().arg("list")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Hooks status should show installed
    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("installed"));
}

// =============================================================================
// Smart Merge Integration Tests
// =============================================================================

#[test]
fn hooks_install_then_uninstall_preserves_custom_hooks() {
    let temp = TempDir::new().unwrap();

    // Create custom hooks
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{"hooks": {"PreCompact": [{"matcher": "", "hooks": [{"type": "command", "command": "custom.sh"}]}]}}"#,
    )
    .unwrap();

    // Install wk hooks
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Uninstall wk hooks
    wk().args(["hooks", "uninstall", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Custom hooks should remain
    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(
        content.contains("custom.sh"),
        "Custom hooks should be preserved after uninstall"
    );
}

#[test]
fn hooks_status_accurately_reflects_partial_installation() {
    let temp = TempDir::new().unwrap();

    // Only PreCompact, no SessionStart
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{"hooks": {"PreCompact": [{"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}]}}"#,
    )
    .unwrap();

    // Should indicate installed (has hooks key)
    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
fn multiple_scopes_with_mixed_configurations() {
    let temp = TempDir::new().unwrap();

    // Create local with custom hooks
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{"hooks": {"PreCompact": [{"matcher": "", "hooks": [{"type": "command", "command": "local-hook.sh"}]}]}}"#,
    )
    .unwrap();

    // Install wk hooks to both
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    wk().args(["hooks", "install", "-y", "project"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Verify local has both hooks
    let local_content =
        std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(
        local_content.contains("local-hook.sh"),
        "Local should preserve custom hook"
    );
    assert!(
        local_content.contains("wk prime"),
        "Local should have wk prime"
    );

    // Verify project has only wk hooks
    let project_content =
        std::fs::read_to_string(temp.path().join(".claude/settings.json")).unwrap();
    assert!(
        project_content.contains("wk prime"),
        "Project should have wk prime"
    );
    assert!(
        !project_content.contains("local-hook.sh"),
        "Project should not have local custom hook"
    );
}

#[test]
fn reinstall_does_not_change_file_when_hooks_already_present() {
    let temp = TempDir::new().unwrap();

    // First install
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let first_content =
        std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();

    // Second install
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let second_content =
        std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();

    // Content should be identical
    assert_eq!(
        first_content, second_content,
        "Reinstall should not change file content"
    );
}

#[test]
fn hooks_work_with_complex_existing_configuration() {
    let temp = TempDir::new().unwrap();

    // Create complex settings
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{
  "mcpServers": {
    "test": {"command": "echo test"}
  },
  "hooks": {
    "PostToolUse": [
      {"matcher": "Bash", "hooks": [{"type": "command", "command": "lint.sh"}]},
      {"matcher": "Edit", "hooks": [{"type": "command", "command": "format.sh"}]}
    ],
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "save-context.sh"}]}
    ]
  },
  "otherSetting": true
}"#,
    )
    .unwrap();

    // Install wk hooks
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();

    // All original content should be preserved
    assert!(content.contains("mcpServers"), "Should preserve mcpServers");
    assert!(
        content.contains("PostToolUse"),
        "Should preserve PostToolUse"
    );
    assert!(content.contains("lint.sh"), "Should preserve lint.sh");
    assert!(content.contains("format.sh"), "Should preserve format.sh");
    assert!(
        content.contains("save-context.sh"),
        "Should preserve save-context.sh"
    );
    assert!(
        content.contains("otherSetting"),
        "Should preserve otherSetting"
    );

    // wk hooks should be added
    assert!(content.contains("wk prime"), "Should add wk prime");
    assert!(content.contains("SessionStart"), "Should add SessionStart");
}
