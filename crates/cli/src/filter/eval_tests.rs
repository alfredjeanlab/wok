// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::models::{Issue, IssueType};
use chrono::{Duration, NaiveDate, TimeZone};

fn make_issue_at(created: DateTime<Utc>, updated: DateTime<Utc>) -> Issue {
    Issue {
        id: "test-1234".to_string(),
        issue_type: IssueType::Task,
        title: "Test issue".to_string(),
        description: None,
        status: crate::models::Status::Todo,
        assignee: None,
        created_at: created,
        updated_at: updated,
        closed_at: None,
    }
}

fn make_issue_created_at(created: DateTime<Utc>) -> Issue {
    make_issue_at(created, created)
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration-based age filtering
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn age_less_than_matches_recent_issues() {
    let now = Utc::now();
    let created = now - Duration::hours(1);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn age_less_than_excludes_old_issues() {
    let now = Utc::now();
    let created = now - Duration::days(5);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };

    assert!(!expr.matches(&issue, now));
}

#[test]
fn age_greater_than_matches_old_issues() {
    let now = Utc::now();
    let created = now - Duration::days(10);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Gt,
        value: FilterValue::Duration(Duration::weeks(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn age_greater_than_excludes_recent_issues() {
    let now = Utc::now();
    let created = now - Duration::days(3);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Gt,
        value: FilterValue::Duration(Duration::weeks(1)),
    };

    assert!(!expr.matches(&issue, now));
}

#[test]
fn age_less_than_or_equal_boundary() {
    let now = Utc::now();
    let created = now - Duration::days(3);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Le,
        value: FilterValue::Duration(Duration::days(3)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn age_greater_than_or_equal_boundary() {
    let now = Utc::now();
    let created = now - Duration::days(7);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Ge,
        value: FilterValue::Duration(Duration::weeks(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn age_equal_exact_match() {
    let now = Utc::now();
    let created = now - Duration::days(5);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Eq,
        value: FilterValue::Duration(Duration::days(5)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn age_not_equal() {
    let now = Utc::now();
    let created = now - Duration::days(5);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Ne,
        value: FilterValue::Duration(Duration::days(3)),
    };

    assert!(expr.matches(&issue, now));
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration-based updated filtering
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn updated_less_than_matches_recently_updated() {
    let now = Utc::now();
    let created = now - Duration::days(10);
    let updated = now - Duration::hours(2);
    let issue = make_issue_at(created, updated);

    let expr = FilterExpr {
        field: FilterField::Updated,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn updated_greater_than_matches_stale_issues() {
    let now = Utc::now();
    let created = now - Duration::days(30);
    let updated = now - Duration::days(14);
    let issue = make_issue_at(created, updated);

    let expr = FilterExpr {
        field: FilterField::Updated,
        op: CompareOp::Gt,
        value: FilterValue::Duration(Duration::weeks(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn updated_less_than_excludes_stale_issues() {
    let now = Utc::now();
    let created = now - Duration::days(30);
    let updated = now - Duration::days(14);
    let issue = make_issue_at(created, updated);

    let expr = FilterExpr {
        field: FilterField::Updated,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(7)),
    };

    assert!(!expr.matches(&issue, now));
}

// ─────────────────────────────────────────────────────────────────────────────
// Date-based filtering
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn created_after_date() {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let created = Utc.with_ymd_and_hms(2024, 3, 1, 10, 0, 0).unwrap();
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Gt,
        value: FilterValue::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
    };

    // Issue was created on March 1, which is after Jan 1
    assert!(expr.matches(&issue, now));
}

#[test]
fn created_before_date() {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let created = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Date(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
    };

    // Issue was created on Jan 15, which is before March 1
    assert!(expr.matches(&issue, now));
}

#[test]
fn created_on_date_with_eq() {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let created = Utc.with_ymd_and_hms(2024, 3, 1, 14, 30, 0).unwrap();
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Eq,
        value: FilterValue::Date(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
    };

    // Should match because dates are compared ignoring time
    assert!(expr.matches(&issue, now));
}

#[test]
fn created_not_on_date() {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let created = Utc.with_ymd_and_hms(2024, 3, 2, 10, 0, 0).unwrap();
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Ne,
        value: FilterValue::Date(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
    };

    // Issue was created on March 2, not March 1
    assert!(expr.matches(&issue, now));
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn zero_duration_age_less_than() {
    let now = Utc::now();
    let created = now - Duration::seconds(1);
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::zero()),
    };

    // 1 second old is not less than 0
    assert!(!expr.matches(&issue, now));
}

#[test]
fn issue_created_at_now() {
    let now = Utc::now();
    let issue = make_issue_created_at(now);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::seconds(1)),
    };

    // Age is ~0, which is less than 1 second
    assert!(expr.matches(&issue, now));
}

#[test]
fn issue_created_in_future_negative_age() {
    let now = Utc::now();
    let created = now + Duration::hours(1); // Future timestamp
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::zero()),
    };

    // Negative age is less than 0
    assert!(expr.matches(&issue, now));
}

#[test]
fn very_old_issue() {
    let now = Utc::now();
    let created = now - Duration::days(3650); // 10 years
    let issue = make_issue_created_at(created);

    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Gt,
        value: FilterValue::Duration(Duration::days(365)),
    };

    assert!(expr.matches(&issue, now));
}

// ─────────────────────────────────────────────────────────────────────────────
// Closed field filtering
// ─────────────────────────────────────────────────────────────────────────────

fn make_closed_issue(closed: DateTime<Utc>) -> Issue {
    Issue {
        id: "test-1234".to_string(),
        issue_type: IssueType::Task,
        title: "Test issue".to_string(),
        description: None,
        status: crate::models::Status::Done,
        assignee: None,
        created_at: closed - Duration::days(7),
        updated_at: closed,
        closed_at: Some(closed),
    }
}

#[test]
fn closed_less_than_matches_recently_closed() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_closed_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn closed_less_than_excludes_old_closed() {
    let now = Utc::now();
    let closed = now - Duration::days(5);
    let issue = make_closed_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };

    assert!(!expr.matches(&issue, now));
}

#[test]
fn closed_greater_than_matches_old_closed() {
    let now = Utc::now();
    let closed = now - Duration::days(10);
    let issue = make_closed_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Gt,
        value: FilterValue::Duration(Duration::weeks(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn closed_filter_excludes_open_issues() {
    let now = Utc::now();
    let issue = make_issue_created_at(now - Duration::days(1));
    // issue.closed_at is None (open issue)

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(30)),
    };

    // Open issues never match closed filter
    assert!(!expr.matches(&issue, now));
}

#[test]
fn closed_after_date() {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let closed = Utc.with_ymd_and_hms(2024, 3, 1, 10, 0, 0).unwrap();
    let issue = make_closed_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Gt,
        value: FilterValue::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
    };

    // Issue was closed on March 1, which is after Jan 1
    assert!(expr.matches(&issue, now));
}

#[test]
fn closed_on_date_with_eq() {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let closed = Utc.with_ymd_and_hms(2024, 3, 1, 14, 30, 0).unwrap();
    let issue = make_closed_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Eq,
        value: FilterValue::Date(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
    };

    // Should match because dates are compared ignoring time
    assert!(expr.matches(&issue, now));
}

// ─────────────────────────────────────────────────────────────────────────────
// Status-aware filtering (Completed vs Skipped vs Closed)
// ─────────────────────────────────────────────────────────────────────────────

fn make_done_issue(closed: DateTime<Utc>) -> Issue {
    Issue {
        id: "test-1234".to_string(),
        issue_type: IssueType::Task,
        title: "Done issue".to_string(),
        description: None,
        status: crate::models::Status::Done,
        assignee: None,
        created_at: closed - Duration::days(7),
        updated_at: closed,
        closed_at: Some(closed),
    }
}

fn make_cancelled_issue(closed: DateTime<Utc>) -> Issue {
    Issue {
        id: "test-5678".to_string(),
        issue_type: IssueType::Task,
        title: "Cancelled issue".to_string(),
        description: None,
        status: crate::models::Status::Closed,
        assignee: None,
        created_at: closed - Duration::days(7),
        updated_at: closed,
        closed_at: Some(closed),
    }
}

#[test]
fn completed_filter_matches_done_status() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_done_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Completed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn completed_filter_excludes_closed_status() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_cancelled_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Completed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    // Completed filter should NOT match Status::Closed issues
    assert!(!expr.matches(&issue, now));
}

#[test]
fn completed_filter_excludes_open_issues() {
    let now = Utc::now();
    let issue = make_issue_created_at(now - Duration::days(1));
    // issue.status is Todo and closed_at is None

    let expr = FilterExpr {
        field: FilterField::Completed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(30)),
    };

    assert!(!expr.matches(&issue, now));
}

#[test]
fn skipped_filter_matches_closed_status() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_cancelled_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Skipped,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn skipped_filter_excludes_done_status() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_done_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Skipped,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    // Skipped filter should NOT match Status::Done issues
    assert!(!expr.matches(&issue, now));
}

#[test]
fn skipped_filter_excludes_open_issues() {
    let now = Utc::now();
    let issue = make_issue_created_at(now - Duration::days(1));

    let expr = FilterExpr {
        field: FilterField::Skipped,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(30)),
    };

    assert!(!expr.matches(&issue, now));
}

#[test]
fn closed_filter_matches_done_status() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_done_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    assert!(expr.matches(&issue, now));
}

#[test]
fn closed_filter_matches_closed_status() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_cancelled_issue(closed);

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(1)),
    };

    assert!(expr.matches(&issue, now));
}

