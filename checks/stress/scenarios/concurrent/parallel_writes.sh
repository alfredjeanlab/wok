#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Parallel Writes Stress Test
#
# Multiple processes creating issues simultaneously.
# Tests database locking and concurrent write handling.

stress_parallel_writes() {
    local writers="${1:-10}"
    local issues_per_writer="${2:-100}"

    echo "=== Parallel Writes Stress Test ==="
    echo "Writers: $writers"
    echo "Issues per writer: $issues_per_writer"
    echo "Total expected: $((writers * issues_per_writer))"

    init_workspace "stress"

    local start_time
    start_time=$(date +%s.%N)

    # Launch parallel writers
    for w in $(seq 1 "$writers"); do
        (
            for i in $(seq 1 "$issues_per_writer"); do
                "$WK_BIN" new task "Writer $w Issue $i" >/dev/null 2>&1
            done
        ) &
    done

    # Wait for all writers
    echo "Waiting for writers to complete..."
    wait

    local end_time
    end_time=$(date +%s.%N)
    local elapsed
    elapsed=$(echo "$end_time - $start_time" | bc)

    section "Results"

    local expected=$((writers * issues_per_writer))
    local actual
    actual=$(get_issue_count)

    echo "  Expected issues: $expected"
    echo "  Actual issues: $actual"
    echo "  Time: ${elapsed}s"

    if [ "$actual" -gt 0 ]; then
        local rate
        rate=$(echo "scale=2; $actual / $elapsed" | bc)
        echo "  Rate: $rate issues/sec"
    fi

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$actual" -eq "$expected" ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "All $expected issues created"
    else
        local missing=$((expected - actual))
        print_result "FAIL" "Data loss: $missing issues missing"
    fi
}
