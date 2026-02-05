// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Filter string parsing for hook configurations.
//!
//! Parses filter strings like "-t bug -l urgent" into filter criteria
//! that can be applied to issues.

use crate::commands::filtering::{matches_label_groups, matches_prefix, LabelMatcher};
use crate::error::{Error, Result};
use crate::models::{Issue, IssueType, Status};

/// Parsed filter criteria from a hook config filter string.
#[derive(Debug, Clone, Default)]
pub struct HookFilter {
    /// Type filter groups (comma-separated within group = OR, multiple groups = AND).
    pub types: Option<Vec<Vec<IssueType>>>,
    /// Label filter groups with positive/negative matchers.
    pub labels: Option<Vec<Vec<LabelMatcher>>>,
    /// Status filter groups.
    pub statuses: Option<Vec<Vec<Status>>>,
    /// Assignee filter groups.
    pub assignees: Option<Vec<Vec<String>>>,
    /// ID prefix filter.
    pub prefix: Option<String>,
}

impl HookFilter {
    /// Parse a filter string (e.g., "-t bug -l urgent,critical -s todo").
    ///
    /// Supports:
    /// - `-t` / `--type`: Issue type filter (comma-separated for OR)
    /// - `-l` / `--label`: Label filter (comma-separated for OR, ! prefix for NOT)
    /// - `-s` / `--status`: Status filter (comma-separated for OR)
    /// - `-a` / `--assignee`: Assignee filter (comma-separated for OR)
    /// - `-p` / `--prefix`: ID prefix filter
    pub fn parse(filter: &str) -> Result<Self> {
        let mut result = HookFilter::default();

        let tokens = tokenize(filter)?;
        let mut iter = tokens.iter().peekable();

        while let Some(token) = iter.next() {
            match token.as_str() {
                "-t" | "--type" => {
                    let value = iter.next().ok_or_else(|| Error::FieldRequired {
                        field: "type value after -t",
                    })?;
                    let types = parse_types(value)?;
                    if let Some(ref mut v) = result.types {
                        v.push(types);
                    } else {
                        result.types = Some(vec![types]);
                    }
                }
                "-l" | "--label" => {
                    let value = iter.next().ok_or_else(|| Error::FieldRequired {
                        field: "label value after -l",
                    })?;
                    let labels = parse_labels(value)?;
                    match &mut result.labels {
                        Some(v) => v.push(labels),
                        None => result.labels = Some(vec![labels]),
                    }
                }
                "-s" | "--status" => {
                    let value = iter.next().ok_or_else(|| Error::FieldRequired {
                        field: "status value after -s",
                    })?;
                    let statuses = parse_statuses(value)?;
                    if let Some(ref mut v) = result.statuses {
                        v.push(statuses);
                    } else {
                        result.statuses = Some(vec![statuses]);
                    }
                }
                "-a" | "--assignee" => {
                    let value = iter.next().ok_or_else(|| Error::FieldRequired {
                        field: "assignee value after -a",
                    })?;
                    let assignees = parse_assignees(value);
                    match &mut result.assignees {
                        Some(v) => v.push(assignees),
                        None => result.assignees = Some(vec![assignees]),
                    }
                }
                "-p" | "--prefix" => {
                    let value = iter.next().ok_or_else(|| Error::FieldRequired {
                        field: "prefix value after -p",
                    })?;
                    result.prefix = Some(value.clone());
                }
                other => {
                    return Err(Error::Config(format!("unknown filter flag: {}", other)));
                }
            }
        }

        Ok(result)
    }

    /// Check if an issue matches this filter.
    pub fn matches(&self, issue: &Issue, labels: &[String]) -> bool {
        // Check type filter
        if let Some(type_groups) = &self.types {
            let matches_types = type_groups
                .iter()
                .all(|group| group.contains(&issue.issue_type));
            if !matches_types {
                return false;
            }
        }

        // Check label filter
        if !matches_label_groups(&self.labels, labels) {
            return false;
        }

        // Check status filter
        if let Some(status_groups) = &self.statuses {
            let matches_statuses = status_groups
                .iter()
                .all(|group| group.contains(&issue.status));
            if !matches_statuses {
                return false;
            }
        }

        // Check assignee filter
        if let Some(assignee_groups) = &self.assignees {
            let matches_assignees = assignee_groups.iter().all(|group| match &issue.assignee {
                Some(assignee) => group.contains(assignee),
                None => false,
            });
            if !matches_assignees {
                return false;
            }
        }

        // Check prefix filter
        if !matches_prefix(&self.prefix, &issue.id) {
            return false;
        }

        true
    }
}

/// Tokenize a filter string, respecting quoted strings.
fn tokenize(input: &str) -> Result<Vec<String>> {
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

    if in_quotes {
        return Err(Error::Config("unclosed quote in filter string".to_string()));
    }

    Ok(tokens)
}

/// Parse comma-separated types.
fn parse_types(value: &str) -> Result<Vec<IssueType>> {
    let mut types = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if !part.is_empty() {
            let issue_type: IssueType = part
                .parse()
                .map_err(|_| Error::Config(format!("invalid issue type: {}", part)))?;
            types.push(issue_type);
        }
    }
    Ok(types)
}

/// Parse comma-separated labels with optional ! prefix.
fn parse_labels(value: &str) -> Result<Vec<LabelMatcher>> {
    let mut labels = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if !part.is_empty() {
            labels.push(LabelMatcher::parse(part)?);
        }
    }
    Ok(labels)
}

/// Parse comma-separated statuses.
fn parse_statuses(value: &str) -> Result<Vec<Status>> {
    let mut statuses = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if !part.is_empty() {
            let status: Status = part
                .parse()
                .map_err(|_| Error::Config(format!("invalid status: {}", part)))?;
            statuses.push(status);
        }
    }
    Ok(statuses)
}

/// Parse comma-separated assignees.
fn parse_assignees(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
#[path = "filter_tests.rs"]
mod tests;
