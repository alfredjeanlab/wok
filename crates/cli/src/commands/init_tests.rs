// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Unit tests for the init command.
//!
//! Tests cover:
//! - Prefix validation rules (length, characters, letter requirement)
//! - Config creation and serialization
//! - Prefix derivation from directory names

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::path::PathBuf;

use yare::parameterized;

use super::derive_prefix_from_path;
use crate::config::Config;
use crate::id::validate_prefix;

/// Tests for prefix validation rules.
mod prefix_validation {
    use super::*;

    #[parameterized(
        two_char_lowercase = { "ab" },
        three_char_lowercase = { "foo" },
        common_prefix = { "test" },
        long_prefix = { "myproject" },
        with_trailing_digit = { "project2" },
        with_leading_digit = { "v0" },
        all_mixed = { "abc123" },
    )]
    fn should_accept_valid_prefix(prefix: &str) {
        assert!(
            validate_prefix(prefix),
            "Expected prefix '{}' to be valid",
            prefix
        );
    }

    #[parameterized(
        empty = { "" },
        single_char = { "a" },
        uppercase = { "AB" },
        mixed_case = { "MyProject" },
        with_hyphen = { "my-project" },
        with_underscore = { "my_project" },
        pure_digits = { "123" },
        pure_digits_short = { "12" },
    )]
    fn should_reject_invalid_prefix(prefix: &str) {
        assert!(
            !validate_prefix(prefix),
            "Expected prefix '{}' to be invalid",
            prefix
        );
    }
}

/// Tests for Config creation and serialization.
mod config {
    use super::*;

    #[test]
    fn should_create_config_with_valid_prefix() {
        let config = Config::new("test".to_string()).unwrap();
        assert_eq!(config.prefix, "test");
        assert!(config.workspace.is_none());
    }

    #[test]
    fn should_reject_config_with_empty_prefix() {
        let result = Config::new("".to_string());
        assert!(result.is_err(), "Empty prefix should be rejected");
    }

    #[test]
    fn should_serialize_config_to_toml() {
        let config = Config::new("myproj".to_string()).unwrap();
        let toml = toml::to_string(&config).unwrap();
        assert!(
            toml.contains("prefix = \"myproj\""),
            "Serialized TOML should contain prefix"
        );
    }

    #[test]
    fn should_deserialize_config_from_toml() {
        let toml = r#"
prefix = "testproj"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.prefix, "testproj");
        assert!(config.workspace.is_none());
    }

    #[test]
    fn should_deserialize_config_with_workspace() {
        let toml = r#"
prefix = "testproj"
workspace = "/path/to/workspace"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.prefix, "testproj");
        assert_eq!(config.workspace, Some("/path/to/workspace".to_string()));
    }
}

/// Tests for prefix derivation from directory names.
mod prefix_derivation {
    use super::*;

    /// Helper to derive prefix from a given directory name.
    fn derive_from(dir_name: &str) -> Result<String, crate::error::Error> {
        let path = PathBuf::from(format!("/tmp/{}", dir_name));
        derive_prefix_from_path(&path)
    }

    #[parameterized(
        simple_lowercase = { "myproject", "myproject" },
        mixed_case_converted = { "MyProject", "myproject" },
        with_digits = { "Project123", "project123" },
        symbols_stripped = { "my-project_v2", "myprojectv2" },
        leading_digit_kept = { "123abc", "123abc" },
        multiple_symbols_stripped = { "foo--bar__baz", "foobarbaz" },
    )]
    fn should_derive_prefix_from_valid_directory(dir_name: &str, expected: &str) {
        let result = derive_from(dir_name).unwrap();
        assert_eq!(
            result, expected,
            "Expected '{}' -> '{}', got '{}'",
            dir_name, expected, result
        );
    }

    #[parameterized(
        too_short_after_strip = { "a---" },
        all_symbols = { "---" },
        single_char = { "x" },
        digits_only = { "123" },
        empty_after_strip = { "---___" },
        unicode_only = { "проект" },
    )]
    fn should_fail_to_derive_prefix_from_invalid_directory(dir_name: &str) {
        let result = derive_from(dir_name);
        assert!(
            result.is_err(),
            "Expected error for directory '{}', got {:?}",
            dir_name,
            result
        );
    }
}
