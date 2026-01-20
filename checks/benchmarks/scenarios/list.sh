#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/scenarios/list.sh - List command benchmark scenarios

# This file is sourced by run.sh, which already sources common.sh

# ============================================================================
# Phase 3: Core List Benchmarks
# ============================================================================

# Benchmark default list (open issues only) across all database sizes
benchmark_list_default() {
    for size in small medium large xlarge; do
        setup_db "$size"
        run_benchmark "list_default_${size}" "$WK_BIN" list
    done
}

# Benchmark list --all (no status filter) across all database sizes
benchmark_list_all() {
    for size in small medium large xlarge; do
        setup_db "$size"
        run_benchmark "list_all_${size}" "$WK_BIN" list --all
    done
}

# Benchmark list with varying limits
benchmark_list_limit() {
    setup_db large
    for limit in 10 50 100 500; do
        run_benchmark "list_limit_${limit}" "$WK_BIN" list --all --limit "$limit"
    done
}

# Run all core list benchmarks
run_list_benchmarks() {
    info "=== Core List Benchmarks ==="
    benchmark_list_default
    benchmark_list_all
    benchmark_list_limit
    success "Core list benchmarks complete"
}

# ============================================================================
# Phase 4: Filter Benchmarks
# ============================================================================

# Benchmark status filtering
benchmark_filter_status() {
    setup_db large
    run_benchmark "filter_status_todo" "$WK_BIN" list --status todo
    run_benchmark "filter_status_in_progress" "$WK_BIN" list --status in_progress
    run_benchmark "filter_status_done" "$WK_BIN" list --status done
    run_benchmark "filter_status_multi" "$WK_BIN" list --status todo,in_progress
}

# Benchmark type filtering
benchmark_filter_type() {
    setup_db large
    run_benchmark "filter_type_task" "$WK_BIN" list --type task --all
    run_benchmark "filter_type_bug" "$WK_BIN" list --type bug --all
    run_benchmark "filter_type_feature" "$WK_BIN" list --type feature --all
    run_benchmark "filter_type_multi" "$WK_BIN" list --type task,bug --all
}

# Benchmark label filtering
benchmark_filter_label() {
    setup_db large
    run_benchmark "filter_label_project" "$WK_BIN" list --label project:alpha --all
    run_benchmark "filter_label_priority" "$WK_BIN" list --label priority:1 --all
    run_benchmark "filter_label_area" "$WK_BIN" list --label area:frontend --all
    run_benchmark "filter_label_multi_or" "$WK_BIN" list --label project:alpha,project:beta --all
    # AND filter: project:alpha AND priority:1
    run_benchmark "filter_label_multi_and" "$WK_BIN" list --label project:alpha --label priority:1 --all
}

# Benchmark assignee filtering
benchmark_filter_assignee() {
    setup_db large
    run_benchmark "filter_assignee_alice" "$WK_BIN" list --assignee alice --all
    run_benchmark "filter_assignee_multi" "$WK_BIN" list --assignee alice,bob --all
    run_benchmark "filter_unassigned" "$WK_BIN" list --unassigned --all
}

# Benchmark blocked filter
benchmark_filter_blocked() {
    setup_db large
    run_benchmark "filter_blocked" "$WK_BIN" list --blocked
    run_benchmark "filter_blocked_todo" "$WK_BIN" list --blocked --status todo
}

# Benchmark time-based filters
benchmark_filter_time() {
    setup_db large
    # Note: Since we just created the DB, all issues are "recent"
    # These benchmarks test filter expression parsing and evaluation overhead
    run_benchmark "filter_age_recent" "$WK_BIN" list --filter "\"age < 1d\"" --all
    run_benchmark "filter_age_old" "$WK_BIN" list --filter "\"age > 7d\"" --all
}

# Run all filter benchmarks
run_filter_benchmarks() {
    info "=== Filter Benchmarks ==="
    benchmark_filter_status
    benchmark_filter_type
    benchmark_filter_label
    benchmark_filter_assignee
    benchmark_filter_blocked
    benchmark_filter_time
    success "Filter benchmarks complete"
}

# ============================================================================
# Phase 5: Combined Filter Benchmarks
# ============================================================================

# Benchmark realistic combined filter queries
benchmark_combined_filters() {
    setup_db large

    # Common workflow: open bugs
    run_benchmark "combined_open_bugs" \
        "$WK_BIN" list --status todo,in_progress --type bug

    # Sprint planning: high priority tasks
    run_benchmark "combined_priority_tasks" \
        "$WK_BIN" list --status todo --type task --label priority:1

    # My work: assigned to specific user and in progress
    run_benchmark "combined_my_work" \
        "$WK_BIN" list --status in_progress --assignee alice

    # Project review: all issues for a project
    run_benchmark "combined_project_all" \
        "$WK_BIN" list --label project:alpha --all

    # Backlog grooming: unassigned todo items
    run_benchmark "combined_backlog" \
        "$WK_BIN" list --status todo --unassigned

    # Complex: multiple labels + status + type
    run_benchmark "combined_complex" \
        "$WK_BIN" list --status todo --type task,bug --label project:alpha,project:beta --label priority:1,priority:2

    # Most restrictive: should return few results
    run_benchmark "combined_restrictive" \
        "$WK_BIN" list --status todo --type bug --label project:alpha --label priority:1 --assignee alice
}

# Run all combined benchmarks
run_combined_benchmarks() {
    info "=== Combined Filter Benchmarks ==="
    benchmark_combined_filters
    success "Combined filter benchmarks complete"
}

# ============================================================================
# Phase 6: Output Format Benchmarks
# ============================================================================

# Benchmark output format overhead
benchmark_output_format() {
    setup_db large

    # Compare text vs JSON output
    run_comparison "output_format_comparison" \
        "$WK_BIN list --all" \
        "$WK_BIN list --all --format json"

    # Individual measurements
    run_benchmark "output_text" "$WK_BIN" list --all
    run_benchmark "output_json" "$WK_BIN" list --all --format json
}

# Run all output benchmarks
run_output_benchmarks() {
    info "=== Output Format Benchmarks ==="
    benchmark_output_format
    success "Output format benchmarks complete"
}

# ============================================================================
# Phase 7: Stress Tests with Default Limit
# ============================================================================
# Test performance with default limit (100) vs unlimited (--limit 0)

# Benchmark default limit (100) on large databases
benchmark_stress_default_limit() {
    info "Stress test: default limit (100)"
    for size in large xlarge; do
        setup_db "$size"
        run_benchmark "stress_default_${size}" "$WK_BIN" list
    done
}

# Benchmark unlimited (--limit 0) on large databases
benchmark_stress_unlimited() {
    info "Stress test: unlimited (--limit 0)"
    for size in large xlarge; do
        setup_db "$size"
        run_benchmark "stress_unlimited_${size}" "$WK_BIN" list --limit 0 --all
    done
}

# Compare default vs unlimited performance
benchmark_stress_compare() {
    setup_db xlarge

    run_comparison "stress_limit_comparison" \
        "$WK_BIN list" \
        "$WK_BIN list --limit 0 --all"
}

# Run all stress benchmarks
run_stress_benchmarks() {
    info "=== Stress Tests ==="
    benchmark_stress_default_limit
    benchmark_stress_unlimited
    benchmark_stress_compare
    success "Stress benchmarks complete"
}
