#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Main runner for wk CLI stress tests
#
# Usage: ./run.sh [OPTIONS] [SCENARIO...]
#
# This script can be run directly (uses ulimit for safety) or inside Docker
# (recommended - uses hard resource limits).

set -euo pipefail

STRESS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat << EOF
Usage: $0 [OPTIONS] [SCENARIO...]

Stress test suite for wk CLI implementations.

Options:
    -b, --binary PATH    Path to wk binary (default: \$WK_BIN or 'wk')
    -o, --output DIR     Output directory for results
    -q, --quick          Run quick versions of tests (smaller counts)
    -h, --help           Show this help

Scenarios:
    scale       Scale tests (massive_db, deep_deps, wide_deps, many_labels, many_notes)
    limits      Limit tests (title_length, note_length, label_length, id_collisions, path_depth)
    concurrent  Concurrent access tests (parallel_writes, parallel_reads, mixed_workload, lock_contention)
    corruption  Corruption/recovery tests (interrupted_write, disk_full, corrupt_db, corrupt_config)
    memory      Memory stress tests (memory_limit, large_export, list_all)
    all         Run all scenarios (default)

Individual tests:
    massive_db [count]      Create count issues (default: 100000)
    deep_deps [depth]       Create dependency chain of depth (default: 1000)
    wide_deps [width]       Create issue blocking width others (default: 1000)
    many_labels [count]     Create issues with many labels (default: 100)
    many_notes [count]      Create issues with many notes (default: 100)
    parallel_writes [n] [c] n writers, c issues each (default: 10, 100)
    mixed_workload [secs]   Run mixed workload for secs (default: 30)

Examples:
    # Run all stress tests
    WK_BIN=./wk $0

    # Run only scale tests (quick version)
    WK_BIN=./wk $0 -q scale

    # Create massive database
    WK_BIN=./wk $0 massive_db 50000
EOF
}

