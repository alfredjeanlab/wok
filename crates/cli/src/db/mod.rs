// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database access for the CLI.
//!
//! In private mode, the CLI opens the database directly using [`wk_core::Database`].
//! Standalone functions provide priority parsing.

pub use wk_core::Database;

/// Extract priority from label list.
///
/// Prefers "priority:" over "p:" if both present.
/// Returns 0-4 where 0 is highest priority.
/// Default (no priority label): 2 (medium)
pub fn priority_from_tags(tags: &[String]) -> u8 {
    for tag in tags {
        if let Some(value) = tag.strip_prefix("priority:") {
            if let Some(p) = parse_priority_value(value) {
                return p;
            }
        }
    }
    for tag in tags {
        if let Some(value) = tag.strip_prefix("p:") {
            if let Some(p) = parse_priority_value(value) {
                return p;
            }
        }
    }
    2
}

/// Parse priority value (numeric 0-4 or named).
fn parse_priority_value(value: &str) -> Option<u8> {
    match value {
        "0" | "highest" => Some(0),
        "1" | "high" => Some(1),
        "2" | "medium" | "med" => Some(2),
        "3" | "low" => Some(3),
        "4" | "lowest" => Some(4),
        _ => None,
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
