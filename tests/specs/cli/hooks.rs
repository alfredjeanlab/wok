// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hooks command tests - converted from tests/specs/cli/unit/hooks.bats
//!
//! BATS test mapping:
//! - "hooks help and documentation"
//!   -> help_and_documentation tests
//! - "hooks install -y creates settings for each scope"
//!   -> install_creates_settings_for_scope (parameterized)
//! - "hooks install -y is idempotent and preserves existing settings"
//!   -> install_is_idempotent, install_preserves_existing_settings
//! - "hooks uninstall removes hooks and preserves other settings"
//!   -> uninstall_* tests
//! - "hooks status shows installation state"
//!   -> status_* tests
//! - "hooks install auto-detects non-interactive mode"
//!   -> auto_detect_* (parameterized)
//! - "hooks install error handling"
//!   -> error_handling_* tests
//! - "hooks install creates valid JSON with PreCompact and wk prime"
//!   -> install_creates_valid_json
//! - "hooks install does not hang in non-TTY or CI"
//!   -> non_tty_does_not_hang tests
//! - "hooks work without wk init"
//!   -> works_without_wk_init tests
//! - "hooks install smart merge preserves existing hooks"
//!   -> smart_merge_* tests
//! - "hooks uninstall only removes wk hooks preserving others"
//!   -> uninstall_only_removes_wk_hooks
//! - "hooks install detects wk prime with full path or args"
//!   -> detects_wk_prime_variants tests

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use std::time::Duration;

// =============================================================================
// Help and Documentation Tests
// From: "hooks help and documentation"
// =============================================================================

#[test]
fn hooks_shows_help_with_no_subcommand() {
    let temp = TempDir::new().unwrap();

    // When no subcommand is provided, help goes to stderr (exit code 2)
    wk().arg("hooks")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .timeout(Duration::from_secs(3))
        .assert()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn hooks_install_help_shows_scopes() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "--help"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("project"))
        .stdout(predicate::str::contains("user"));
}

#[test]
fn hooks_short_help_works() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "-h"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hooks"));
}

// =============================================================================
// Install Creates Settings for Each Scope
// From: "hooks install -y creates settings for each scope"
// =============================================================================

#[yare::parameterized(
    default_local = { &[], ".claude/settings.local.json" },
    explicit_local = { &["local"], ".claude/settings.local.json" },
    project = { &["project"], ".claude/settings.json" },
)]
fn install_creates_settings_for_scope(scope_args: &[&str], expected_file: &str) {
    let temp = TempDir::new().unwrap();

    let mut args = vec!["hooks", "install", "-y"];
    args.extend(scope_args.iter());

    wk().args(&args).current_dir(temp.path()).env("HOME", temp.path()).assert().success();

    let file_path = temp.path().join(expected_file);
    assert!(file_path.exists(), "Expected {} to exist", expected_file);

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("\"hooks\""), "Settings file should contain hooks");
}

#[test]
fn install_user_creates_home_settings() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "-y", "user"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let file_path = temp.path().join(".claude/settings.json");
    assert!(file_path.exists(), "Expected ~/.claude/settings.json to exist");

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("\"hooks\""), "Settings file should contain hooks");
}

// =============================================================================
// Idempotent Install and Preserving Settings
// From: "hooks install -y is idempotent and preserves existing settings"
// =============================================================================

#[test]
fn install_is_idempotent() {
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

    assert_eq!(first_content, second_content, "Install should be idempotent");
}

#[test]
fn install_preserves_existing_settings() {
    let temp = TempDir::new().unwrap();

    // Create existing settings with mcpServers
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{"mcpServers": {"test": {}}}"#,
    )
    .unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(content.contains("\"hooks\""), "Should add hooks");
    assert!(content.contains("\"mcpServers\""), "Should preserve mcpServers");
}

// =============================================================================
// Uninstall Tests
// From: "hooks uninstall removes hooks and preserves other settings"
// =============================================================================

