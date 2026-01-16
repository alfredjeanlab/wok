// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// Log command
#[test]
fn test_log_without_id() {
    let cli = parse(&["wk", "log"]).unwrap();
    match cli.command {
        Command::Log { id, limit } => {
            assert!(id.is_none());
            assert_eq!(limit, 20); // default
        }
        _ => panic!("Expected Log command"),
    }
}

#[test]
fn test_log_with_id() {
    let cli = parse(&["wk", "log", "prj-1234"]).unwrap();
    match cli.command {
        Command::Log { id, .. } => {
            assert_eq!(id, Some("prj-1234".to_string()));
        }
        _ => panic!("Expected Log command"),
    }
}

#[test]
fn test_log_with_limit() {
    let cli = parse(&["wk", "log", "--limit", "50"]).unwrap();
    match cli.command {
        Command::Log { limit, .. } => {
            assert_eq!(limit, 50);
        }
        _ => panic!("Expected Log command"),
    }
}

#[test]
fn test_log_with_limit_short_flag() {
    let cli = parse(&["wk", "log", "-n", "25"]).unwrap();
    match cli.command {
        Command::Log { limit, .. } => {
            assert_eq!(limit, 25);
        }
        _ => panic!("Expected Log command"),
    }
}

#[test]
fn test_log_rejects_l_shorthand() {
    // -l short flag was removed from 'log' command
    let result = parse(&["wk", "log", "-l", "50"]);
    assert!(result.is_err());
}

// Export command
#[test]
fn test_export_command() {
    let cli = parse(&["wk", "export", "/tmp/issues.jsonl"]).unwrap();
    match cli.command {
        Command::Export { filepath } => {
            assert_eq!(filepath, "/tmp/issues.jsonl");
        }
        _ => panic!("Expected Export command"),
    }
}

// Import command tests
#[test]
fn test_import_with_file() {
    let cli = parse(&["wk", "import", "issues.jsonl"]).unwrap();
    match cli.command {
        Command::Import {
            file,
            input,
            prefix,
            ..
        } => {
            assert_eq!(file, Some("issues.jsonl".to_string()));
            assert!(input.is_none());
            assert!(prefix.is_none());
        }
        _ => panic!("Expected Import command"),
    }
}

#[test]
fn test_import_with_input_flag() {
    let cli = parse(&["wk", "import", "--input", "issues.jsonl"]).unwrap();
    match cli.command {
        Command::Import { file, input, .. } => {
            assert!(file.is_none());
            assert_eq!(input, Some("issues.jsonl".to_string()));
        }
        _ => panic!("Expected Import command"),
    }
}

#[test]
fn test_import_with_prefix_flag() {
    let cli = parse(&["wk", "import", "--prefix", "myproj", "issues.jsonl"]).unwrap();
    match cli.command {
        Command::Import { file, prefix, .. } => {
            assert_eq!(file, Some("issues.jsonl".to_string()));
            assert_eq!(prefix, Some("myproj".to_string()));
        }
        _ => panic!("Expected Import command"),
    }
}

#[test]
fn test_import_rejects_i_shorthand() {
    // -i short flag was removed from 'import' command
    let result = parse(&["wk", "import", "-i", "issues.jsonl"]);
    assert!(result.is_err());
}

#[test]
fn test_import_rejects_p_shorthand() {
    // -p short flag was removed from 'import' command
    let result = parse(&["wk", "import", "-p", "myproj", "issues.jsonl"]);
    assert!(result.is_err());
}

// Completion command
#[test]
fn test_completion_bash() {
    let cli = parse(&["wk", "completion", "bash"]).unwrap();
    match cli.command {
        Command::Completion { shell } => {
            assert_eq!(shell, clap_complete::Shell::Bash);
        }
        _ => panic!("Expected Completion command"),
    }
}

#[test]
fn test_completion_zsh() {
    let cli = parse(&["wk", "completion", "zsh"]).unwrap();
    match cli.command {
        Command::Completion { shell } => {
            assert_eq!(shell, clap_complete::Shell::Zsh);
        }
        _ => panic!("Expected Completion command"),
    }
}

#[test]
fn test_completion_invalid_shell() {
    let result = parse(&["wk", "completion", "invalid"]);
    assert!(result.is_err());
}

// Error cases
#[test]
fn test_unknown_command() {
    let result = parse(&["wk", "unknown"]);
    assert!(result.is_err());
}

#[test]
fn test_no_command() {
    let result = parse(&["wk"]);
    assert!(result.is_err());
}
