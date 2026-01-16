#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Disk Full Stress Test
#
# Tests behavior when disk space runs out during operations.
# WARNING: This test is potentially dangerous and is skipped by default.
# Set STRESS_SKIP_DANGEROUS=0 to enable.

stress_disk_full() {
    echo "=== Disk Full Stress Test ==="

    if [ "${STRESS_SKIP_DANGEROUS:-1}" -eq 1 ]; then
        echo "SKIPPED: Set STRESS_SKIP_DANGEROUS=0 to enable this test"
        print_result "SKIP" "Dangerous test skipped"
        return 0
    fi

    # Only run in container where we have tmpfs limits
    if ! is_containerized; then
        echo "WARNING: Running disk_full outside container is risky"
        echo "This test will create large files to fill disk space"
        echo ""
    fi

    init_workspace "stress"

    # Create some initial issues
    local initial_count=10
    echo "Creating $initial_count initial issues..."
    for i in $(seq 1 "$initial_count"); do
        "$WK_BIN" new task "Issue $i" >/dev/null 2>&1
    done

    local pre_count
    pre_count=$(get_issue_count)
    echo "Initial issues: $pre_count"

    section "Filling Disk Space"

    # Get available space
    local avail_mb
    avail_mb=$(df -m . | awk 'NR==2 {print $4}')
    echo "Available space: ${avail_mb}MB"

    # Create large files to fill disk (leave small buffer)
    local fill_mb=$((avail_mb - 5))
    if [ "$fill_mb" -gt 0 ]; then
        echo "Creating ${fill_mb}MB of filler files..."

        local filled=0
        local block=50  # MB per file
        while [ $filled -lt $fill_mb ]; do
            local size=$block
            [ $((filled + size)) -gt $fill_mb ] && size=$((fill_mb - filled))

            dd if=/dev/zero of="filler_$filled.tmp" bs=1M count="$size" 2>/dev/null || break
            filled=$((filled + size))
            echo "  Filled ${filled}MB..."
        done
    fi

    # Check remaining space
    local remaining
    remaining=$(df -m . | awk 'NR==2 {print $4}')
    echo "Remaining space: ${remaining}MB"

    section "Operations Under Disk Pressure"

    # Try to create issues
    echo "Attempting to create issues..."
    local created=0
    local failed=0

    for i in $(seq 1 20); do
        if "$WK_BIN" new task "Disk full test $i" >/dev/null 2>&1; then
            ((created++))
        else
            ((failed++))
        fi
    done

    echo "  Created: $created"
    echo "  Failed: $failed"

    # Try other operations
    echo "Testing other operations..."

    local list_ok=0
    if "$WK_BIN" list >/dev/null 2>&1; then
        echo "  List: OK"
        list_ok=1
    else
        echo "  List: FAILED"
    fi

    local show_ok=0
    local show_id
    show_id=$(get_random_issue_id)
    if [ -n "$show_id" ] && "$WK_BIN" show "$show_id" >/dev/null 2>&1; then
        echo "  Show: OK"
        show_ok=1
    else
        echo "  Show: FAILED"
    fi

    section "Recovery"

    # Remove filler files
    echo "Removing filler files..."
    rm -f filler_*.tmp

    local recovered_space
    recovered_space=$(df -m . | awk 'NR==2 {print $4}')
    echo "Recovered space: ${recovered_space}MB"

    # Try operations after recovery
    echo "Testing operations after space recovery..."

    if "$WK_BIN" new task "Post-recovery issue" >/dev/null 2>&1; then
        echo "  Create: OK"
    else
        echo "  Create: FAILED"
    fi

    section "Results"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    local final_count
    final_count=$(get_issue_count)
    echo "  Final issue count: $final_count"

    if [ "$integrity" = "OK" ] && [ "$list_ok" -eq 1 ]; then
        print_result "PASS" "Handled disk full gracefully"
    else
        print_result "FAIL" "Problems after disk full"
    fi
}
