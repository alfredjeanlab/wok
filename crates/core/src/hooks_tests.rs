// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::path::Path;
use tempfile::TempDir;

/// Helper to run tests in a temporary directory.
/// Uses absolute paths to avoid issues with current directory changes.
fn with_temp_dir<F>(f: F)
where
    F: FnOnce(&Path),
{
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path().to_path_buf();
    f(&temp_path);
    // temp is dropped here, cleaning up the directory
}

/// Install hooks using absolute paths.
fn install_hooks_at(base: &Path, scope: HookScope) -> io::Result<PathBuf> {
    let rel_path = scope.settings_path()?;
    let abs_path = base.join(&rel_path);

    // Ensure parent directory exists
    if let Some(parent) = abs_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read existing settings or start fresh
    let mut settings: serde_json::Value = if abs_path.exists() {
        let content = fs::read_to_string(&abs_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Smart merge wk hooks
    merge_wk_hooks(&mut settings);

    // Write back
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(&abs_path, content)?;

    Ok(abs_path)
}

/// Uninstall hooks using absolute paths.
fn uninstall_hooks_at(base: &Path, scope: HookScope) -> io::Result<()> {
    let rel_path = scope.settings_path()?;
    let abs_path = base.join(&rel_path);

    if !abs_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&abs_path)?;
    let mut settings: serde_json::Value =
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));

    // Smart remove wk hooks
    remove_wk_hooks(&mut settings);

    // If empty, remove the file; otherwise write back
    if settings.as_object().map_or(true, |o| o.is_empty()) {
        fs::remove_file(&abs_path)?;
    } else {
        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&abs_path, content)?;
    }

    Ok(())
}

/// Check hooks using absolute paths.
fn check_hooks_at(base: &Path, scope: HookScope) -> io::Result<HookStatus> {
    let rel_path = scope.settings_path()?;
    let abs_path = base.join(&rel_path);

    let installed = if abs_path.exists() {
        let content = fs::read_to_string(&abs_path)?;
        let settings: serde_json::Value =
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
        settings.get("hooks").is_some()
    } else {
        false
    };

    Ok(HookStatus {
        scope,
        installed,
        path: abs_path,
    })
}

#[test]
fn hook_scope_from_str_valid() {
    assert_eq!(HookScope::parse("local"), Some(HookScope::Local));
    assert_eq!(HookScope::parse("LOCAL"), Some(HookScope::Local));
    assert_eq!(HookScope::parse("project"), Some(HookScope::Project));
    assert_eq!(HookScope::parse("user"), Some(HookScope::User));
}

#[test]
fn hook_scope_from_str_invalid() {
    assert_eq!(HookScope::parse("invalid"), None);
    assert_eq!(HookScope::parse(""), None);
    assert_eq!(HookScope::parse("global"), None);
}

#[test]
fn hook_scope_display_name() {
    assert_eq!(HookScope::Local.display_name(), "local");
    assert_eq!(HookScope::Project.display_name(), "project");
    assert_eq!(HookScope::User.display_name(), "user");
}

#[test]
fn hook_scope_local_path() {
    let path = HookScope::Local.settings_path().unwrap();
    assert_eq!(path, PathBuf::from(".claude/settings.local.json"));
}

#[test]
fn hook_scope_project_path() {
    let path = HookScope::Project.settings_path().unwrap();
    assert_eq!(path, PathBuf::from(".claude/settings.json"));
}

#[test]
fn hook_scope_user_path() {
    let path = HookScope::User.settings_path().unwrap();
    assert!(path.ends_with(".claude/settings.json"));
    assert!(path.is_absolute());
}

#[test]
fn install_hooks_creates_file() {
    with_temp_dir(|base| {
        let path = install_hooks_at(base, HookScope::Local).unwrap();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("hooks"));
        assert!(content.contains("PreCompact"));
    });
}

#[test]
fn install_hooks_preserves_existing() {
    with_temp_dir(|base| {
        // Create existing settings
        let claude_dir = base.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join("settings.local.json"),
            r#"{"mcpServers": {}}"#,
        )
        .unwrap();

        install_hooks_at(base, HookScope::Local).unwrap();

        let content = fs::read_to_string(claude_dir.join("settings.local.json")).unwrap();
        assert!(content.contains("mcpServers"));
        assert!(content.contains("hooks"));
    });
}

