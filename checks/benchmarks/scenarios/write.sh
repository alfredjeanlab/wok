#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/scenarios/write.sh - Write operation benchmark scenarios

# This file is sourced by run.sh, which already sources common.sh and bench.sh

# ============================================================================
# New Issue Benchmarks
# ============================================================================

# Benchmark creating a single issue (sequential)
benchmark_new_sequential() {
    info "Benchmarking sequential issue creation..."

    for size in small medium large xlarge; do
        run_benchmark_mutation "new_sequential_${size}" "$size" \
            "$WK_BIN" new task "Benchmark test issue"
    done
}

# Benchmark batch creation of multiple issues
# Uses shell mode to run multiple commands in sequence
benchmark_new_batch() {
    info "Benchmarking batch issue creation..."

    local size="large"

    for count in 10 50 100; do
        local cmd="for i in \$(seq 1 $count); do $WK_BIN new task \"Batch task \$i\"; done"
        run_benchmark_batch "new_batch_${count}" "$size" "$count" "$cmd"
    done
}

# ============================================================================
# Edit Operation Benchmarks
# ============================================================================

# Benchmark editing issue title
# Note: Requires an existing issue ID. We use the first issue in the DB.
benchmark_edit_title() {
    info "Benchmarking title edit..."

    setup_db large

    # Get an existing issue ID to edit
    local id
    id=$("$WK_BIN" list --limit 1 --format json | jq -r '.issues[0].id')

    if [[ -z "$id" || "$id" == "null" ]]; then
        error "No issues found in database"
        return 1
    fi

    run_benchmark_mutation "edit_title" "large" \
        "$WK_BIN" edit "$id" title "Updated title for benchmark"
}

# Benchmark editing issue type
benchmark_edit_type() {
    info "Benchmarking type edit..."

    setup_db large

    # Get an existing task to change to bug
    local id
    id=$("$WK_BIN" list --type task --limit 1 --format json | jq -r '.issues[0].id')

    if [[ -z "$id" || "$id" == "null" ]]; then
        error "No task issues found in database"
        return 1
    fi

    run_benchmark_mutation "edit_type" "large" \
        "$WK_BIN" edit "$id" type bug
}

# Benchmark editing issue assignee
benchmark_edit_assignee() {
    info "Benchmarking assignee edit..."

    setup_db large

    # Get an existing issue
    local id
    id=$("$WK_BIN" list --limit 1 --format json | jq -r '.issues[0].id')

    if [[ -z "$id" || "$id" == "null" ]]; then
        error "No issues found in database"
        return 1
    fi

    run_benchmark_mutation "edit_assignee" "large" \
        "$WK_BIN" edit "$id" assignee benchuser
}

# ============================================================================
# Close/Done Operation Benchmarks
# ============================================================================

# Benchmark closing a single issue
benchmark_close_single() {
    info "Benchmarking single close..."

    setup_db large

    # Get an in_progress issue (can be closed)
    local id
    id=$("$WK_BIN" list --status in_progress --limit 1 --format json | jq -r '.issues[0].id')

    if [[ -z "$id" || "$id" == "null" ]]; then
        error "No in_progress issues found in database"
        return 1
    fi

    run_benchmark_mutation "close_single" "large" \
        "$WK_BIN" close "$id" -r "Benchmark close"
}

# Benchmark closing multiple issues in batch
benchmark_close_batch() {
    info "Benchmarking batch close..."

    for count in 10 50; do
        setup_db large

        # Get multiple in_progress issues
        local ids
        ids=$("$WK_BIN" list --status in_progress --limit "$count" --format json | jq -r '.issues[].id' | tr '\n' ' ')

        if [[ -z "$ids" ]]; then
            warn "Not enough in_progress issues for batch close of $count"
            continue
        fi

        # Build the command with all IDs
        local cmd="$WK_BIN close $ids -r 'Benchmark batch close'"
        run_benchmark_batch "close_batch_${count}" "large" "$count" "$cmd"
    done
}

# Benchmark completing (done) a single issue
benchmark_done_single() {
    info "Benchmarking single done..."

    setup_db large

    # Get an in_progress issue (can transition to done)
    local id
    id=$("$WK_BIN" list --status in_progress --limit 1 --format json | jq -r '.issues[0].id')

    if [[ -z "$id" || "$id" == "null" ]]; then
        error "No in_progress issues found in database"
        return 1
    fi

    run_benchmark_mutation "done_single" "large" \
        "$WK_BIN" done "$id"
}

# ============================================================================
# Start Operation Benchmarks
# ============================================================================

# Benchmark starting a single issue (todo -> in_progress)
benchmark_start_single() {
    info "Benchmarking single start..."

    setup_db large

    # Get a todo issue (can transition to in_progress)
    local id
    id=$("$WK_BIN" list --status todo --limit 1 --format json | jq -r '.issues[0].id')

    if [[ -z "$id" || "$id" == "null" ]]; then
        error "No todo issues found in database"
        return 1
    fi

    run_benchmark_mutation "start_single" "large" \
        "$WK_BIN" start "$id"
}

# ============================================================================
# Main Entry Point
# ============================================================================

# Run all write operation benchmarks
run_write_benchmarks() {
    info "=== Write Operation Benchmarks ==="

    benchmark_new_sequential
    benchmark_new_batch
    benchmark_edit_title
    benchmark_edit_type
    benchmark_edit_assignee
    benchmark_start_single
    benchmark_done_single
    benchmark_close_single
    benchmark_close_batch

    success "Write operation benchmarks complete"
}