# Parse arguments
export WK_BIN="${WK_BIN:-wk}"
OUTPUT_DIR=""
QUICK_MODE=0
SCENARIOS=()
SCENARIO_ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -b|--binary) WK_BIN="$2"; shift 2 ;;
        -o|--output) OUTPUT_DIR="$2"; shift 2 ;;
        -q|--quick) QUICK_MODE=1; shift ;;
        -h|--help) usage; exit 0 ;;
        -*) echo "Unknown option: $1"; usage; exit 1 ;;
        *)
            SCENARIOS+=("$1")
            shift
            # Capture remaining args as scenario args
            while [[ $# -gt 0 ]] && [[ ! "$1" =~ ^- ]]; do
                SCENARIO_ARGS+=("$1")
                shift
            done
            ;;
    esac
done

[ ${#SCENARIOS[@]} -eq 0 ] && SCENARIOS=(all)

# Print header
echo "=== wk CLI Stress Tests ==="
echo "Binary: $WK_BIN"
echo "Date: $(date)"
echo "Quick mode: $QUICK_MODE"
if [ -n "${STRESS_CONTAINERIZED:-}" ]; then
    echo "Environment: Docker container"
else
    echo "Environment: Native (ulimit)"
fi
echo ""

# Source common utilities (which sources safety.sh)
source "$STRESS_ROOT/lib/common.sh"
source "$STRESS_ROOT/lib/generators.sh"
source "$STRESS_ROOT/lib/monitors.sh"

# Verify wk binary
verify_wk_binary

# Set default counts based on mode
if [ "$QUICK_MODE" -eq 1 ]; then
    DEFAULT_MASSIVE_DB=1000
    DEFAULT_DEEP_DEPS=100
    DEFAULT_WIDE_DEPS=100
    DEFAULT_MANY_LABELS=20
    DEFAULT_MANY_NOTES=20
    DEFAULT_PARALLEL_WRITERS=5
    DEFAULT_PARALLEL_ISSUES=20
    DEFAULT_MIXED_DURATION=10
else
    DEFAULT_MASSIVE_DB=10000
    DEFAULT_DEEP_DEPS=500
    DEFAULT_WIDE_DEPS=500
    DEFAULT_MANY_LABELS=100
    DEFAULT_MANY_NOTES=100
    DEFAULT_PARALLEL_WRITERS=10
    DEFAULT_PARALLEL_ISSUES=100
    DEFAULT_MIXED_DURATION=30
fi

# Run scenarios
run_scenario() {
    local scenario="$1"
    shift
    local args=("$@")

    case $scenario in
        all)
            for s in scale limits concurrent corruption memory; do
                run_scenario "$s"
            done
            ;;

        scale)
            source "$STRESS_ROOT/scenarios/scale/massive_db.sh"
            source "$STRESS_ROOT/scenarios/scale/deep_deps.sh"
            source "$STRESS_ROOT/scenarios/scale/wide_deps.sh"
            source "$STRESS_ROOT/scenarios/scale/many_labels.sh"
            source "$STRESS_ROOT/scenarios/scale/many_notes.sh"

            stress_massive_db "$DEFAULT_MASSIVE_DB"
            stress_deep_dependency_chain "$DEFAULT_DEEP_DEPS"
            stress_wide_dependencies "$DEFAULT_WIDE_DEPS"
            stress_many_labels "$DEFAULT_MANY_LABELS"
            stress_many_notes "$DEFAULT_MANY_NOTES"
            ;;

        limits)
            source "$STRESS_ROOT/scenarios/limits/title_length.sh"
            source "$STRESS_ROOT/scenarios/limits/note_length.sh"
            source "$STRESS_ROOT/scenarios/limits/label_length.sh"
            source "$STRESS_ROOT/scenarios/limits/id_collisions.sh"
            source "$STRESS_ROOT/scenarios/limits/path_depth.sh"

            stress_title_length
            stress_note_length
            stress_label_length
            stress_id_collisions
            stress_path_depth
            ;;

        concurrent)
            source "$STRESS_ROOT/scenarios/concurrent/parallel_writes.sh"
            source "$STRESS_ROOT/scenarios/concurrent/parallel_reads.sh"
            source "$STRESS_ROOT/scenarios/concurrent/mixed_workload.sh"
            source "$STRESS_ROOT/scenarios/concurrent/lock_contention.sh"

            stress_parallel_writes "$DEFAULT_PARALLEL_WRITERS" "$DEFAULT_PARALLEL_ISSUES"
            stress_parallel_reads
            stress_mixed_workload "$DEFAULT_MIXED_DURATION"
            stress_lock_contention
            ;;

        corruption)
            source "$STRESS_ROOT/scenarios/corruption/interrupted_write.sh"
            source "$STRESS_ROOT/scenarios/corruption/corrupt_db.sh"
            source "$STRESS_ROOT/scenarios/corruption/corrupt_config.sh"

            stress_interrupted_write
            stress_corrupt_db
            stress_corrupt_config

            # disk_full is dangerous, skip by default
            if [ "${STRESS_SKIP_DANGEROUS:-1}" -ne 1 ]; then
                source "$STRESS_ROOT/scenarios/corruption/disk_full.sh"
                stress_disk_full
            else
                echo "Skipping disk_full test (set STRESS_SKIP_DANGEROUS=0 to enable)"
            fi
            ;;

        memory)
            source "$STRESS_ROOT/scenarios/memory/memory_limit.sh"
            source "$STRESS_ROOT/scenarios/memory/large_export.sh"
            source "$STRESS_ROOT/scenarios/memory/list_all.sh"

            stress_memory_limit
            stress_large_export
            stress_list_all
            ;;

        # Individual tests
        massive_db)
            source "$STRESS_ROOT/scenarios/scale/massive_db.sh"
            stress_massive_db "${args[0]:-$DEFAULT_MASSIVE_DB}"
            ;;

        deep_deps)
            source "$STRESS_ROOT/scenarios/scale/deep_deps.sh"
            stress_deep_dependency_chain "${args[0]:-$DEFAULT_DEEP_DEPS}"
            ;;

        wide_deps)
            source "$STRESS_ROOT/scenarios/scale/wide_deps.sh"
            stress_wide_dependencies "${args[0]:-$DEFAULT_WIDE_DEPS}"
            ;;

        many_labels)
            source "$STRESS_ROOT/scenarios/scale/many_labels.sh"
            stress_many_labels "${args[0]:-$DEFAULT_MANY_LABELS}"
            ;;

        many_notes)
            source "$STRESS_ROOT/scenarios/scale/many_notes.sh"
            stress_many_notes "${args[0]:-$DEFAULT_MANY_NOTES}"
            ;;

        parallel_writes)
            source "$STRESS_ROOT/scenarios/concurrent/parallel_writes.sh"
            stress_parallel_writes "${args[0]:-$DEFAULT_PARALLEL_WRITERS}" "${args[1]:-$DEFAULT_PARALLEL_ISSUES}"
            ;;

        parallel_reads)
            source "$STRESS_ROOT/scenarios/concurrent/parallel_reads.sh"
            stress_parallel_reads
            ;;

        mixed_workload)
            source "$STRESS_ROOT/scenarios/concurrent/mixed_workload.sh"
            stress_mixed_workload "${args[0]:-$DEFAULT_MIXED_DURATION}"
            ;;

        lock_contention)
            source "$STRESS_ROOT/scenarios/concurrent/lock_contention.sh"
            stress_lock_contention
            ;;

        title_length)
            source "$STRESS_ROOT/scenarios/limits/title_length.sh"
            stress_title_length
            ;;

        id_collisions)
            source "$STRESS_ROOT/scenarios/limits/id_collisions.sh"
            stress_id_collisions
            ;;

        interrupted_write)
            source "$STRESS_ROOT/scenarios/corruption/interrupted_write.sh"
            stress_interrupted_write
            ;;

        corrupt_db)
            source "$STRESS_ROOT/scenarios/corruption/corrupt_db.sh"
            stress_corrupt_db
            ;;

        disk_full)
            if [ "${STRESS_SKIP_DANGEROUS:-1}" -ne 1 ]; then
                source "$STRESS_ROOT/scenarios/corruption/disk_full.sh"
                stress_disk_full
            else
                echo "disk_full test skipped (set STRESS_SKIP_DANGEROUS=0 to enable)"
            fi
            ;;

        memory_limit)
            source "$STRESS_ROOT/scenarios/memory/memory_limit.sh"
            stress_memory_limit "${args[0]:-100}"
            ;;

        large_export)
            source "$STRESS_ROOT/scenarios/memory/large_export.sh"
            stress_large_export
            ;;

        list_all)
            source "$STRESS_ROOT/scenarios/memory/list_all.sh"
            stress_list_all
            ;;

        *)
            echo "Unknown scenario: $scenario"
            exit 1
            ;;
    esac
}

# Run each scenario
for scenario in "${SCENARIOS[@]}"; do
    run_scenario "$scenario" "${SCENARIO_ARGS[@]+"${SCENARIO_ARGS[@]}"}"
done

echo ""
echo "=== All tests completed ==="