#[test]
fn install_hooks_idempotent() {
    with_temp_dir(|base| {
        install_hooks_at(base, HookScope::Local).unwrap();
        let settings_path = base.join(".claude/settings.local.json");
        let first = fs::read_to_string(&settings_path).unwrap();

        install_hooks_at(base, HookScope::Local).unwrap();
        let second = fs::read_to_string(&settings_path).unwrap();

        assert_eq!(first, second);
    });
}

#[test]
fn uninstall_hooks_removes_hooks() {
    with_temp_dir(|base| {
        install_hooks_at(base, HookScope::Local).unwrap();
        uninstall_hooks_at(base, HookScope::Local).unwrap();

        // File should be removed (was only hooks)
        let settings_path = base.join(".claude/settings.local.json");
        assert!(!settings_path.exists());
    });
}

#[test]
fn uninstall_hooks_preserves_other_settings() {
    with_temp_dir(|base| {
        let claude_dir = base.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join("settings.local.json"),
            r#"{"mcpServers": {}, "hooks": {"PreCompact": []}}"#,
        )
        .unwrap();

        uninstall_hooks_at(base, HookScope::Local).unwrap();

        let content = fs::read_to_string(claude_dir.join("settings.local.json")).unwrap();
        assert!(content.contains("mcpServers"));
        assert!(!content.contains("hooks"));
    });
}

#[test]
fn uninstall_hooks_nonexistent_succeeds() {
    with_temp_dir(|base| {
        // Should not error
        uninstall_hooks_at(base, HookScope::Local).unwrap();
    });
}

#[test]
fn check_hooks_not_installed() {
    with_temp_dir(|base| {
        let status = check_hooks_at(base, HookScope::Local).unwrap();
        assert!(!status.installed);
    });
}

#[test]
fn check_hooks_installed() {
    with_temp_dir(|base| {
        install_hooks_at(base, HookScope::Local).unwrap();

        let status = check_hooks_at(base, HookScope::Local).unwrap();
        assert!(status.installed);
        assert_eq!(status.scope, HookScope::Local);
    });
}

#[test]
fn wk_hook_events_contains_expected_events() {
    assert!(WK_HOOK_EVENTS.contains(&"PreCompact"));
    assert!(WK_HOOK_EVENTS.contains(&"SessionStart"));
}

#[test]
fn is_wk_hook_detects_plain_wk_prime() {
    let hook = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "wk prime"}]
    });
    assert!(is_wk_hook(&hook));
}

#[test]
fn is_wk_hook_detects_full_path() {
    let hook = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "/usr/local/bin/wk prime"}]
    });
    assert!(is_wk_hook(&hook));
}

#[test]
fn is_wk_hook_detects_with_args() {
    let hook = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "wk prime --verbose"}]
    });
    assert!(is_wk_hook(&hook));
}

#[test]
fn is_wk_hook_ignores_other_commands() {
    let hook = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "custom-script.sh"}]
    });
    assert!(!is_wk_hook(&hook));
}

#[test]
fn is_wk_hook_ignores_partial_match() {
    let hook = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "my-wk-prime-like-thing"}]
    });
    // Should not match because it doesn't start with "wk prime" or contain "/wk prime"
    assert!(!is_wk_hook(&hook));
}

#[test]
fn is_wk_hook_handles_empty_hooks() {
    let hook = serde_json::json!({"matcher": "", "hooks": []});
    assert!(!is_wk_hook(&hook));
}

#[test]
fn is_wk_hook_handles_missing_hooks() {
    let hook = serde_json::json!({"matcher": ""});
    assert!(!is_wk_hook(&hook));
}

#[test]
fn is_wk_hook_handles_multiple_commands() {
    let hook = serde_json::json!({
        "matcher": "",
        "hooks": [
            {"type": "command", "command": "echo hello"},
            {"type": "command", "command": "wk prime"}
        ]
    });
    assert!(is_wk_hook(&hook));
}

#[test]
fn merge_preserves_custom_hooks() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "custom.sh"}]}
            ]
        }
    });
    merge_wk_hooks(&mut settings);

    let hooks = settings["hooks"]["PreCompact"].as_array().unwrap();
    // Should have both: custom hook and wk hook
    assert_eq!(hooks.len(), 2);
    assert!(hooks
        .iter()
        .any(|h| { h["hooks"][0]["command"].as_str() == Some("custom.sh") }));
    assert!(hooks.iter().any(is_wk_hook));
}

#[test]
fn merge_does_not_duplicate_wk_hooks() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ],
            "SessionStart": [
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ]
        }
    });
    merge_wk_hooks(&mut settings);

    // Should still have exactly 1 hook per event
    let precompact = settings["hooks"]["PreCompact"].as_array().unwrap();
    let sessionstart = settings["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(precompact.len(), 1);
    assert_eq!(sessionstart.len(), 1);
}

