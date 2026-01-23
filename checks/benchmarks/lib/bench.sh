#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/lib/bench.sh - Hyperfine benchmark wrappers

# This file is sourced by run.sh after common.sh
# It provides benchmark-specific functions extracted from common.sh

# ============================================================================
# Standard Benchmark Wrappers
# ============================================================================

# Run a benchmark with hyperfine using standard options
# Usage: run_benchmark <name> <command...>
# Example: run_benchmark "list_default" "$WK_BIN" list
run_benchmark() {
    local name="$1"
    shift
    local cmd="$*"  # Join args into single command string

    local output_file="$RESULTS_DIR/${name}.json"
    mkdir -p "$RESULTS_DIR"

    info "Running benchmark: $name"
    hyperfine \
        --warmup 3 \
        --min-runs 30 \
        --shell=none \
        --export-json "$output_file" \
        "$cmd"

    success "Results saved to: $output_file"
}

# Run a cold-start benchmark (no warmup)
# Usage: run_benchmark_cold <name> <command...>
run_benchmark_cold() {
    local name="$1"
    shift
    local cmd="$*"  # Join args into single command string

    local output_file="$RESULTS_DIR/${name}.json"
    mkdir -p "$RESULTS_DIR"

    info "Running cold-start benchmark: $name"
    hyperfine \
        --warmup 0 \
        --min-runs 20 \
        --shell=none \
        --export-json "$output_file" \
        "$cmd"

    success "Results saved to: $output_file"
}

# Run a comparative benchmark with multiple commands
# Usage: run_comparison <name> <cmd1> <cmd2> ...
# Note: Uses shell to parse commands, since each arg is a full command string
run_comparison() {
    local name="$1"
    shift

    local output_file="$RESULTS_DIR/${name}.json"
    mkdir -p "$RESULTS_DIR"

    info "Running comparison benchmark: $name"
    hyperfine \
        --warmup 3 \
        --min-runs 30 \
        --export-json "$output_file" \
        "$@"

    success "Results saved to: $output_file"
}

# ============================================================================
# Mutation Benchmark Wrappers
# ============================================================================
# Mutations modify database state, so they need special handling with
# hyperfine's --prepare flag to restore DB between runs.

# Run a mutation benchmark with DB restoration between runs
# Usage: run_benchmark_mutation <name> <size> <command...>
# Example: run_benchmark_mutation "new_single" "large" "$WK_BIN" new task "Test"
run_benchmark_mutation() {
    local name="$1"
    local size="$2"
    shift 2
    local cmd="$*"

    local output_file="$RESULTS_DIR/${name}.json"
    local sql_file="$SCRIPT_DIR/setup/${size}.sql"

    if [[ ! -f "$sql_file" ]]; then
        error "Database file not found: $sql_file"
        error "Run 'generate_db.sh $size' first"
        return 1
    fi

    mkdir -p "$RESULTS_DIR"

    info "Running mutation benchmark: $name (with DB restore)"
    hyperfine \
        --warmup 3 \
        --min-runs 30 \
        --prepare "rm -rf .wok && mkdir -p .wok && sqlite3 .wok/issues.db < $sql_file && echo 'prefix = \"bench\"' > .wok/config.toml" \
        --shell=none \
        --export-json "$output_file" \
        "$cmd"

    success "Results saved to: $output_file"
}

# Run a mutation benchmark with cold start (no warmup)
# Usage: run_benchmark_mutation_cold <name> <size> <command...>
run_benchmark_mutation_cold() {
    local name="$1"
    local size="$2"
    shift 2
    local cmd="$*"

    local output_file="$RESULTS_DIR/${name}.json"
    local sql_file="$SCRIPT_DIR/setup/${size}.sql"

    if [[ ! -f "$sql_file" ]]; then
        error "Database file not found: $sql_file"
        error "Run 'generate_db.sh $size' first"
        return 1
    fi

    mkdir -p "$RESULTS_DIR"

    info "Running cold mutation benchmark: $name (with DB restore)"
    hyperfine \
        --warmup 0 \
        --min-runs 20 \
        --prepare "rm -rf .wok && mkdir -p .wok && sqlite3 .wok/issues.db < $sql_file && echo 'prefix = \"bench\"' > .wok/config.toml" \
        --shell=none \
        --export-json "$output_file" \
        "$cmd"

    success "Results saved to: $output_file"
}

# Run a batch operation benchmark (multiple operations in sequence)
# Usage: run_benchmark_batch <name> <size> <count> <shell_command>
# Example: run_benchmark_batch "new_batch_10" "large" 10 'for i in $(seq 1 10); do wk new task "Task $i"; done'
# Note: Uses shell mode since we need to run multiple commands
run_benchmark_batch() {
    local name="$1"
    local size="$2"
    local count="$3"
    local shell_cmd="$4"

    local output_file="$RESULTS_DIR/${name}.json"
    local sql_file="$SCRIPT_DIR/setup/${size}.sql"

    if [[ ! -f "$sql_file" ]]; then
        error "Database file not found: $sql_file"
        error "Run 'generate_db.sh $size' first"
        return 1
    fi

    mkdir -p "$RESULTS_DIR"

    info "Running batch benchmark: $name (count=$count, with DB restore)"
    hyperfine \
        --warmup 1 \
        --min-runs 10 \
        --prepare "rm -rf .wok && mkdir -p .wok && sqlite3 .wok/issues.db < $sql_file && echo 'prefix = \"bench\"' > .wok/config.toml" \
        --export-json "$output_file" \
        "$shell_cmd"

    success "Results saved to: $output_file"
}

# ============================================================================
# Result Extraction Functions
# ============================================================================

# Parse mean time from a benchmark result JSON
# Usage: get_mean <result_file>
get_mean() {
    local file="$1"
    jq -r '.results[0].mean' "$file"
}

# Parse stddev from a benchmark result JSON
# Usage: get_stddev <result_file>
get_stddev() {
    local file="$1"
    jq -r '.results[0].stddev' "$file"
}

# Get the p95 (approximated as mean + 2*stddev) from a benchmark result
# Usage: get_p95 <result_file>
get_p95() {
    local file="$1"
    local mean stddev
    mean=$(get_mean "$file")
    stddev=$(get_stddev "$file")
    echo "$mean + 2 * $stddev" | bc -l
}

# Format time in milliseconds
# Usage: format_ms <seconds>
format_ms() {
    local seconds="$1"
    echo "scale=1; $seconds * 1000" | bc -l
}

# Get minimum time from a benchmark result JSON
# Usage: get_min <result_file>
get_min() {
    local file="$1"
    jq -r '.results[0].min' "$file"
}

# Get maximum time from a benchmark result JSON
# Usage: get_max <result_file>
get_max() {
    local file="$1"
    jq -r '.results[0].max' "$file"
}

# Get median time from a benchmark result JSON
# Usage: get_median <result_file>
get_median() {
    local file="$1"
    jq -r '.results[0].median' "$file"
}
