#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/run.sh - Main benchmark runner for wk

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common utilities
source "$SCRIPT_DIR/lib/common.sh"

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] <COMMAND>

Run wk benchmarks using hyperfine.

COMMANDS:
    all             Run all benchmarks
    list            Run list command benchmarks (default, all, limit)
    filter          Run filter benchmarks (status, type, label, etc.)
    combined        Run combined filter benchmarks
    output          Run output format benchmarks
    generate        Generate test databases (requires writing to setup/)
    report          Generate benchmark report

OPTIONS:
    -h, --help      Show this help message
    -s, --size      Database size to use: small, medium, large, xlarge (default: large)
    -v, --verbose   Enable verbose output
    -d, --dry-run   Show what would be run without executing

ENVIRONMENT:
    WK_BIN          Path to wk binary (default: wk)
    RESULTS_DIR     Directory for results (default: checks/benchmarks/results)

EXAMPLES:
    # Run all benchmarks with release binary
    WK_BIN=./target/release/wk ./checks/benchmarks/run.sh all

    # Run only filter benchmarks on medium database
    ./checks/benchmarks/run.sh -s medium filter

    # Generate test databases
    WK_BIN=./target/release/wk ./checks/benchmarks/run.sh generate
EOF
}

# Default options
SIZE="large"
VERBOSE=false
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        -s|--size)
            SIZE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -*)
            error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            break
            ;;
    esac
done

COMMAND="${1:-}"

if [[ -z "$COMMAND" ]]; then
    error "No command specified"
    usage
    exit 1
fi

# Validate size
case "$SIZE" in
    small|medium|large|xlarge) ;;
    *)
        error "Invalid size: $SIZE (must be small, medium, large, or xlarge)"
        exit 1
        ;;
esac

# Source scenario files
source "$SCRIPT_DIR/scenarios/list.sh"

run_all() {
    info "Running all benchmarks..."
    check_dependencies
    check_wk_binary

    run_list_benchmarks
    run_filter_benchmarks
    run_combined_benchmarks
    run_output_benchmarks

    success "All benchmarks complete!"
    info "Results in: $RESULTS_DIR"
}

run_list() {
    info "Running list benchmarks..."
    check_dependencies
    check_wk_binary
    run_list_benchmarks
}

run_filter() {
    info "Running filter benchmarks..."
    check_dependencies
    check_wk_binary
    run_filter_benchmarks
}

run_combined() {
    info "Running combined filter benchmarks..."
    check_dependencies
    check_wk_binary
    run_combined_benchmarks
}

run_output() {
    info "Running output format benchmarks..."
    check_dependencies
    check_wk_binary
    run_output_benchmarks
}

run_generate() {
    info "Generating test databases..."
    check_dependencies
    check_wk_binary
    source "$SCRIPT_DIR/setup/generate_db.sh"
    generate_all_databases
}

run_report() {
    info "Generating report..."
    source "$SCRIPT_DIR/lib/report.sh"
    generate_report
}

# Execute command
case "$COMMAND" in
    all)
        run_all
        ;;
    list)
        run_list
        ;;
    filter)
        run_filter
        ;;
    combined)
        run_combined
        ;;
    output)
        run_output
        ;;
    generate)
        run_generate
        ;;
    report)
        run_report
        ;;
    *)
        error "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac
