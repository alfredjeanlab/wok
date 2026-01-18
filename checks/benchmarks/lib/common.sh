#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/lib/common.sh - Shared utilities for wk benchmarks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WK_BIN="${WK_BIN:-wk}"
RESULTS_DIR="${RESULTS_DIR:-$SCRIPT_DIR/results}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[OK]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Setup a test database from a pre-generated SQL file
# Usage: setup_db <size>
# Example: setup_db large
setup_db() {
    local size="$1"
    local sql_file="$SCRIPT_DIR/setup/${size}.sql"

    if [[ ! -f "$sql_file" ]]; then
        error "Database file not found: $sql_file"
        error "Run 'generate_db.sh $size' first"
        return 1
    fi

    rm -rf .work
    mkdir -p .work
    sqlite3 .work/issues.db < "$sql_file"

    cat > .work/config.toml << EOF
prefix = "bench"
EOF
}

# Restore a database from SQL (same as setup_db but more explicit name)
# Usage: restore_db <size>
restore_db() {
    setup_db "$@"
}

# Run a benchmark with hyperfine using standard options
# Usage: run_benchmark <name> <command...>
# Example: run_benchmark "list_default" "$WK_BIN" list
run_benchmark() {
    local name="$1"
    shift
    local cmd="$*"  # Join args into single string for --shell=none

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
    local cmd="$*"  # Join args into single string for --shell=none

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
run_comparison() {
    local name="$1"
    shift

    local output_file="$RESULTS_DIR/${name}.json"
    mkdir -p "$RESULTS_DIR"

    info "Running comparison benchmark: $name"
    hyperfine \
        --warmup 3 \
        --min-runs 30 \
        --shell=none \
        --export-json "$output_file" \
        "$@"

    success "Results saved to: $output_file"
}

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

# Check if required tools are installed
check_dependencies() {
    local missing=()

    if ! command -v hyperfine &> /dev/null; then
        missing+=("hyperfine")
    fi

    if ! command -v jq &> /dev/null; then
        missing+=("jq")
    fi

    if ! command -v bc &> /dev/null; then
        missing+=("bc")
    fi

    if ! command -v sqlite3 &> /dev/null; then
        missing+=("sqlite3")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing required tools: ${missing[*]}"
        echo "Install with:"
        echo "  brew install ${missing[*]}"
        return 1
    fi

    success "All dependencies found"
}

# Verify the wk binary exists and is executable
check_wk_binary() {
    if ! command -v "$WK_BIN" &> /dev/null; then
        error "wk binary not found: $WK_BIN"
        echo "Build with: cargo build --release"
        echo "Then set: WK_BIN=./target/release/wk"
        return 1
    fi

    success "Using wk binary: $(command -v "$WK_BIN")"
}
