#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Mixed Workload Stress Test
#
# Simultaneous readers, writers, and modifiers.
# Tests database under realistic concurrent load.

stress_mixed_workload() {
    local duration="${1:-30}"

    echo "=== Mixed Workload Stress Test ==="
    echo "Duration: ${duration}s"

    init_workspace "stress"

    # Pre-populate with some issues
    local prep_count=100
    echo "Pre-populating with $prep_count issues..."
    for i in $(seq 1 "$prep_count"); do
        "$WK_BIN" new task "Initial $i" >/dev/null 2>&1
    done

    local end_time=$(($(date +%s) + duration))

    # Create temp files for logging
    local log_dir
    log_dir=$(mktemp -d)

    echo "Starting mixed workload..."

    # Writer process
    (
        while [ "$(date +%s)" -lt "$end_time" ]; do
            if "$WK_BIN" new task "Dynamic $(date +%s%N)" >/dev/null 2>&1; then
                echo "W"
            else
                echo "E"
            fi
        done
    ) > "$log_dir/writes.log" &
    local writer_pid=$!

    # Reader process 1 - list
    (
        while [ "$(date +%s)" -lt "$end_time" ]; do
            if "$WK_BIN" list >/dev/null 2>&1; then
                echo "R"
            else
                echo "E"
            fi
        done
    ) > "$log_dir/reads1.log" &

    # Reader process 2 - show random
    (
        while [ "$(date +%s)" -lt "$end_time" ]; do
            local id
            id=$(sqlite3 .wok/issues.db "SELECT id FROM issues ORDER BY RANDOM() LIMIT 1" 2>/dev/null)
            if [ -n "$id" ] && "$WK_BIN" show "$id" >/dev/null 2>&1; then
                echo "R"
            else
                echo "E"
            fi
            sleep 0.1
        done
    ) > "$log_dir/reads2.log" &

    # Modifier process - start/done
    (
        while [ "$(date +%s)" -lt "$end_time" ]; do
            local id
            id=$(sqlite3 .wok/issues.db "SELECT id FROM issues WHERE status='todo' ORDER BY RANDOM() LIMIT 1" 2>/dev/null)
            if [ -n "$id" ]; then
                "$WK_BIN" start "$id" >/dev/null 2>&1
                if "$WK_BIN" done "$id" >/dev/null 2>&1; then
                    echo "M"
                else
                    echo "E"
                fi
            fi
            sleep 0.2
        done
    ) > "$log_dir/mods.log" &

    # Labeler process
    (
        while [ "$(date +%s)" -lt "$end_time" ]; do
            local id
            id=$(sqlite3 .wok/issues.db "SELECT id FROM issues ORDER BY RANDOM() LIMIT 1" 2>/dev/null)
            if [ -n "$id" ]; then
                if "$WK_BIN" label "$id" "stress:test" >/dev/null 2>&1; then
                    echo "L"
                else
                    echo "E"
                fi
            fi
            sleep 0.3
        done
    ) > "$log_dir/labels.log" &

    # Wait for all processes
    wait

    section "Results"

    # Tally results
    local writes reads mods labels errors

    writes=$(grep -c "W" "$log_dir/writes.log" 2>/dev/null || echo 0)
    reads=$(( $(grep -c "R" "$log_dir/reads1.log" 2>/dev/null || echo 0) + $(grep -c "R" "$log_dir/reads2.log" 2>/dev/null || echo 0) ))
    mods=$(grep -c "M" "$log_dir/mods.log" 2>/dev/null || echo 0)
    labels=$(grep -c "L" "$log_dir/labels.log" 2>/dev/null || echo 0)
    errors=$(cat "$log_dir"/*.log 2>/dev/null | grep -c "E" || echo 0)

    local total_ops=$((writes + reads + mods + labels))

    echo "  Writes: $writes"
    echo "  Reads: $reads"
    echo "  Modifications: $mods"
    echo "  Labels: $labels"
    echo "  Errors: $errors"
    echo "  Total ops: $total_ops"
    echo "  Ops/sec: $((total_ops / duration))"

    rm -rf "$log_dir"

    # Verify database integrity
    section "Integrity Check"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database: $integrity"

    local final_count
    final_count=$(get_issue_count)
    echo "  Final issue count: $final_count"

    local error_percent=0
    if [ "$total_ops" -gt 0 ]; then
        error_percent=$((errors * 100 / total_ops))
    fi

    if [ "$integrity" = "OK" ] && [ "$error_percent" -lt 10 ]; then
        print_result "PASS" "$total_ops operations, ${error_percent}% errors"
    else
        print_result "FAIL" "Integrity: $integrity, errors: ${error_percent}%"
    fi
}
