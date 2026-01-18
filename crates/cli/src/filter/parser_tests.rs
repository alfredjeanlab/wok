// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use chrono::Duration;
use yare::parameterized;

// ─────────────────────────────────────────────────────────────────────────────
// Field parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_field_age() {
    let expr = parse_filter("age < 3d").unwrap();
    assert_eq!(expr.field, FilterField::Age);
}

#[test]
fn parse_field_created_synonym() {
    let expr = parse_filter("created < 3d").unwrap();
    assert_eq!(expr.field, FilterField::Age);
}

#[test]
fn parse_field_updated() {
    let expr = parse_filter("updated > 1w").unwrap();
    assert_eq!(expr.field, FilterField::Updated);
}

#[test]
fn parse_field_activity_synonym() {
    let expr = parse_filter("activity > 1w").unwrap();
    assert_eq!(expr.field, FilterField::Updated);
}

#[test]
fn parse_field_closed() {
    let expr = parse_filter("closed < 3d").unwrap();
    assert_eq!(expr.field, FilterField::Closed);
}

#[test]
fn parse_field_completed_synonym() {
    let expr = parse_filter("completed > 1w").unwrap();
    assert_eq!(expr.field, FilterField::Closed);
}

#[test]
fn parse_field_done_synonym() {
    let expr = parse_filter("done >= 2024-01-01").unwrap();
    assert_eq!(expr.field, FilterField::Closed);
}

#[test]
fn parse_field_case_insensitive() {
    let expr = parse_filter("AGE < 3d").unwrap();
    assert_eq!(expr.field, FilterField::Age);

    let expr = parse_filter("Updated > 1w").unwrap();
    assert_eq!(expr.field, FilterField::Updated);
}

#[test]
fn parse_field_unknown_error() {
    let err = parse_filter("unknown < 3d").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown field"));
    assert!(msg.contains("unknown"));
    assert!(msg.contains("age"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Operator parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_operator_lt() {
    let expr = parse_filter("age < 3d").unwrap();
    assert_eq!(expr.op, CompareOp::Lt);
}

#[test]
fn parse_operator_le() {
    let expr = parse_filter("age <= 3d").unwrap();
    assert_eq!(expr.op, CompareOp::Le);
}

#[test]
fn parse_operator_gt() {
    let expr = parse_filter("age > 3d").unwrap();
    assert_eq!(expr.op, CompareOp::Gt);
}

#[test]
fn parse_operator_ge() {
    let expr = parse_filter("age >= 3d").unwrap();
    assert_eq!(expr.op, CompareOp::Ge);
}

#[test]
fn parse_operator_eq() {
    let expr = parse_filter("age = 3d").unwrap();
    assert_eq!(expr.op, CompareOp::Eq);
}

#[test]
fn parse_operator_ne() {
    let expr = parse_filter("age != 3d").unwrap();
    assert_eq!(expr.op, CompareOp::Ne);
}

#[test]
fn parse_operator_unknown_error() {
    let err = parse_filter("age << 3d").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown operator"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Word-based operators (shell-friendly aliases)
// ─────────────────────────────────────────────────────────────────────────────

#[parameterized(
    lt = { "age lt 3d", CompareOp::Lt },
    lte = { "age lte 3d", CompareOp::Le },
    gt = { "age gt 3d", CompareOp::Gt },
    gte = { "age gte 3d", CompareOp::Ge },
    eq = { "age eq 3d", CompareOp::Eq },
    ne = { "age ne 3d", CompareOp::Ne },
    lt_upper = { "age LT 3d", CompareOp::Lt },
    gte_upper = { "age GTE 3d", CompareOp::Ge },
    lt_mixed = { "age Lt 3d", CompareOp::Lt },
)]
fn parse_operator_word(input: &str, expected: CompareOp) {
    let expr = parse_filter(input).unwrap();
    assert_eq!(expr.op, expected);
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_duration_milliseconds() {
    let expr = parse_filter("age < 500ms").unwrap();
    assert_eq!(
        expr.value,
        FilterValue::Duration(Duration::milliseconds(500))
    );
}

#[test]
fn parse_duration_seconds() {
    let expr = parse_filter("age < 30s").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::seconds(30)));
}

#[test]
fn parse_duration_minutes() {
    let expr = parse_filter("age < 5m").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::minutes(5)));
}

#[test]
fn parse_duration_hours() {
    let expr = parse_filter("age < 24h").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::hours(24)));
}

#[test]
fn parse_duration_days() {
    let expr = parse_filter("age < 3d").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::days(3)));
}

#[test]
fn parse_duration_weeks() {
    let expr = parse_filter("age < 2w").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::weeks(2)));
}

#[test]
fn parse_duration_months() {
    let expr = parse_filter("age < 1M").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::days(30)));
}

#[test]
fn parse_duration_years() {
    let expr = parse_filter("age < 1y").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::days(365)));
}

#[test]
fn parse_duration_zero() {
    let expr = parse_filter("age < 0d").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::zero()));
}

