// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Evaluation of filter expressions against issues.

use chrono::{DateTime, Duration, NaiveTime, Utc};

use crate::models::{Issue, Status};

use super::expr::{CompareOp, FilterExpr, FilterField, FilterValue};

impl FilterExpr {
    /// Evaluate this filter against an issue at a given reference time.
    ///
    /// For duration-based filters (e.g., `age < 3d`), the comparison is against
    /// the age (time elapsed), not the timestamp:
    /// - `age < 3d` means "created less than 3 days ago" (recent issues)
    /// - `age > 1w` means "created more than 1 week ago" (older issues)
    ///
    /// For date-based filters (e.g., `created > 2024-01-01`), the comparison
    /// is directly against the timestamp.
    ///
    /// For closed filters:
    /// - `completed`/`done`: only matches issues with Status::Done
    /// - `skipped`/`cancelled`: only matches issues with Status::Closed
    /// - `closed`: matches any terminal state (Status::Done or Status::Closed)
    pub fn matches(&self, issue: &Issue, now: DateTime<Utc>) -> bool {
        // Check status requirement for terminal-state fields
        let status_matches = match self.field {
            FilterField::Completed => issue.status == Status::Done,
            FilterField::Skipped => issue.status == Status::Closed,
            FilterField::Closed => issue.status == Status::Done || issue.status == Status::Closed,
            FilterField::Age | FilterField::Updated => true,
        };

        if !status_matches {
            return false;
        }

        // Get the relevant timestamp for this field
        let issue_time = match self.field {
            FilterField::Age => Some(issue.created_at),
            FilterField::Updated => Some(issue.updated_at),
            FilterField::Completed | FilterField::Skipped | FilterField::Closed => issue.closed_at,
        };

        // For terminal-state fields: non-closed issues never match
        let issue_time = match issue_time {
            Some(t) => t,
            None => return false,
        };

        match &self.value {
            FilterValue::Duration(threshold) => {
                // Calculate the age of the issue
                let age = now.signed_duration_since(issue_time);
                self.op.compare_duration(age, *threshold)
            }
            FilterValue::Date(date) => {
                // Convert the date to a datetime at midnight UTC
                let threshold = date
                    .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default())
                    .and_utc();
                self.op.compare_datetime(issue_time, threshold)
            }
            FilterValue::Now => {
                // "now" compares the issue timestamp directly to the current time
                self.op.compare_datetime(issue_time, now)
            }
        }
    }
}

impl CompareOp {
    /// Compare two durations.
    fn compare_duration(&self, actual: Duration, threshold: Duration) -> bool {
        match self {
            CompareOp::Lt => actual < threshold,
            CompareOp::Le => actual <= threshold,
            CompareOp::Gt => actual > threshold,
            CompareOp::Ge => actual >= threshold,
            CompareOp::Eq => actual == threshold,
            CompareOp::Ne => actual != threshold,
        }
    }

    /// Compare two datetimes.
    ///
    /// For equality comparisons, we compare only the date portion
    /// (ignoring time) to make date-based filters more intuitive.
    fn compare_datetime(&self, actual: DateTime<Utc>, threshold: DateTime<Utc>) -> bool {
        match self {
            CompareOp::Lt => actual < threshold,
            CompareOp::Le => actual < threshold || actual.date_naive() == threshold.date_naive(),
            CompareOp::Gt => actual >= threshold && actual.date_naive() != threshold.date_naive(),
            CompareOp::Ge => actual >= threshold,
            CompareOp::Eq => actual.date_naive() == threshold.date_naive(),
            CompareOp::Ne => actual.date_naive() != threshold.date_naive(),
        }
    }
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
