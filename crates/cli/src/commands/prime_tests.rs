// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn template_not_empty() {
    assert!(!TEMPLATE.is_empty());
}

#[test]
fn template_contains_expected_sections() {
    assert!(TEMPLATE.contains("## Core Rules"));
    assert!(TEMPLATE.contains("## Finding Work"));
}

#[test]
fn template_contains_common_commands() {
    assert!(TEMPLATE.contains("wk list"));
    assert!(TEMPLATE.contains("wk new"));
    assert!(TEMPLATE.contains("wk start"));
    assert!(TEMPLATE.contains("wk done"));
}

#[test]
fn run_succeeds() {
    // The run function should succeed
    let result = run();
    assert!(result.is_ok());
}

#[test]
fn template_starts_with_header() {
    // Verify template starts with expected markdown header
    assert!(TEMPLATE.starts_with("# "));
}

#[test]
fn template_contains_priority_documentation() {
    // Verify priority tag documentation is present
    assert!(TEMPLATE.contains("priority:"));
}

#[test]
fn template_contains_dependency_examples() {
    // Verify dep command examples
    assert!(TEMPLATE.contains("wk dep"));
    assert!(TEMPLATE.contains("blocks"));
}

#[test]
fn template_no_trailing_whitespace() {
    // Quality check: no trailing whitespace on lines
    for line in TEMPLATE.lines() {
        assert_eq!(
            line,
            line.trim_end(),
            "Line has trailing whitespace: {:?}",
            line
        );
    }
}
