#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/scenarios/search.sh - Search command benchmark scenarios

# This file is sourced by run.sh, which already sources common.sh and bench.sh

# ============================================================================
# Basic Search Benchmarks
# ============================================================================

# Benchmark basic search scaling across database sizes
# Uses "task" as query term (matches ~60% of issues by title content)
benchmark_search_basic() {
    info "Benchmarking basic search scaling..."

    for size in small medium large xlarge; do
        setup_db "$size"
        run_benchmark "search_basic_${size}" "$WK_BIN" search "task"
    done
}

# Benchmark search with different match rates
benchmark_search_match_rates() {
    info "Benchmarking search match rates..."

    setup_db large

    # High match rate (~60%): "task" appears in most task titles
    run_benchmark "search_high_match" "$WK_BIN" search "task"

    # Medium match rate (~20%): "bug" appears in bug issue titles
    run_benchmark "search_medium_match" "$WK_BIN" search "bug"

    # Low match rate (~15%): "feature" appears in feature issue titles
    run_benchmark "search_low_match" "$WK_BIN" search "feature"

    # Very low match rate: "epic" appears in ~5% of issues
    run_benchmark "search_very_low_match" "$WK_BIN" search "epic"

    # No match: nonsense query
    run_benchmark "search_no_match" "$WK_BIN" search "xyznonexistent123"
}

# ============================================================================
# Search with Filters Benchmarks
# ============================================================================

# Benchmark search with status filter
benchmark_search_status_filter() {
    info "Benchmarking search with status filter..."

    setup_db large

    run_benchmark "search_status_todo" "$WK_BIN" search "task" --status todo
    run_benchmark "search_status_in_progress" "$WK_BIN" search "task" --status in_progress
    run_benchmark "search_status_done" "$WK_BIN" search "task" --status done
}

# Benchmark search with type filter
benchmark_search_type_filter() {
    info "Benchmarking search with type filter..."

    setup_db large

    run_benchmark "search_type_task" "$WK_BIN" search "Issue" --type task
    run_benchmark "search_type_bug" "$WK_BIN" search "Issue" --type bug
    run_benchmark "search_type_feature" "$WK_BIN" search "Issue" --type feature
}

# Benchmark search with label filter
benchmark_search_label_filter() {
    info "Benchmarking search with label filter..."

    setup_db large

    run_benchmark "search_label_project" "$WK_BIN" search "task" --label project:alpha
    run_benchmark "search_label_priority" "$WK_BIN" search "task" --label priority:1
    run_benchmark "search_label_area" "$WK_BIN" search "task" --label area:frontend
}

# Benchmark search with assignee filter
benchmark_search_assignee_filter() {
    info "Benchmarking search with assignee filter..."

    setup_db large

    run_benchmark "search_assignee_alice" "$WK_BIN" search "task" --assignee alice
    run_benchmark "search_assignee_bob" "$WK_BIN" search "task" --assignee bob
    run_benchmark "search_unassigned" "$WK_BIN" search "task" --unassigned
}

# Benchmark search with combined filters
benchmark_search_combined_filters() {
    info "Benchmarking search with combined filters..."

    setup_db large

    # Status + type
    run_benchmark "search_combined_status_type" \
        "$WK_BIN" search "Issue" --status todo --type task

    # Status + label
    run_benchmark "search_combined_status_label" \
        "$WK_BIN" search "task" --status todo --label project:alpha

    # All filters combined
    run_benchmark "search_combined_all" \
        "$WK_BIN" search "task" --status todo --type task --label project:alpha --assignee alice
}

# ============================================================================
# Search Limit Benchmarks
# ============================================================================

# Benchmark search with different result limits
benchmark_search_limits() {
    info "Benchmarking search result limits..."

    setup_db large

    # Default limit (25)
    run_benchmark "search_limit_default" "$WK_BIN" search "task"

    # Custom limits
    run_benchmark "search_limit_10" "$WK_BIN" search "task" --limit 10
    run_benchmark "search_limit_50" "$WK_BIN" search "task" --limit 50
    run_benchmark "search_limit_100" "$WK_BIN" search "task" --limit 100

    # Unlimited
    run_benchmark "search_limit_unlimited" "$WK_BIN" search "task" --limit 0
}

# ============================================================================
# Search Output Format Benchmarks
# ============================================================================

# Benchmark search output format overhead
benchmark_search_output() {
    info "Benchmarking search output formats..."

    setup_db large

    # Compare text vs JSON output
    run_comparison "search_output_comparison" \
        "$WK_BIN search task" \
        "$WK_BIN search task --output json"

    # Individual measurements
    run_benchmark "search_output_text" "$WK_BIN" search "task"
    run_benchmark "search_output_json" "$WK_BIN" search "task" --output json
}

# ============================================================================
# Complex Query Benchmarks
# ============================================================================

# Benchmark complex/realistic search patterns
benchmark_search_complex() {
    info "Benchmarking complex search queries..."

    setup_db large

    # Multi-word query
    run_benchmark "search_multiword" "$WK_BIN" search "Issue task"

    # Query with project name (matches labels and titles)
    run_benchmark "search_project_name" "$WK_BIN" search "alpha"

    # Query with assignee name (might match notes/mentions)
    run_benchmark "search_person_name" "$WK_BIN" search "alice"

    # Numeric query (issue numbers in titles)
    run_benchmark "search_numeric" "$WK_BIN" search "100"

    # Short query (1-2 chars) - stress test
    run_benchmark "search_short_query" "$WK_BIN" search "a"
}

# ============================================================================
# Search Scaling Analysis
# ============================================================================

# Compare search performance vs list performance
benchmark_search_vs_list() {
    info "Benchmarking search vs list performance..."

    setup_db large

    # Compare search-then-filter vs list-with-filter
    run_comparison "search_vs_list_comparison" \
        "$WK_BIN search task --status todo" \
        "$WK_BIN list --status todo"
}

# ============================================================================
# Main Entry Point
# ============================================================================

# Run all search benchmarks
run_search_benchmarks() {
    info "=== Search Command Benchmarks ==="

    benchmark_search_basic
    benchmark_search_match_rates
    benchmark_search_status_filter
    benchmark_search_type_filter
    benchmark_search_label_filter
    benchmark_search_assignee_filter
    benchmark_search_combined_filters
    benchmark_search_limits
    benchmark_search_output
    benchmark_search_complex
    benchmark_search_vs_list

    success "Search command benchmarks complete"
}

# Run a subset of search benchmarks (for quick validation)
run_search_quick() {
    info "=== Search Quick Benchmarks ==="

    benchmark_search_basic
    benchmark_search_limits

    success "Search quick benchmarks complete"
}
