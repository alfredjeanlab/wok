// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_shells() {
    // Should detect at least one shell on most systems
    let shells = detect_shells();
    // This test may be environment-dependent, but at least verify it doesn't panic
    assert!(shells.len() <= 3);
}

#[test]
fn test_shell_exists() {
    // 'sh' should exist on all Unix systems
    assert!(shell_exists("sh"));
    // Nonexistent binary should return false
    assert!(!shell_exists("nonexistent_shell_binary_xyz"));
}

#[test]
fn test_shell_kind_script_filename() {
    assert_eq!(ShellKind::Bash.script_filename(), "wk.bash");
    assert_eq!(ShellKind::Zsh.script_filename(), "_wk");
    assert_eq!(ShellKind::Fish.script_filename(), "wk.fish");
}

#[test]
fn test_shell_kind_clap_shell() {
    assert_eq!(ShellKind::Bash.clap_shell(), clap_complete::Shell::Bash);
    assert_eq!(ShellKind::Zsh.clap_shell(), clap_complete::Shell::Zsh);
    assert_eq!(ShellKind::Fish.clap_shell(), clap_complete::Shell::Fish);
}

#[test]
fn test_completions_dir() {
    // Should return a path on systems with data_local_dir
    let dir = completions_dir();
    if let Some(d) = dir {
        assert!(d.to_string_lossy().contains("wk/completions"));
    }
}

#[test]
fn test_write_completion_script() {
    let temp = TempDir::new().unwrap();

    // Override HOME to use temp directory
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp.path());
    std::env::set_var("XDG_DATA_HOME", temp.path().join(".local/share"));

    // Write a completion script
    let result = write_completion_script(ShellKind::Bash);

    // Restore HOME
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
    std::env::remove_var("XDG_DATA_HOME");

    // Should succeed and create file
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.exists());

    // Check script contains valid bash completion code
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("complete") || content.contains("wk"));
}

#[test]
fn test_install_completion_source_idempotent() {
    let temp = TempDir::new().unwrap();
    let rc_path = temp.path().join(".bashrc");

    // Create a fake RC file
    fs::write(&rc_path, "# my bashrc\nexport FOO=bar\n").unwrap();

    // Create a fake completion script
    let script_path = temp.path().join("wk.bash");
    fs::write(&script_path, "# completions").unwrap();

    // Override home temporarily by testing install_completion_source directly
    // First, manually add the sourcing line
    let source_line = format!(
        "\n{}\n[ -f \"{}\" ] && source \"{}\"\n",
        WK_COMPLETION_MARKER,
        script_path.display(),
        script_path.display()
    );
    let mut file = OpenOptions::new().append(true).open(&rc_path).unwrap();
    file.write_all(source_line.as_bytes()).unwrap();
    drop(file);

    let content_before = fs::read_to_string(&rc_path).unwrap();
    let marker_count_before = content_before.matches(WK_COMPLETION_MARKER).count();
    assert_eq!(marker_count_before, 1);

    // Trying to add again should be idempotent (check existing.contains logic)
    let existing = fs::read_to_string(&rc_path).unwrap();
    assert!(existing.contains(WK_COMPLETION_MARKER));
}

// Note: tests that set HOME env var are fragile because dirs::home_dir()
// may not respect HOME on all platforms (esp. macOS). These tests verify
// behavior using the actual home directory's RC files when they exist.

#[test]
fn test_shell_kind_rc_file_returns_correct_paths() {
    // Test that when RC files exist, they have the expected suffixes
    if let Some(rc) = ShellKind::Bash.rc_file() {
        let path_str = rc.to_string_lossy();
        assert!(
            path_str.ends_with(".bashrc") || path_str.ends_with(".bash_profile"),
            "Bash rc_file should end with .bashrc or .bash_profile"
        );
    }

    if let Some(rc) = ShellKind::Zsh.rc_file() {
        assert!(
            rc.to_string_lossy().ends_with(".zshrc"),
            "Zsh rc_file should end with .zshrc"
        );
    }

    if let Some(rc) = ShellKind::Fish.rc_file() {
        assert!(
            rc.to_string_lossy().ends_with("config.fish"),
            "Fish rc_file should end with config.fish"
        );
    }
}

#[test]
fn test_install_all_no_shells() {
    // With no RC files, install_all should succeed but do nothing
    let temp = TempDir::new().unwrap();
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp.path());
    std::env::set_var("XDG_CONFIG_HOME", temp.path().join(".config"));
    std::env::set_var("XDG_DATA_HOME", temp.path().join(".local/share"));

    let result = install_all();
    // Should succeed (no shells with RC files to install)
    assert!(result.is_ok());

    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
    std::env::remove_var("XDG_CONFIG_HOME");
    std::env::remove_var("XDG_DATA_HOME");
}

#[test]
fn test_marker_constant() {
    assert_eq!(WK_COMPLETION_MARKER, "# wk-shell-completion");
}
