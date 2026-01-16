#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Massive Database Stress Test
#
# Creates a large number of issues to test database scalability.
# Tests list performance, show performance, and export at scale.

stress_massive_db() {
    local target_count="${1:-100000}"

    echo "=== Massive Database Stress Test ==="
    echo "Target: $target_count issues"

    init_workspace "stress"

    local start_time
    start_time=$(date +%s)
    local batch_size=100
    local created=0
    local last_report=0

    echo "Creating issues..."
    while [ $created -lt $target_count ]; do
        # Check system health periodically
        if [ $((created % 1000)) -eq 0 ] && [ $created -gt 0 ]; then
            if ! check_system_health; then
                echo "Stopping at $created due to resource pressure"
                break
            fi
        fi

        # Create batch of issues in parallel
        local batch_end=$((created + batch_size))
        [ $batch_end -gt $target_count ] && batch_end=$target_count

        for i in $(seq $((created + 1)) $batch_end); do
            "$WK_BIN" new task "Issue $i" >/dev/null 2>&1 &
        done
        wait

        created=$batch_end

        # Progress report every 1000 issues
        if [ $((created - last_report)) -ge 1000 ]; then
            local elapsed=$(($(date +%s) - start_time))
            local rate=$((created / (elapsed + 1)))
            echo "  Created: $created ($rate issues/sec)"
            last_report=$created
        fi
    done

    local total_time=$(($(date +%s) - start_time))
    local final_count
    final_count=$(get_issue_count)

    echo ""
    echo "Creation complete:"
    echo "  Issues created: $final_count"
    echo "  Time: ${total_time}s"
    echo "  Rate: $((final_count / (total_time + 1))) issues/sec"
    echo "  Database size: $(get_db_size)"

    # Performance tests
    section "Performance Tests"

    echo "List all issues:"
    local list_time
    list_time=$(time_cmd "$WK_BIN" list --all)
    echo "  Time: ${list_time}s"

    echo "List default (ready):"
    local list_ready_time
    list_ready_time=$(time_cmd "$WK_BIN" list)
    echo "  Time: ${list_ready_time}s"

    echo "Show random issue:"
    local random_id
    random_id=$(get_random_issue_id)
    if [ -n "$random_id" ]; then
        local show_time
        show_time=$(time_cmd "$WK_BIN" show "$random_id")
        echo "  Time: ${show_time}s"
    fi

    echo "Export to JSONL:"
    local export_time
    export_time=$(time_cmd "$WK_BIN" export /tmp/stress_export.jsonl)
    echo "  Time: ${export_time}s"
    if [ -f /tmp/stress_export.jsonl ]; then
        echo "  Export size: $(du -h /tmp/stress_export.jsonl | cut -f1)"
        rm -f /tmp/stress_export.jsonl
    fi

    # Integrity check
    section "Integrity Check"
    local integrity
    integrity=$(check_db_integrity)
    echo "  Database: $integrity"

    if [ "$final_count" -eq "$target_count" ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "$final_count issues created and verified"
    else
        print_result "FAIL" "Expected $target_count, got $final_count (integrity: $integrity)"
    fi
}