#[test]
fn uninstall_removes_hooks() {
    let temp = TempDir::new().unwrap();

    // Install then uninstall
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    wk().args(["hooks", "uninstall", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let file_path = temp.path().join(".claude/settings.local.json");
    if file_path.exists() {
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("\"PreCompact\""), "PreCompact should be removed");
    }
}

#[test]
fn uninstall_preserves_other_settings() {
    let temp = TempDir::new().unwrap();

    // Create settings with hooks and other settings
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{"mcpServers": {"test": {}}, "hooks": {"PreCompact": []}}"#,
    )
    .unwrap();

    wk().args(["hooks", "uninstall", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(content.contains("\"mcpServers\""), "Should preserve mcpServers");
}

#[test]
fn uninstall_on_nonexistent_succeeds() {
    let temp = TempDir::new().unwrap();

    // No .claude directory exists
    wk().args(["hooks", "uninstall", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn uninstall_rejects_y_flag() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "uninstall", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

// =============================================================================
// Status Tests
// From: "hooks status shows installation state"
// =============================================================================

#[test]
fn status_shows_no_hooks_when_none_installed() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No hooks installed"));
}

#[test]
fn status_shows_installed_scope() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("installed"));
}

#[test]
fn status_shows_multiple_scopes() {
    let temp = TempDir::new().unwrap();

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

    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("project"));
}

// =============================================================================
// Auto-Detect Non-Interactive Mode
// From: "hooks install auto-detects non-interactive mode"
// =============================================================================

#[yare::parameterized(
    claude_code_env = { "CLAUDE_CODE", "1" },
    codex_env = { "CODEX_ENV", "1" },
    aider_model_env = { "AIDER_MODEL", "gpt-4" },
)]
fn auto_detect_non_interactive_via_env(env_var: &str, env_val: &str) {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env(env_var, env_val)
        .timeout(Duration::from_secs(3))
        .assert()
        .success();

    assert!(
        temp.path().join(".claude/settings.local.json").exists(),
        "Settings file should be created with {} env",
        env_var
    );
}

#[test]
fn auto_detect_non_interactive_via_stdin() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .write_stdin("")
        .timeout(Duration::from_secs(3))
        .assert()
        .success();

    assert!(
        temp.path().join(".claude/settings.local.json").exists(),
        "Settings file should be created with piped stdin"
    );
}

// =============================================================================
// Error Handling
// From: "hooks install error handling"
// =============================================================================

#[test]
fn install_rejects_invalid_scope() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "-y", "invalid"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn uninstall_rejects_invalid_scope() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "uninstall", "invalid"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
fn install_fails_on_permission_error() {
    let temp = TempDir::new().unwrap();

    // Create .claude directory with read-only permissions
    let claude_dir = temp.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&claude_dir, std::fs::Permissions::from_mode(0o444)).unwrap();
    }

    let result = wk()
        .args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .failure();

    // Check for permission error in output
    let output = result.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("permission"),
        "Should mention permission error: {}",
        stderr
    );

    // Restore permissions for cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&claude_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[test]
fn install_rejects_both_i_and_y_flags() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "-i", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

// =============================================================================
// Valid JSON Creation
// From: "hooks install creates valid JSON with PreCompact and wk prime"
// =============================================================================

#[test]
fn install_creates_valid_json_with_precompact_and_wk_prime() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();

    // Contains PreCompact
    assert!(content.contains("\"PreCompact\""), "Should contain PreCompact");

    // Valid JSON - parse it
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("Should be valid JSON");
    assert!(parsed.is_object(), "Should be a JSON object");

    // References wk prime command
    assert!(content.contains("wk prime"), "Should reference wk prime");
}

// =============================================================================
// Non-TTY / CI Tests
// From: "hooks install does not hang in non-TTY or CI"
// =============================================================================

