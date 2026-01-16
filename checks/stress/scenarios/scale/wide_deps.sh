#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Wide Dependencies Stress Test
#
# Creates one issue that blocks many others (star pattern).
# Tests:
# - Show performance with many relationships
# - List filtering with many blocked issues
# - Unblocking many issues at once

stress_wide_dependencies() {
    local width="${1:-1000}"

    echo "=== Wide Dependencies Test ==="
    echo "Target: 1 issue blocking $width others"

    init_workspace "stress"

    # Create blocker
    local blocker_output
    blocker_output=$("$WK_BIN" new task "The Blocker" 2>&1)
    local blocker
    blocker=$(extract_issue_id "$blocker_output")
    echo "Created blocker: $blocker"

    # Create blocked issues
    echo "Creating $width dependent issues..."
    local start_time
    start_time=$(date +%s)
    local deps=()

    for i in $(seq 1 "$width"); do
        local output
        output=$("$WK_BIN" new task "Blocked $i" 2>&1)
        local id
        id=$(extract_issue_id "$output")
        deps+=("$id")

        if [ $((i % 100)) -eq 0 ]; then
            echo "  Created $i issues..."

            if ! check_system_health; then
                echo "Stopping at $i due to resource pressure"
                width=$i
                break
            fi
        fi
    done

    local create_time=$(($(date +%s) - start_time))
    echo "Issues created in ${create_time}s"

    # Add dependencies
    echo "Adding dependencies..."
    local dep_start
    dep_start=$(date +%s)

    for id in "${deps[@]}"; do
        "$WK_BIN" dep "$blocker" blocks "$id" >/dev/null 2>&1 || true
    done

    local dep_time=$(($(date +%s) - dep_start))
    echo "Dependencies added in ${dep_time}s"

    section "Testing Operations"

    echo "Show blocker (lists all blocked):"
    local show_time
    show_time=$(time_cmd "$WK_BIN" show "$blocker")
    echo "  Time: ${show_time}s"

    echo "List (should show 1 ready, $width blocked):"
    local list_time
    list_time=$(time_cmd "$WK_BIN" list)
    echo "  Time: ${list_time}s"

    echo "List --blocked:"
    local list_blocked_time
    list_blocked_time=$(time_cmd "$WK_BIN" list --blocked)
    echo "  Time: ${list_blocked_time}s"

    # Complete blocker and measure cascade
    echo "Completing blocker (should unblock $width issues)..."
    "$WK_BIN" start "$blocker" >/dev/null 2>&1 || true

    local done_start
    done_start=$(date +%s.%N)
    "$WK_BIN" done "$blocker" >/dev/null 2>&1 || true
    local done_end
    done_end=$(date +%s.%N)
    local done_time
    done_time=$(echo "$done_end - $done_start" | bc)
    echo "  Complete time: ${done_time}s"

    echo "List after completing blocker:"
    local list_after_time
    list_after_time=$(time_cmd "$WK_BIN" list)
    echo "  Time: ${list_after_time}s"

    # Count ready issues
    local ready_count
    ready_count=$("$WK_BIN" list 2>/dev/null | wc -l)
    echo "  Ready issues: $ready_count"

    section "Results"
    echo "  Width: $width"
    echo "  Create time: ${create_time}s"
    echo "  Dep add time: ${dep_time}s"
    echo "  Show time: ${show_time}s"
    echo "  List time: ${list_time}s"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Star pattern with $width dependents tested"
    else
        print_result "FAIL" "Database corruption detected"
    fi
}