#[test]
fn parse_duration_large_number() {
    let expr = parse_filter("age < 1000d").unwrap();
    assert_eq!(expr.value, FilterValue::Duration(Duration::days(1000)));
}

#[test]
fn parse_duration_invalid_unit_error() {
    let err = parse_filter("age < 3x").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown duration unit"));
    assert!(msg.contains("'x'"));
}

#[test]
fn parse_duration_missing_unit_error() {
    let err = parse_filter("age < 3").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("duration missing unit"));
}

#[test]
fn parse_duration_missing_number_error() {
    let err = parse_filter("age < d").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("must start with a number"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Date parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_date_valid() {
    let expr = parse_filter("created > 2024-01-15").unwrap();
    let expected = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    assert_eq!(expr.value, FilterValue::Date(expected));
}

#[test]
fn parse_date_year_boundary() {
    let expr = parse_filter("age < 2023-12-31").unwrap();
    let expected = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
    assert_eq!(expr.value, FilterValue::Date(expected));
}

#[test]
fn parse_date_leap_year() {
    let expr = parse_filter("age < 2024-02-29").unwrap();
    let expected = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
    assert_eq!(expr.value, FilterValue::Date(expected));
}

#[test]
fn parse_date_invalid_format() {
    // Should fail to parse as date, then fail as duration
    let err = parse_filter("age < 01-15-2024").unwrap_err();
    assert!(err.to_string().contains("duration"));
}

#[test]
fn parse_date_invalid_day() {
    // Feb 30 doesn't exist - should fail to parse as date, then fail as duration
    let err = parse_filter("age < 2024-02-30").unwrap_err();
    assert!(err.to_string().contains("duration"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Whitespace handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_no_whitespace() {
    let expr = parse_filter("age<3d").unwrap();
    assert_eq!(expr.field, FilterField::Age);
    assert_eq!(expr.op, CompareOp::Lt);
    assert_eq!(expr.value, FilterValue::Duration(Duration::days(3)));
}

#[test]
fn parse_extra_whitespace() {
    let expr = parse_filter("  age   <   3d  ").unwrap();
    assert_eq!(expr.field, FilterField::Age);
    assert_eq!(expr.op, CompareOp::Lt);
    assert_eq!(expr.value, FilterValue::Duration(Duration::days(3)));
}

#[test]
fn parse_tabs_and_spaces() {
    let expr = parse_filter("age\t<\t3d").unwrap();
    assert_eq!(expr.field, FilterField::Age);
}

// ─────────────────────────────────────────────────────────────────────────────
// Complete expressions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_age_less_than_3d() {
    let expr = parse_filter("age < 3d").unwrap();
    assert_eq!(
        expr,
        FilterExpr {
            field: FilterField::Age,
            op: CompareOp::Lt,
            value: FilterValue::Duration(Duration::days(3)),
        }
    );
}

#[test]
fn parse_updated_greater_than_1w() {
    let expr = parse_filter("updated > 1w").unwrap();
    assert_eq!(
        expr,
        FilterExpr {
            field: FilterField::Updated,
            op: CompareOp::Gt,
            value: FilterValue::Duration(Duration::weeks(1)),
        }
    );
}

#[test]
fn parse_created_after_date() {
    let expr = parse_filter("created > 2024-01-01").unwrap();
    assert_eq!(expr.field, FilterField::Age);
    assert_eq!(expr.op, CompareOp::Gt);
    assert_eq!(
        expr.value,
        FilterValue::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Error cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_empty_string_error() {
    let err = parse_filter("").unwrap_err();
    assert!(err.to_string().contains("empty filter expression"));
}

#[test]
fn parse_whitespace_only_error() {
    let err = parse_filter("   ").unwrap_err();
    assert!(err.to_string().contains("empty filter expression"));
}

#[test]
fn parse_missing_value_error() {
    let err = parse_filter("age <").unwrap_err();
    assert!(err.to_string().contains("missing value"));
}

#[test]
fn parse_missing_operator_error() {
    let err = parse_filter("age 3d").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown operator"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration unit function
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn duration_parse_standalone() {
    assert_eq!(
        parse_duration("100ms").unwrap(),
        Duration::milliseconds(100)
    );
    assert_eq!(parse_duration("30s").unwrap(), Duration::seconds(30));
    assert_eq!(parse_duration("5m").unwrap(), Duration::minutes(5));
    assert_eq!(parse_duration("2h").unwrap(), Duration::hours(2));
    assert_eq!(parse_duration("7d").unwrap(), Duration::days(7));
    assert_eq!(parse_duration("2w").unwrap(), Duration::weeks(2));
    assert_eq!(parse_duration("3M").unwrap(), Duration::days(90));
    assert_eq!(parse_duration("2y").unwrap(), Duration::days(730));
}

#[test]
fn duration_parse_empty_error() {
    let err = parse_duration("").unwrap_err();
    assert!(err.to_string().contains("empty duration"));
}

#[test]
fn duration_parse_negative_error() {
    let err = parse_duration("-5d").unwrap_err();
    assert!(err.to_string().contains("negative"));
}
