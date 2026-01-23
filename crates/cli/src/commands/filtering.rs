// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Filter group parsing and matching functions.
//!
//! This module provides shared filtering utilities used by list, search, and ready commands.

use crate::error::Result;

/// Parse filter values: comma-separated values within each Vec entry are OR'd,
/// multiple Vec entries are AND'd together.
/// Returns None if no filters provided, Some(groups) otherwise.
pub(crate) fn parse_filter_groups<T, F>(
    values: &[String],
    parse_fn: F,
) -> Result<Option<Vec<Vec<T>>>>
where
    F: Fn(&str) -> Result<T>,
{
    if values.is_empty() {
        return Ok(None);
    }

    let mut groups = Vec::new();
    for value in values {
        let mut group = Vec::new();
        for part in value.split(',') {
            let part = part.trim();
            if !part.is_empty() {
                group.push(parse_fn(part)?);
            }
        }
        if !group.is_empty() {
            groups.push(group);
        }
    }

    if groups.is_empty() {
        Ok(None)
    } else {
        Ok(Some(groups))
    }
}

/// Check if an issue matches the filter groups.
/// Each group is OR'd internally, all groups must match (AND).
pub(crate) fn matches_filter_groups<T, F>(groups: &Option<Vec<Vec<T>>>, get_value: F) -> bool
where
    T: PartialEq,
    F: Fn() -> T,
{
    match groups {
        None => true,
        Some(groups) => {
            let value = get_value();
            groups.iter().all(|group| group.contains(&value))
        }
    }
}

/// Check if an issue matches label filter groups.
/// Each group is OR'd internally (issue has at least one label from group),
/// all groups must match (AND).
pub(crate) fn matches_label_groups(
    groups: &Option<Vec<Vec<String>>>,
    issue_labels: &[String],
) -> bool {
    match groups {
        None => true,
        Some(groups) => groups
            .iter()
            .all(|group| group.iter().any(|label| issue_labels.contains(label))),
    }
}

#[cfg(test)]
#[path = "filtering_tests.rs"]
mod tests;
