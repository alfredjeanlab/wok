// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Filter expression types for time-based filtering.
//!
//! Filter expressions allow filtering issues by age or update time using
//! expressions like `age < 3d` or `updated > 1w`.

use chrono::{Duration, NaiveDate};

/// A parsed filter expression.
///
/// Filter expressions have the form `field op value`, for example:
/// - `age < 3d` - issues created less than 3 days ago
/// - `updated > 1w` - issues not updated in 7+ days
#[derive(Debug, Clone, PartialEq)]
pub struct FilterExpr {
    /// The field to filter on (age/created or updated/activity).
    pub field: FilterField,
    /// The comparison operator.
    pub op: CompareOp,
    /// The value to compare against.
    pub value: FilterValue,
}

/// Fields that can be filtered on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    /// Time since creation (`age` or `created` synonyms).
    /// Represents `now - created_at`.
    Age,
    /// Time since last update (`updated` or `activity` synonyms).
    /// Represents `now - updated_at`.
    Updated,
    /// Successfully completed issues only (Status::Done).
    /// Time since closed via `wk done` (`completed` or `done` synonyms).
    Completed,
    /// Cancelled/skipped issues only (Status::Closed).
    /// Time since closed via `wk close --reason` (`skipped` or `cancelled` synonyms).
    Skipped,
    /// Any terminal state (Status::Done or Status::Closed).
    /// Time since closed (`closed` synonym). Matches any issue with a closed_at timestamp.
    Closed,
}

impl FilterField {
    /// Returns valid field names for error messages.
    pub fn valid_names() -> &'static str {
        "age, created, activity, updated, completed, done, skipped, cancelled, closed"
    }
}

/// Comparison operators for filter expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Less than (`<`).
    Lt,
    /// Less than or equal (`<=`).
    Le,
    /// Greater than (`>`).
    Gt,
    /// Greater than or equal (`>=`).
    Ge,
    /// Equal (`=`).
    Eq,
    /// Not equal (`!=`).
    Ne,
}

impl CompareOp {
    /// Returns valid operator symbols for error messages.
    pub fn valid_symbols() -> &'static str {
        "<, <=, >, >=, =, != (or: lt, lte, gt, gte, eq, ne)"
    }
}

/// Values that can be compared against in filter expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// A duration like `3d`, `1w`, `24h`.
    Duration(Duration),
    /// An absolute date like `2024-01-01`.
    Date(NaiveDate),
}

#[cfg(test)]
#[path = "expr_tests.rs"]
mod tests;