#[test]
fn install_does_not_hang_without_scope_in_non_tty() {
    let temp = TempDir::new().unwrap();

    // Without scope, in non-TTY (piped stdin), should not hang
    // Status code is not checked - just verifying it doesn't hang
    let _ = wk()
        .args(["hooks", "install"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .write_stdin("")
        .timeout(Duration::from_secs(3))
        .assert();
}

#[test]
fn install_works_in_ci_environment() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "install", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env("CI", "true")
        .env("GITHUB_ACTIONS", "true")
        .timeout(Duration::from_secs(3))
        .assert()
        .success();

    assert!(temp.path().join(".claude/settings.local.json").exists());
}

// =============================================================================
// Works Without wk init
// From: "hooks work without wk init"
// =============================================================================

#[test]
fn install_works_without_wk_init() {
    let temp = TempDir::new().unwrap();
    // No wk init - just run hooks install

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".claude/settings.local.json").exists());
}

#[test]
fn status_works_without_wk_init() {
    let temp = TempDir::new().unwrap();

    wk().args(["hooks", "status"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();
}

// =============================================================================
// Smart Merge Tests
// From: "hooks install smart merge preserves existing hooks"
// =============================================================================

#[test]
fn smart_merge_preserves_existing_non_wk_hooks() {
    let temp = TempDir::new().unwrap();

    // Create settings with existing custom hooks
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "custom-script.sh"}]}
    ]
  }
}"#,
    )
    .unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(content.contains("custom-script.sh"), "Should preserve custom hooks");
    assert!(content.contains("wk prime"), "Should add wk prime");
}

#[test]
fn smart_merge_does_not_duplicate_wk_hooks() {
    let temp = TempDir::new().unwrap();

    // First install
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Second install
    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    let count = content.matches("wk prime").count();
    // wk prime appears in PreCompact and SessionStart (2 occurrences expected)
    assert_eq!(count, 2, "Should not duplicate wk prime hooks");
}

#[test]
fn smart_merge_adds_missing_events() {
    let temp = TempDir::new().unwrap();

    // Create settings with only PreCompact
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
    ]
  }
}"#,
    )
    .unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(content.contains("\"SessionStart\""), "Should add SessionStart event");
}

#[test]
fn smart_merge_preserves_hooks_on_other_events() {
    let temp = TempDir::new().unwrap();

    // Create settings with hooks on other events
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{
  "hooks": {
    "PostToolUse": [
      {"matcher": "", "hooks": [{"type": "command", "command": "my-hook.sh"}]}
    ]
  }
}"#,
    )
    .unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(content.contains("PostToolUse"), "Should preserve PostToolUse");
    assert!(content.contains("my-hook.sh"), "Should preserve my-hook.sh");
}

// =============================================================================
// Uninstall Only Removes wk Hooks
// From: "hooks uninstall only removes wk hooks preserving others"
// =============================================================================

#[test]
fn uninstall_only_removes_wk_hooks() {
    let temp = TempDir::new().unwrap();

    // Create settings with both custom and wk hooks
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "custom.sh"}]},
      {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
    ]
  }
}"#,
    )
    .unwrap();

    wk().args(["hooks", "uninstall", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    assert!(content.contains("custom.sh"), "Should preserve custom.sh");
    assert!(!content.contains("wk prime"), "Should remove wk prime");
}

// =============================================================================
// Detects wk prime Variants
// From: "hooks install detects wk prime with full path or args"
// =============================================================================

#[test]
fn detects_wk_prime_with_full_path() {
    let temp = TempDir::new().unwrap();

    // Create settings with full path to wk prime
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "/usr/local/bin/wk prime"}]}
    ]
  }
}"#,
    )
    .unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    // Should not duplicate - count occurrences of "wk prime"
    let count = content.matches("wk prime").count();
    assert_eq!(count, 2, "Should recognize /usr/local/bin/wk prime as existing hook");
}

#[test]
fn detects_wk_prime_with_args() {
    let temp = TempDir::new().unwrap();

    // Create settings with wk prime --verbose
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
    std::fs::write(
        temp.path().join(".claude/settings.local.json"),
        r#"{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "wk prime --verbose"}]}
    ]
  }
}"#,
    )
    .unwrap();

    wk().args(["hooks", "install", "-y", "local"])
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join(".claude/settings.local.json")).unwrap();
    // Should detect "wk prime --verbose" as wk prime and not duplicate PreCompact
    let count = content.matches("PreCompact").count();
    assert_eq!(count, 1, "Should only have one PreCompact entry, got content: {}", content);
}
