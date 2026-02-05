// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Filter group parsing and matching functions.
//!
//! This module provides shared filtering utilities used by list, search, and ready commands.

use crate::error::Result;

/// A label matcher that supports positive (`label`) and negative (`!label`) matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LabelMatcher {
    /// Match issues that have this label.
    Has(String),
    /// Match issues that do NOT have this label.
    NotHas(String),
}

impl LabelMatcher {
    /// Parse a label matcher from a string.
    /// A leading `!` indicates negation (NotHas), otherwise it's a positive match (Has).
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(rest) = s.strip_prefix('!') {
            if rest.is_empty() {
                return Err(crate::error::Error::FilterInvalidValue {
                    field: "label".to_string(),
                    reason: "empty label after '!' prefix".to_string(),
                });
            }
            Ok(LabelMatcher::NotHas(rest.to_string()))
        } else {
            Ok(LabelMatcher::Has(s.to_string()))
        }
    }

    /// Check if this matcher matches the given set of issue labels.
    pub fn matches(&self, issue_labels: &[String]) -> bool {
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
/// Each group is OR'd internally (issue has at least one matching label matcher from group),
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
