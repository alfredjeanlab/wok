// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::bool_assert_comparison)]

use super::*;

#[test]
fn is_foreground_process_returns_bool() {
    // is_foreground_process should return a boolean without panicking
    let result = is_foreground_process();
    // In test environment, we're typically running in foreground
    assert!(result == true || result == false);
}

#[test]
fn detect_returns_valid_result() {
    // In a test environment, detect_ai_assistant should return a valid Option
    // It may return Some (if running under an AI assistant) or None
    let result = detect_ai_assistant();
    // Verify it's a valid Option - the function should not panic
    let _ = result;
}

#[test]
fn is_ai_subprocess_returns_bool() {
    // is_ai_subprocess should always return a boolean without panicking
    let result = is_ai_subprocess();
    assert!(result || !result);
}

#[test]
fn ai_assistant_enum_is_debug() {
    // Verify Debug trait is implemented
    let assistant = AiAssistant::ClaudeCode;
    let debug_str = format!("{:?}", assistant);
    assert!(!debug_str.is_empty());
}

#[test]
fn ai_assistant_enum_is_clone() {
    // Verify Clone trait is implemented
    let assistant = AiAssistant::Codex;
    let cloned = assistant;
    assert_eq!(assistant, cloned);
}

#[test]
fn ai_assistant_enum_variants_are_distinct() {
    // Verify all enum variants are distinct
    assert_ne!(AiAssistant::ClaudeCode, AiAssistant::Codex);
    assert_ne!(AiAssistant::ClaudeCode, AiAssistant::Aider);
    assert_ne!(AiAssistant::ClaudeCode, AiAssistant::Cursor);
    assert_ne!(AiAssistant::ClaudeCode, AiAssistant::Unknown);
    assert_ne!(AiAssistant::Codex, AiAssistant::Aider);
    assert_ne!(AiAssistant::Codex, AiAssistant::Cursor);
    assert_ne!(AiAssistant::Codex, AiAssistant::Unknown);
    assert_ne!(AiAssistant::Aider, AiAssistant::Cursor);
    assert_ne!(AiAssistant::Aider, AiAssistant::Unknown);
    assert_ne!(AiAssistant::Cursor, AiAssistant::Unknown);
}

/// Helper to save and restore environment variables
fn with_env_vars<F, R>(vars: &[&str], f: F) -> R
where
    F: FnOnce() -> R,
{
    // Save current values
    let saved: Vec<_> = vars.iter().map(|v| (*v, std::env::var_os(v))).collect();

    // Clear all vars
    for var in vars {
        std::env::remove_var(var);
    }

    let result = f();

    // Restore all vars
    for (var, value) in saved {
        match value {
            Some(v) => std::env::set_var(var, v),
            None => std::env::remove_var(var),
        }
    }

    result
}

/// Combined test for all environment-based detection to avoid race conditions.
/// These tests must run sequentially since they modify shared environment variables.
#[test]
fn env_detection_tests() {
    let ai_vars = &[
        "CLAUDE_CODE",
        "CLAUDE_CODE_ENTRY",
        "CODEX_ENV",
        "AIDER_MODEL",
        "CURSOR_TRACE_ID",
    ];

    // Test: detects Codex via CODEX_ENV
    with_env_vars(ai_vars, || {
        std::env::set_var("CODEX_ENV", "1");
        assert_eq!(detect_ai_assistant(), Some(AiAssistant::Codex));
    });

    // Test: detects Aider via AIDER_MODEL
    with_env_vars(ai_vars, || {
        std::env::set_var("AIDER_MODEL", "gpt-4");
        assert_eq!(detect_ai_assistant(), Some(AiAssistant::Aider));
    });

    // Test: detects Cursor via CURSOR_TRACE_ID
    with_env_vars(ai_vars, || {
        std::env::set_var("CURSOR_TRACE_ID", "trace123");
        assert_eq!(detect_ai_assistant(), Some(AiAssistant::Cursor));
    });

    // Test: is_human_interactive returns false when AI env is set
    with_env_vars(ai_vars, || {
        std::env::set_var("CLAUDE_CODE", "1");
        assert!(!is_human_interactive());
    });

    // Test: is_human_interactive returns false in CI
    with_env_vars(&["CI"], || {
        std::env::set_var("CI", "true");
        assert!(!is_human_interactive());
    });
}

#[test]
fn is_human_interactive_returns_bool() {
    // Should return a boolean without panicking
    let result = is_human_interactive();
    assert!(result || !result);
}