#[test]
fn merge_adds_missing_events() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ]
        }
    });
    merge_wk_hooks(&mut settings);

    // SessionStart should be added
    assert!(settings["hooks"]["SessionStart"].is_array());
    let sessionstart = settings["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(sessionstart.len(), 1);
}

#[test]
fn merge_handles_empty_hooks_object() {
    let mut settings = serde_json::json!({"hooks": {}});
    merge_wk_hooks(&mut settings);

    assert!(settings["hooks"]["PreCompact"].is_array());
    assert!(settings["hooks"]["SessionStart"].is_array());
}

#[test]
fn merge_handles_missing_hooks_object() {
    let mut settings = serde_json::json!({"mcpServers": {}});
    merge_wk_hooks(&mut settings);

    assert!(settings["hooks"]["PreCompact"].is_array());
    assert!(settings["hooks"]["SessionStart"].is_array());
}

#[test]
fn merge_preserves_hooks_on_other_events() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PostToolUse": [
                {"matcher": "", "hooks": [{"type": "command", "command": "my-hook.sh"}]}
            ]
        }
    });
    merge_wk_hooks(&mut settings);

    // PostToolUse should remain unchanged
    let posttooluse = settings["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(posttooluse.len(), 1);
    assert_eq!(posttooluse[0]["hooks"][0]["command"], "my-hook.sh");
}

#[test]
fn merge_detects_wk_prime_with_full_path() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "/usr/local/bin/wk prime"}]}
            ]
        }
    });
    merge_wk_hooks(&mut settings);

    // Should not add another wk hook to PreCompact
    let precompact = settings["hooks"]["PreCompact"].as_array().unwrap();
    assert_eq!(precompact.len(), 1);
}

#[test]
fn remove_only_removes_wk_hooks() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "custom.sh"}]},
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ]
        }
    });
    remove_wk_hooks(&mut settings);

    let hooks = settings["hooks"]["PreCompact"].as_array().unwrap();
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0]["hooks"][0]["command"], "custom.sh");
}

#[test]
fn remove_deletes_empty_event_arrays() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ],
            "SessionStart": [
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ]
        }
    });
    remove_wk_hooks(&mut settings);

    // Both events should be removed since they only had wk hooks
    assert!(settings["hooks"]["PreCompact"].is_null());
    assert!(settings["hooks"]["SessionStart"].is_null());
}

#[test]
fn remove_deletes_hooks_key_when_empty() {
    let mut settings = serde_json::json!({
        "mcpServers": {},
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ]
        }
    });
    remove_wk_hooks(&mut settings);

    // hooks key should be removed
    assert!(settings["hooks"].is_null());
    // mcpServers should remain
    assert!(settings["mcpServers"].is_object());
}

#[test]
fn remove_preserves_other_events() {
    let mut settings = serde_json::json!({
        "hooks": {
            "PreCompact": [
                {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
            ],
            "PostToolUse": [
                {"matcher": "", "hooks": [{"type": "command", "command": "my-hook.sh"}]}
            ]
        }
    });
    remove_wk_hooks(&mut settings);

    // PostToolUse should remain unchanged
    let posttooluse = settings["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(posttooluse.len(), 1);
}

#[test]
fn install_hooks_creates_parent_dirs() {
    with_temp_dir(|base| {
        // .claude doesn't exist
        let claude_dir = base.join(".claude");
        assert!(!claude_dir.exists());

        install_hooks_at(base, HookScope::Local).unwrap();

        assert!(claude_dir.exists());
        assert!(claude_dir.join("settings.local.json").exists());
    });
}

#[test]
fn check_all_hooks_returns_all_scopes() {
    // This test uses the actual functions which rely on current directory
    // Just verify the function runs without error
    let statuses = check_all_hooks();
    // Should have entries for local, project, and user (3 scopes)
    assert_eq!(statuses.len(), 3);
}

#[test]
fn install_hooks_handles_malformed_json() {
    with_temp_dir(|base| {
        let claude_dir = base.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("settings.local.json"), "not valid json {{{").unwrap();

        // Should succeed by overwriting with valid JSON
        let result = install_hooks_at(base, HookScope::Local);
        assert!(result.is_ok());

        // Verify result is valid JSON
        let content = fs::read_to_string(claude_dir.join("settings.local.json")).unwrap();
        let _: serde_json::Value = serde_json::from_str(&content).unwrap();
    });
}
