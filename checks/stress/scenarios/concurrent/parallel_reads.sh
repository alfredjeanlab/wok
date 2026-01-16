#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Parallel Reads Stress Test
#
# Multiple processes reading from database simultaneously.
# Tests read concurrency and consistency.

stress_parallel_reads() {
    local readers="${1:-10}"
    local reads_per_reader="${2:-50}"

    echo "=== Parallel Reads Stress Test ==="
    echo "Readers: $readers"
    echo "Reads per reader: $reads_per_reader"

    init_workspace "stress"

    # Pre-populate database
    local prep_count=100
    echo "Pre-populating with $prep_count issues..."
    for i in $(seq 1 "$prep_count"); do
        "$WK_BIN" new task "Read test issue $i" >/dev/null 2>&1
    done

    local issue_count
    issue_count=$(get_issue_count)
    echo "Issues in database: $issue_count"

    local start_time
    start_time=$(date +%s.%N)
    local total_reads=0
    local errors=0

    # Create temp files for results
    local results_dir
    results_dir=$(mktemp -d)

    # Launch parallel readers
    for r in $(seq 1 "$readers"); do
        (
            local success=0
            local fail=0
            for _ in $(seq 1 "$reads_per_reader"); do
                # Randomly choose between list and show
                if [ $((RANDOM % 2)) -eq 0 ]; then
                    if "$WK_BIN" list >/dev/null 2>&1; then
                        ((success++))
                    else
                        ((fail++))
                    fi
                else
                    local id
                    id=$(sqlite3 .wok/issues.db "SELECT id FROM issues ORDER BY RANDOM() LIMIT 1" 2>/dev/null)
                    if [ -n "$id" ] && "$WK_BIN" show "$id" >/dev/null 2>&1; then
                        ((success++))
                    else
                        ((fail++))
                    fi
                fi
            done
            echo "$success $fail" > "$results_dir/reader_$r"
        ) &
    done

    echo "Waiting for readers to complete..."
    wait

    local end_time
    end_time=$(date +%s.%N)
    local elapsed
    elapsed=$(echo "$end_time - $start_time" | bc)

    # Aggregate results
    for f in "$results_dir"/reader_*; do
        if [ -f "$f" ]; then
            read -r success fail < "$f"
            total_reads=$((total_reads + success))
            errors=$((errors + fail))
        fi
    done
    rm -rf "$results_dir"

    section "Results"

    local expected=$((readers * reads_per_reader))
    echo "  Total reads attempted: $expected"
    echo "  Successful reads: $total_reads"
    echo "  Failed reads: $errors"
    echo "  Time: ${elapsed}s"

    if [ "$total_reads" -gt 0 ]; then
        local rate
        rate=$(echo "scale=2; $total_reads / $elapsed" | bc)
        echo "  Rate: $rate reads/sec"
    fi

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    local error_percent=0
    if [ "$expected" -gt 0 ]; then
        error_percent=$((errors * 100 / expected))
    fi

    if [ "$error_percent" -lt 5 ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "$total_reads successful reads"
    else
        print_result "FAIL" "Error rate: ${error_percent}%"
    fi
}
