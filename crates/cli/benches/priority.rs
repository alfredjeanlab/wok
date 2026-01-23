// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Benchmarks for priority extraction from label lists.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use wkrs::db::Database;

fn priority_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_extraction");

    let cases: &[(&str, Vec<String>)] = &[
        ("empty", vec![]),
        ("no_priority", vec!["bug".into(), "frontend".into()]),
        (
            "priority_prefix",
            vec!["bug".into(), "priority:high".into()],
        ),
        ("p_prefix", vec!["p:1".into(), "feature".into()]),
        ("both_prefixes", vec!["p:3".into(), "priority:1".into()]), // priority: wins
        (
            "many_labels",
            (0..20)
                .map(|i| format!("label{}", i))
                .chain(["priority:0".into()])
                .collect(),
        ),
    ];

    for (name, labels) in cases {
        group.bench_with_input(
            BenchmarkId::new("priority_from_tags", name),
            labels,
            |b, l| b.iter(|| Database::priority_from_tags(l)),
        );
    }
    group.finish();
}

criterion_group!(benches, priority_extraction);
criterion_main!(benches);
