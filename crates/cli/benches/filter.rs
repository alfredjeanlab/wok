// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Benchmarks for filter parsing and evaluation.

#![allow(clippy::expect_used)]

use chrono::{Duration, Utc};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use wkrs::filter::parse_filter;
use wkrs::models::{Issue, IssueType, Status};

fn filter_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_parsing");

    let inputs = [
        ("duration_simple", "age < 3d"),
        ("duration_word_op", "age lte 1w"),
        ("date", "created > 2024-01-01"),
        ("bare_status", "closed"),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("parse_filter", name), input, |b, i| {
            b.iter(|| parse_filter(i))
        });
    }
    group.finish();
}

fn filter_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_evaluation");
    let now = Utc::now();

    let recent_issue = create_issue(now - Duration::hours(1));
    let old_issue = create_issue(now - Duration::weeks(4));

    let filters = [
        ("age_lt_3d", parse_filter("age < 3d").expect("valid filter")),
        ("age_gt_1w", parse_filter("age > 1w").expect("valid filter")),
        (
            "updated_gte_date",
            parse_filter("updated >= 2024-01-01").expect("valid filter"),
        ),
    ];

    for (name, filter) in &filters {
        group.bench_function(format!("matches_recent_{}", name), |b| {
            b.iter(|| filter.matches(&recent_issue, now))
        });
        group.bench_function(format!("matches_old_{}", name), |b| {
            b.iter(|| filter.matches(&old_issue, now))
        });
    }
    group.finish();
}

fn create_issue(created_at: chrono::DateTime<Utc>) -> Issue {
    Issue {
        id: "test-001".to_string(),
        issue_type: IssueType::Task,
        title: "Test issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at,
        updated_at: created_at,
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    }
}

criterion_group!(benches, filter_parsing, filter_evaluation);
criterion_main!(benches);
