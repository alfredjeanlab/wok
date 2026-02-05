// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Filter string parsing for hook conditions.
//!
//! Parses CLI-style filter strings like "-t bug -l urgent" into filter criteria.

use crate::models::{Issue, IssueType, Status};

use crate::commands::filtering::{
    matches_filter_groups, matches_label_groups, matches_prefix, parse_filter_groups, LabelMatcher,
};
use crate::error::{Error, Result};

/// Parsed filter criteria from a hook configuration filter string.
#[derive(Debug, Clone, Default)]
pub struct HookFilter {
    /// Type filters (OR within group, AND across groups).
    pub types: Option<Vec<Vec<IssueType>>>,
    /// Label filters with Has/NotHas matching.
    pub labels: Option<Vec<Vec<LabelMatcher>>>,
    /// Status filters (OR within group, AND across groups).
    pub statuses: Option<Vec<Vec<Status>>>,
    /// Assignee filters (OR within group, AND across groups).
    pub assignees: Option<Vec<Vec<String>>>,
    /// Issue ID prefix filter.
    pub prefix: Option<String>,
}

impl HookFilter {
    /// Parse a filter string into filter criteria.
    ///
    /// Supports the following flags:
    /// - `-t`, `--type`: Filter by issue type (comma-separated for OR)
    /// - `-l`, `--label`: Filter by label (comma-separated for OR, ! prefix for negation)
    /// - `-s`, `--status`: Filter by status (comma-separated for OR)
    /// - `-a`, `--assignee`: Filter by assignee (comma-separated for OR)
    /// - `-p`, `--prefix`: Filter by issue ID prefix
    ///
    /// # Errors
    ///
    /// Returns an error if the filter string contains invalid syntax or values.
    pub fn parse(filter: &str) -> Result<Self> {
        let tokens = tokenize(filter);
        let mut result = HookFilter::default();

        let mut types_raw: Vec<String> = Vec::new();
        let mut labels_raw: Vec<String> = Vec::new();
        let mut statuses_raw: Vec<String> = Vec::new();
        let mut assignees_raw: Vec<String> = Vec::new();

        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            match token.as_str() {
                "-t" | "--type" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(Error::Config("--type requires a value".to_string()));
                    }
                    types_raw.push(tokens[i].clone());
                }
                "-l" | "--label" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(Error::Config("--label requires a value".to_string()));
                    }
                    labels_raw.push(tokens[i].clone());
                }
                "-s" | "--status" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(Error::Config("--status requires a value".to_string()));
                    }
                    statuses_raw.push(tokens[i].clone());
                }
                "-a" | "--assignee" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(Error::Config("--assignee requires a value".to_string()));
                    }
                    assignees_raw.push(tokens[i].clone());
                }
                "-p" | "--prefix" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(Error::Config("--prefix requires a value".to_string()));
                    }
                    result.prefix = Some(tokens[i].clone());
                }
                other => {
                    return Err(Error::Config(format!("unknown filter flag: {}", other)));
                }
            }
            i += 1;
        }

        // Parse type groups
        result.types = parse_filter_groups(&types_raw, |s| Ok(s.parse::<IssueType>()?))?;

        // Parse label groups
        result.labels = parse_filter_groups(&labels_raw, LabelMatcher::parse)?;

        // Parse status groups
        result.statuses = parse_filter_groups(&statuses_raw, |s| Ok(s.parse::<Status>()?))?;

        // Parse assignee groups (no parsing needed, just strings)
        result.assignees = parse_filter_groups(&assignees_raw, |s| Ok(s.to_string()))?;

        Ok(result)
    }

    /// Check if an issue matches this filter.
    #[must_use]
    pub fn matches(&self, issue: &Issue, labels: &[String]) -> bool {
        // Check type filter
        if !matches_filter_groups(&self.types, || issue.issue_type) {
            return false;
        }

        // Check label filter
        if !matches_label_groups(&self.labels, labels) {
            return false;
        }

        // Check status filter
        if !matches_filter_groups(&self.statuses, || issue.status) {
            return false;
        }

        // Check assignee filter
        if let Some(ref groups) = self.assignees {
            let assignee = issue.assignee.clone().unwrap_or_default();
            if !groups.iter().all(|group| group.contains(&assignee)) {
                return false;
            }
        }

        // Check prefix filter
        if !matches_prefix(&self.prefix, &issue.id) {
            return false;
        }

        true
    }

    /// Returns true if no filters are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_none()
            && self.labels.is_none()
            && self.statuses.is_none()
            && self.assignees.is_none()
            && self.prefix.is_none()
    }
}

/// Tokenize a filter string, respecting quoted values.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for c in input.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
#[path = "filter_tests.rs"]
mod tests;