// ─────────────────────────────────────────────────────────────────────────────
// Now value evaluation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn now_value_matches_past_timestamps() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_closed_issue(closed);

    // closed < now should match (closed 1 hour ago)
    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Now,
    };
    assert!(expr.matches(&issue, now));

    // closed > now should not match
    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Gt,
        value: FilterValue::Now,
    };
    assert!(!expr.matches(&issue, now));
}

#[test]
fn now_value_with_age_field() {
    let now = Utc::now();
    let created = now - Duration::hours(1);
    let issue = make_issue_created_at(created);

    // created < now should match (created 1 hour ago)
    let expr = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Now,
    };
    assert!(expr.matches(&issue, now));
}

#[test]
fn now_value_with_updated_field() {
    let now = Utc::now();
    let created = now - Duration::days(7);
    let updated = now - Duration::hours(1);
    let issue = make_issue_at(created, updated);

    // updated < now should match
    let expr = FilterExpr {
        field: FilterField::Updated,
        op: CompareOp::Lt,
        value: FilterValue::Now,
    };
    assert!(expr.matches(&issue, now));
}

#[test]
fn now_value_excludes_open_issues_for_closed_field() {
    let now = Utc::now();
    let issue = make_issue_created_at(now - Duration::days(1));
    // issue.closed_at is None (open issue)

    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Now,
    };

    // Open issues never match closed filter even with Now value
    assert!(!expr.matches(&issue, now));
}

#[test]
fn now_value_with_le_operator() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_closed_issue(closed);

    // closed <= now should match
    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Le,
        value: FilterValue::Now,
    };
    assert!(expr.matches(&issue, now));
}

#[test]
fn now_value_with_ge_operator() {
    let now = Utc::now();
    let closed = now - Duration::hours(1);
    let issue = make_closed_issue(closed);

    // closed >= now should not match (closed in the past)
    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Ge,
        value: FilterValue::Now,
    };
    assert!(!expr.matches(&issue, now));
}
