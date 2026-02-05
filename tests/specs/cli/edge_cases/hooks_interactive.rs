// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for interactive mode edge cases.
//! Converted from tests/specs/cli/edge_cases/hooks_interactive.bats
//!
//! These tests verify the interactive mode doesn't break non-interactive usage.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::super::common::*;
use std::process::{Command, Stdio};
use std::time::Duration;

#[cfg(unix)]
use libc;

/// Get the path to the wk binary for use with std::process::Command.
#[allow(deprecated)]
fn wk_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("wok")
}

/// Test that `hooks install -i` fails gracefully when not a TTY.
/// Force interactive mode on non-TTY (without scope) should error.
#[test]
fn hooks_interactive_fails_gracefully_when_not_a_tty() {
    let temp = init_temp();

    // Run with stdin from /dev/null to simulate non-TTY
    let output = Command::new(wk_bin())
        .args(["hooks", "install", "-i"])
        .current_dir(temp.path())
        .stdin(Stdio::null())
        .output()
        .expect("Failed to execute command");

    // Should fail (not hang)
    assert!(
        !output.status.success(),
        "Should fail when -i used without TTY"
    );

    // Should provide helpful error message mentioning terminal/TTY/interactive
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);
    let combined_lower = combined.to_lowercase();

    assert!(
        combined_lower.contains("terminal")
            || combined_lower.contains("tty")
            || combined_lower.contains("interactive"),
        "Error should mention terminal, TTY, or interactive. Got: {}",
        combined
    );
}

/// Test that `hooks install -i` with explicit scope skips the picker.
/// Even with -i, if scope is provided, no picker needed.
#[test]
fn hooks_interactive_with_explicit_scope_skips_picker() {
    let temp = init_temp();

    // Even with -i, if scope is provided, no picker needed
    // This might still work even on non-TTY
    let output = Command::new(wk_bin())
        .args(["hooks", "install", "-i", "local"])
        .current_dir(temp.path())
        .stdin(Stdio::null())
        .output()
        .expect("Failed to execute command");

    // Either succeeds or fails with clear message, doesn't hang
    // The key point is that we got a response (didn't timeout)
    // The command completed - success or failure is acceptable
    let _ = output.status; // Just verify we got here without hanging
}

/// Test that `hooks install` terminates cleanly on SIGINT.
#[cfg(unix)]
#[test]
fn hooks_install_terminates_cleanly_on_sigint() {
    let temp = init_temp();

    // Spawn the process
    let mut child = Command::new(wk_bin())
        .args(["hooks", "install"])
        .current_dir(temp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn command");

    // Give it a moment to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGINT using libc
    #[cfg(unix)]
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGINT);
    }

    // Wait with timeout
    let result = wait_with_timeout(&mut child, Duration::from_secs(2));

    // Test passes if process terminated (didn't hang)
    assert!(
        result.is_some(),
        "Process should terminate cleanly on SIGINT"
    );
}

/// Test that `hooks install` terminates cleanly on SIGTERM.
#[cfg(unix)]
#[test]
fn hooks_install_terminates_cleanly_on_sigterm() {
    let temp = init_temp();

    // Spawn the process
    let mut child = Command::new(wk_bin())
        .args(["hooks", "install"])
        .current_dir(temp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn command");

    // Give it a moment to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGTERM using libc
    #[cfg(unix)]
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
    }

    // Wait with timeout
    let result = wait_with_timeout(&mut child, Duration::from_secs(2));

    // Test passes if process terminated (didn't hang)
    assert!(
        result.is_some(),
        "Process should terminate cleanly on SIGTERM"
    );
}

/// Test that `hooks install` runs in non-interactive mode when backgrounded.
/// When run with explicit scope, should complete without hanging.
#[test]
fn hooks_install_runs_in_non_interactive_mode_when_backgrounded() {
    let temp = init_temp();

    // Run with explicit scope to avoid needing interactive picker
    let output = Command::new(wk_bin())
        .args(["hooks", "install", "local"])
        .current_dir(temp.path())
        .stdin(Stdio::null())
        .output()
        .expect("Failed to execute command");

    // Should complete without hanging - success or clear failure
    // The key point is that the command terminated
    let _ = output.status; // Just verify we got here without hanging
}

/// Helper to wait for a child process with timeout.
#[cfg(unix)]
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Option<std::process::ExitStatus> {
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {
                if start.elapsed() > timeout {
                    // Kill and return None to indicate timeout
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }
}
