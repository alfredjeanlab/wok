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

#[test]
fn detects_codex_via_env() {
    // Save current value if any
    let prev = std::env::var_os("CODEX_ENV");
    std::env::set_var("CODEX_ENV", "1");
    let result = detect_ai_assistant();
    // Restore
    match prev {
        Some(v) => std::env::set_var("CODEX_ENV", v),
        None => std::env::remove_var("CODEX_ENV"),
    }
    assert_eq!(result, Some(AiAssistant::Codex));
}

#[test]
fn detects_aider_via_env() {
    let prev = std::env::var_os("AIDER_MODEL");
    std::env::set_var("AIDER_MODEL", "gpt-4");
    let result = detect_ai_assistant();
    match prev {
        Some(v) => std::env::set_var("AIDER_MODEL", v),
        None => std::env::remove_var("AIDER_MODEL"),
    }
    assert_eq!(result, Some(AiAssistant::Aider));
}

#[test]
fn detects_cursor_via_env() {
    let prev = std::env::var_os("CURSOR_TRACE_ID");
    std::env::set_var("CURSOR_TRACE_ID", "trace123");
    let result = detect_ai_assistant();
    match prev {
        Some(v) => std::env::set_var("CURSOR_TRACE_ID", v),
        None => std::env::remove_var("CURSOR_TRACE_ID"),
    }
    assert_eq!(result, Some(AiAssistant::Cursor));
}

#[test]
fn is_human_interactive_false_when_ai_env_set() {
    let prev = std::env::var_os("CLAUDE_CODE");
    std::env::set_var("CLAUDE_CODE", "1");
    let result = is_human_interactive();
    match prev {
        Some(v) => std::env::set_var("CLAUDE_CODE", v),
        None => std::env::remove_var("CLAUDE_CODE"),
    }
    assert!(!result);
}

#[test]
fn is_human_interactive_false_in_ci() {
    let prev = std::env::var_os("CI");
    std::env::set_var("CI", "true");
    let result = is_human_interactive();
    match prev {
        Some(v) => std::env::set_var("CI", v),
        None => std::env::remove_var("CI"),
    }
    assert!(!result);
}

#[test]
fn is_human_interactive_returns_bool() {
    // Should return a boolean without panicking
    let result = is_human_interactive();
    assert!(result || !result);
}
