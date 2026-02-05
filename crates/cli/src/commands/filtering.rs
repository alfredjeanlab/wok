// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Filter group parsing and matching functions.
//!
//! This module provides shared filtering utilities used by list, search, and ready commands.

use crate::error::{Error, Result};

/// A label matcher that can be positive (Has) or negative (NotHas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LabelMatcher {
    /// Issue must have this label
    Has(String),
    /// Issue must NOT have this label
    NotHas(String),
}

impl LabelMatcher {
    /// Parse a label matcher from a string.
    /// Strings starting with `!` are negated (NotHas), otherwise positive (Has).
    pub(crate) fn parse(s: &str) -> Result<Self> {
        if let Some(label) = s.strip_prefix('!') {
            if label.is_empty() {
                return Err(Error::FieldEmpty {
                    field: "label after '!'",
                });
            }
            Ok(LabelMatcher::NotHas(label.to_string()))
        } else {
            Ok(LabelMatcher::Has(s.to_string()))
        }
    }

    /// Check if this matcher matches the given issue labels.
    pub(crate) fn matches(&self, issue_labels: &[String]) -> bool {
        match self {
            LabelMatcher::Has(label) => issue_labels.contains(label),
            LabelMatcher::NotHas(label) => !issue_labels.contains(label),
        }
    }
}

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
/// Each group is OR'd internally (at least one matcher in group must match),
/// all groups must match (AND).
pub(crate) fn matches_label_groups(
    groups: &Option<Vec<Vec<LabelMatcher>>>,
    issue_labels: &[String],
) -> bool {
    match groups {
        None => true,
        Some(groups) => groups
            .iter()
            .all(|group| group.iter().any(|matcher| matcher.matches(issue_labels))),
    }
}

/// Check if an issue ID matches the given prefix filter.
/// The prefix is the portion of the ID before the first hyphen.
pub(crate) fn matches_prefix(prefix: &Option<String>, issue_id: &str) -> bool {
    match prefix {
        None => true,
        Some(p) => issue_id
            .split('-')
            .next()
            .is_some_and(|id_prefix| id_prefix == p),
    }
}

#[cfg(test)]
#[path = "filtering_tests.rs"]
mod tests;
