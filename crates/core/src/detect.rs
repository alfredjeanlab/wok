// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Detection of AI coding assistant parent processes and terminal state.
//!
//! This module provides utilities to detect if the current process
//! is running as a subprocess of AI coding assistants like Claude Code,
//! Codex, Aider, or similar tools. It also provides terminal state detection
//! such as whether the process is running in the foreground.

#[cfg(test)]
#[path = "detect_tests.rs"]
mod tests;

/// Known AI coding assistant indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiAssistant {
    /// Anthropic's Claude Code CLI
    ClaudeCode,
    /// OpenAI's Codex assistant
    Codex,
    /// Aider open source AI coding assistant
    Aider,
    /// Cursor VS Code fork with AI
    Cursor,
    /// Unknown or unrecognized AI assistant
    Unknown,
}

/// Detects if running under an AI coding assistant.
///
/// Checks environment variables commonly set by AI assistants to identify
/// which assistant (if any) is running the current process.
///
/// Returns `Some(assistant)` if detected, `None` otherwise.
pub fn detect_ai_assistant() -> Option<AiAssistant> {
    check_env_indicators()
}

/// Returns true if running under any AI coding assistant.
pub fn is_ai_subprocess() -> bool {
    detect_ai_assistant().is_some()
}

/// Check if this process is running in the foreground of the terminal.
///
/// A backgrounded process (e.g., `cmd &`) should not attempt interactive
/// terminal operations. This function checks if the process's process group
/// matches the terminal's foreground process group.
///
/// Returns `true` if we're in the foreground, `false` if backgrounded.
/// On non-Unix systems or if the check fails, returns `true` (optimistic default).
#[cfg(unix)]
pub fn is_foreground_process() -> bool {
    use std::os::unix::io::AsFd;

    // Get stdin file descriptor
    let stdin = std::io::stdin();
    let fd = stdin.as_fd();

    // tcgetpgrp returns Err if fd is not a controlling terminal.
    let foreground_pgrp = match nix::unistd::tcgetpgrp(fd) {
        Ok(pid) => pid,
        Err(_) => {
            // Not a controlling terminal - we cannot determine foreground status,
            // so return false to prevent interactive operations that would hang.
            return false;
        }
    };

    let my_pgrp = nix::unistd::getpgrp();

    foreground_pgrp == my_pgrp
}

#[cfg(not(unix))]
pub fn is_foreground_process() -> bool {
    // On non-Unix systems, assume we're in the foreground
    true
}

/// Returns true if the current process is an interactive human session.
///
/// Criteria:
/// - stdout is a TTY
/// - Not running under an AI assistant (Claude Code, Codex, etc.)
/// - Not in CI environment
pub fn is_human_interactive() -> bool {
    use std::io::IsTerminal;

    // Must be a TTY
    if !std::io::stdout().is_terminal() {
        return false;
    }

    // Must not be an AI subprocess
    if is_ai_subprocess() {
        return false;
    }

    // Must not be CI
    if std::env::var_os("CI").is_some() {
        return false;
    }

    true
}

/// Check environment variables for AI assistant indicators.
fn check_env_indicators() -> Option<AiAssistant> {
    // Claude Code indicators
    if std::env::var_os("CLAUDE_CODE").is_some() || std::env::var_os("CLAUDE_CODE_ENTRY").is_some()
    {
        return Some(AiAssistant::ClaudeCode);
    }

    // OpenAI Codex indicators
    if std::env::var_os("CODEX_ENV").is_some() {
        return Some(AiAssistant::Codex);
    }

    // Aider indicators
    if std::env::var_os("AIDER_MODEL").is_some() {
        return Some(AiAssistant::Aider);
    }

    // Cursor indicators
    if std::env::var_os("CURSOR_TRACE_ID").is_some() {
        return Some(AiAssistant::Cursor);
    }

    None
}
