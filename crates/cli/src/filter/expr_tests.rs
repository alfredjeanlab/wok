// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use chrono::Duration;

#[test]
fn filter_field_valid_names_includes_all_synonyms() {
    let names = FilterField::valid_names();
    assert!(names.contains("age"));
    assert!(names.contains("created"));
    assert!(names.contains("updated"));
    assert!(names.contains("activity"));
    assert!(names.contains("completed"));
    assert!(names.contains("done"));
    assert!(names.contains("skipped"));
    assert!(names.contains("cancelled"));
    assert!(names.contains("closed"));
}

#[test]
fn compare_op_valid_symbols_includes_all_operators() {
    let symbols = CompareOp::valid_symbols();
    assert!(symbols.contains('<'));
    assert!(symbols.contains('>'));
    assert!(symbols.contains('='));
    assert!(symbols.contains("<="));
    assert!(symbols.contains(">="));
    assert!(symbols.contains("!="));
}

#[test]
fn filter_expr_equality() {
    let expr1 = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let expr2 = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    assert_eq!(expr1, expr2);
}

#[test]
fn filter_expr_inequality_different_field() {
    let expr1 = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let expr2 = FilterExpr {
        field: FilterField::Updated,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    assert_ne!(expr1, expr2);
}

#[test]
fn filter_expr_terminal_fields_distinct() {
    let age = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let updated = FilterExpr {
        field: FilterField::Updated,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let completed = FilterExpr {
        field: FilterField::Completed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let skipped = FilterExpr {
        field: FilterField::Skipped,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let closed = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    // All fields should be distinct
    assert_ne!(age, completed);
    assert_ne!(age, skipped);
    assert_ne!(age, closed);
    assert_ne!(updated, completed);
    assert_ne!(updated, skipped);
    assert_ne!(updated, closed);
    assert_ne!(completed, skipped);
    assert_ne!(completed, closed);
    assert_ne!(skipped, closed);
}

#[test]
fn filter_expr_inequality_different_op() {
    let expr1 = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let expr2 = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Gt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    assert_ne!(expr1, expr2);
}

#[test]
fn filter_expr_inequality_different_value() {
    let expr1 = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(3)),
    };
    let expr2 = FilterExpr {
        field: FilterField::Age,
        op: CompareOp::Lt,
        value: FilterValue::Duration(Duration::days(5)),
    };
    assert_ne!(expr1, expr2);
}

#[test]
fn filter_value_duration_vs_date() {
    let duration = FilterValue::Duration(Duration::days(1));
    let date = FilterValue::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert_ne!(duration, date);
}

#[test]
fn filter_expr_clone() {
    let expr = FilterExpr {
        field: FilterField::Updated,
        op: CompareOp::Ge,
        value: FilterValue::Duration(Duration::weeks(1)),
    };
    let cloned = expr.clone();
    assert_eq!(expr, cloned);
}
