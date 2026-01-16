#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Interrupted Write Stress Test
#
# Tests behavior when write operations are killed mid-execution.
# Verifies database consistency after crash.

stress_interrupted_write() {
    echo "=== Interrupted Write Test ==="

    init_workspace "stress"

    # Create some initial issues
    local initial_count=10
    echo "Creating $initial_count initial issues..."
    for i in $(seq 1 "$initial_count"); do
        "$WK_BIN" new task "Issue $i" >/dev/null 2>&1
    done

    local pre_interrupt_count
    pre_interrupt_count=$(get_issue_count)
    echo "Initial issues: $pre_interrupt_count"

    # Back up database
    cp .wok/issues.db /tmp/backup_before_interrupt.db

    section "Interrupt During Creates"

    # Start a write operation and kill it mid-way
    echo "Starting write process (will be killed)..."
    (
        for i in $(seq 1 100); do
            "$WK_BIN" new task "Interrupted $i" >/dev/null 2>&1
            sleep 0.02
        done
    ) &
    local pid=$!

    # Let it run briefly then kill it
    sleep 0.3
    kill -9 "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true

    echo "Process killed (pid: $pid)"

    # Check database state
    section "Post-Interrupt State"

    local post_count
    post_count=$(get_issue_count)
    echo "  Issues after interrupt: $post_count"
    echo "  Issues created before kill: $((post_count - pre_interrupt_count))"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Integrity check: $integrity"

    # Try to continue using the database
    section "Post-Crash Operations"

    if "$WK_BIN" list >/dev/null 2>&1; then
        echo "  List: OK"
    else
        echo "  List: FAILED"
    fi

    local new_output
    if new_output=$("$WK_BIN" new task "Post-crash issue" 2>&1); then
        local new_id
        new_id=$(extract_issue_id "$new_output")
        echo "  Create new issue: OK ($new_id)"
    else
        echo "  Create new issue: FAILED"
    fi

    local final_count
    final_count=$(get_issue_count)
    echo "  Final issue count: $final_count"

    section "Interrupt During Update"

    # Get an issue to update
    local update_id
    update_id=$(get_random_issue_id)

    if [ -n "$update_id" ]; then
        echo "Testing interrupt during status changes..."

        # Rapid status changes with interruption
        (
            for _ in $(seq 1 50); do
                "$WK_BIN" start "$update_id" >/dev/null 2>&1
                "$WK_BIN" stop "$update_id" >/dev/null 2>&1
            done
        ) &
        pid=$!

        sleep 0.2
        kill -9 "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true

        # Verify issue is still accessible
        if "$WK_BIN" show "$update_id" >/dev/null 2>&1; then
            echo "  Issue still accessible: OK"
        else
            echo "  Issue still accessible: FAILED"
        fi
    fi

    section "Results"

    integrity=$(check_db_integrity)
    echo "  Final integrity: $integrity"

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Database consistent after interruption"
    else
        print_result "FAIL" "Database corrupted after interruption"
    fi

    # Clean up
    rm -f /tmp/backup_before_interrupt.db
}
