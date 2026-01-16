#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Deep Dependency Chain Stress Test
#
# Creates a linear chain of dependencies to test:
# - Cycle detection performance
# - Transitive blocking computation
# - Dependency resolution at depth

stress_deep_dependency_chain() {
    local depth="${1:-1000}"

    echo "=== Deep Dependency Chain Test ==="
    echo "Target depth: $depth"

    init_workspace "stress"

    local ids=()
    local prev_id=""

    echo "Creating chain..."
    local start_time
    start_time=$(date +%s)

    for i in $(seq 1 "$depth"); do
        local output
        output=$("$WK_BIN" new task "Chain link $i" 2>&1)
        local id
        id=$(extract_issue_id "$output")
        ids+=("$id")

        if [ -n "$prev_id" ]; then
            "$WK_BIN" dep "$prev_id" blocks "$id" >/dev/null 2>&1 || true
        fi
        prev_id="$id"

        # Progress
        if [ $((i % 100)) -eq 0 ]; then
            echo "  Created $i links..."

            # Health check
            if ! check_system_health; then
                echo "Stopping at $i due to resource pressure"
                depth=$i
                break
            fi
        fi
    done

    local create_time=$(($(date +%s) - start_time))
    echo "Chain created: $depth links in ${create_time}s"

    section "Testing Operations"

    # Test cycle detection - try to add edge that would create cycle
    echo "Cycle detection (adding edge from last to first):"
    local first_id="${ids[0]}"
    local last_id="${ids[-1]}"

    local cycle_start
    cycle_start=$(date +%s.%N)
    local cycle_result
    cycle_result=$("$WK_BIN" dep "$last_id" blocks "$first_id" 2>&1) || true
    local cycle_end
    cycle_end=$(date +%s.%N)
    local cycle_time
    cycle_time=$(echo "$cycle_end - $cycle_start" | bc)

    if echo "$cycle_result" | grep -qi "cycle\|circular"; then
        echo "  Cycle detected correctly in ${cycle_time}s"
    else
        echo "  WARNING: Cycle may not have been detected"
        echo "  Output: $cycle_result"
    fi

    # Test list performance (computes transitive blocking)
    echo "List (computes transitive blocking):"
    local list_time
    list_time=$(time_cmd "$WK_BIN" list)
    echo "  Time: ${list_time}s"

    # Test show on deeply nested issue
    echo "Show last issue (deeply blocked):"
    local show_time
    show_time=$(time_cmd "$WK_BIN" show "$last_id")
    echo "  Time: ${show_time}s"

    # Complete first issue and check cascading unblock
    echo "Completing first issue..."
    "$WK_BIN" start "$first_id" >/dev/null 2>&1 || true

    local done_start
    done_start=$(date +%s.%N)
    "$WK_BIN" done "$first_id" >/dev/null 2>&1 || true
    local done_end
    done_end=$(date +%s.%N)
    local done_time
    done_time=$(echo "$done_end - $done_start" | bc)
    echo "  Complete time: ${done_time}s"

    echo "List after completing first:"
    local list_after_time
    list_after_time=$(time_cmd "$WK_BIN" list)
    echo "  Time: ${list_after_time}s"

    section "Results"
    echo "  Chain depth: $depth"
    echo "  Create time: ${create_time}s"
    echo "  Cycle detection: ${cycle_time}s"
    echo "  List time: ${list_time}s"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Chain of $depth created and tested"
    else
        print_result "FAIL" "Database corruption detected"
    fi
}
