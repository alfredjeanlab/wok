#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/scenarios/ready.sh - Ready command benchmark scenarios

# This file is sourced by run.sh, which already sources common.sh

# ============================================================================
# Ready Command Benchmarks
# ============================================================================
# The ready command has a hard limit of 5 issues, so it should be consistently
# fast regardless of database size.

# Benchmark ready command across all database sizes
benchmark_ready_default() {
    for size in small medium large xlarge; do
        setup_db "$size"
        run_benchmark "ready_default_${size}" "$WK_BIN" ready
    done
}

# Benchmark ready with JSON output
benchmark_ready_json() {
    for size in small medium large xlarge; do
        setup_db "$size"
        run_benchmark "ready_json_${size}" "$WK_BIN" ready --format json
    done
}

# Benchmark ready with assignee filters
benchmark_ready_assignee() {
    setup_db large
    run_benchmark "ready_assignee_alice" "$WK_BIN" ready --assignee alice
    run_benchmark "ready_assignee_multi" "$WK_BIN" ready --assignee alice,bob
    run_benchmark "ready_unassigned" "$WK_BIN" ready --unassigned
    run_benchmark "ready_all_assignees" "$WK_BIN" ready --all-assignees
}

# Benchmark ready with label filters
benchmark_ready_label() {
    setup_db large
    run_benchmark "ready_label_project" "$WK_BIN" ready --label project:alpha
    run_benchmark "ready_label_priority" "$WK_BIN" ready --label priority:1
    run_benchmark "ready_label_multi" "$WK_BIN" ready --label project:alpha,project:beta
}

# Benchmark ready with type filters
benchmark_ready_type() {
    setup_db large
    run_benchmark "ready_type_task" "$WK_BIN" ready --type task
    run_benchmark "ready_type_bug" "$WK_BIN" ready --type bug
    run_benchmark "ready_type_multi" "$WK_BIN" ready --type task,bug
}

# Run all ready benchmarks
run_ready_benchmarks() {
    info "=== Ready Command Benchmarks ==="
    benchmark_ready_default
    benchmark_ready_json
    benchmark_ready_assignee
    benchmark_ready_label
    benchmark_ready_type
    success "Ready command benchmarks complete"
}

# ============================================================================
# Ready Performance Validation
# ============================================================================
# Verify that ready maintains consistent performance with hard limit

# Compare ready performance across sizes (should be similar)
benchmark_ready_consistency() {
    info "=== Ready Consistency Check ==="

    # Ready with hard limit should have similar performance regardless of DB size
    setup_db small
    run_benchmark "ready_consistency_small" "$WK_BIN" ready

    setup_db xlarge
    run_benchmark "ready_consistency_xlarge" "$WK_BIN" ready

    success "Ready consistency check complete"
}

# Run ready consistency validation
run_ready_consistency() {
    benchmark_ready_consistency
}
