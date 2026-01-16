// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::config::Config;
use crate::id::validate_prefix;

#[test]
fn test_valid_prefix() {
    assert!(validate_prefix("test"));
    assert!(validate_prefix("myproject"));
    assert!(validate_prefix("foo"));
    assert!(validate_prefix("ab")); // Minimum 2 chars
}

#[test]
fn test_invalid_prefix_empty() {
    assert!(!validate_prefix(""));
}

#[test]
fn test_invalid_prefix_with_dash() {
    assert!(!validate_prefix("my-project"));
}

#[test]
fn test_prefix_with_numbers() {
    // Numbers are allowed when mixed with letters
    assert!(validate_prefix("project2"));
    assert!(validate_prefix("v0"));
    // Pure numbers are not allowed
    assert!(!validate_prefix("123"));
}

#[test]
fn test_config_creation() {
    let config = Config::new("test".to_string()).unwrap();
    assert_eq!(config.prefix, "test");
    assert!(config.workspace.is_none());
}

#[test]
fn test_config_with_invalid_prefix() {
    let result = Config::new("".to_string());
    assert!(result.is_err());
}

#[test]
fn test_config_serialization() {
    let config = Config::new("myproj".to_string()).unwrap();
    let toml = toml::to_string(&config).unwrap();
    assert!(toml.contains("prefix = \"myproj\""));
}

#[test]
fn test_config_deserialization() {
    let toml = r#"
prefix = "testproj"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.prefix, "testproj");
    assert!(config.workspace.is_none());
}

#[test]
fn test_config_with_workspace() {
    let toml = r#"
prefix = "testproj"
workspace = "/path/to/workspace"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.prefix, "testproj");
    assert_eq!(config.workspace, Some("/path/to/workspace".to_string()));
}
