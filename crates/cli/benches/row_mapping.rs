// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Benchmarks for SQL row to Issue struct mapping.

#![allow(clippy::expect_used)]

use chrono::Utc;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use wkrs::db::Database;
use wkrs::models::{Issue, IssueType, Status};

fn row_mapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("row_mapping");

    group.bench_function("get_issue", |b| {
        b.iter_batched(
            || {
                let db = Database::open_in_memory().expect("open db");
                let issue = create_test_issue("test-001");
                db.create_issue(&issue).expect("create issue");
                (db, issue.id)
            },
            |(db, id)| db.get_issue(&id),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("list_issues_10", |b| {
        b.iter_batched(
            || {
                let db = Database::open_in_memory().expect("open db");
                for i in 0..10 {
                    let issue = create_test_issue(&format!("test-{:03}", i));
                    db.create_issue(&issue).expect("create issue");
                }
                db
            },
            |db| db.list_issues(None, None, None),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("list_issues_100", |b| {
        b.iter_batched(
            || {
                let db = Database::open_in_memory().expect("open db");
                for i in 0..100 {
                    let issue = create_test_issue(&format!("test-{:03}", i));
                    db.create_issue(&issue).expect("create issue");
                }
                db
            },
            |db| db.list_issues(None, None, None),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn create_test_issue(id: &str) -> Issue {
    let now = Utc::now();
    Issue {
        id: id.to_string(),
        issue_type: IssueType::Task,
        title: "Test issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: now,
        updated_at: now,
        closed_at: None,
    }
}

criterion_group!(benches, row_mapping);
criterion_main!(benches);
