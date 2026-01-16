#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# List All Stress Test
#
# Tests memory and performance of listing all issues in a huge database.
# This is often a worst-case memory scenario.

stress_list_all() {
    local target_issues="${1:-20000}"

    echo "=== List All Stress Test ==="
    echo "Target issues: $target_issues"

    init_workspace "stress"

    # Create many issues
    echo "Creating $target_issues issues..."

    local created=0
    local batch=500

    while [ $created -lt $target_issues ]; do
        for i in $(seq 1 $batch); do
            "$WK_BIN" new task "List test $((created + i))" >/dev/null 2>&1
        done
        created=$((created + batch))

        if [ $((created % 2000)) -eq 0 ]; then
            echo "  Created $created..."

            if ! check_system_health >/dev/null; then
                echo "  Stopping due to resource pressure"
                break
            fi
        fi
    done

    local final_count
    final_count=$(get_issue_count)
    echo "Total issues: $final_count"
    echo "Database size: $(get_db_size)"

    section "List Performance"

    # List all - most memory intensive
    echo "List all issues..."
    local list_all_start
    list_all_start=$(date +%s.%N)

    local list_count
    list_count=$("$WK_BIN" list --all 2>/dev/null | wc -l)

    local list_all_end
    list_all_end=$(date +%s.%N)
    local list_all_time
    list_all_time=$(echo "$list_all_end - $list_all_start" | bc)

    echo "  Time: ${list_all_time}s"
    echo "  Lines output: $list_count"

    # List ready only
    echo "List ready issues only..."
    local list_ready_start
    list_ready_start=$(date +%s.%N)

    local ready_count
    ready_count=$("$WK_BIN" list 2>/dev/null | wc -l)

    local list_ready_end
    list_ready_end=$(date +%s.%N)
    local list_ready_time
    list_ready_time=$(echo "$list_ready_end - $list_ready_start" | bc)

    echo "  Time: ${list_ready_time}s"
    echo "  Ready issues: $ready_count"

    # List with various filters
    section "Filtered Lists"

    # By status
    echo "List by status (in_progress)..."
    local status_time
    status_time=$(time_cmd "$WK_BIN" list --status in_progress)
    echo "  Time: ${status_time}s"

    # With tag (should be fast if indexed)
    echo "List with tag filter..."
    # First add a tag to some issues
    local sample_id
    sample_id=$(get_random_issue_id)
    if [ -n "$sample_id" ]; then
        "$WK_BIN" tag "$sample_id" "filter:test" >/dev/null 2>&1 || true

        local tag_time
        tag_time=$(time_cmd "$WK_BIN" list --tag "filter:test")
        echo "  Time: ${tag_time}s"
    fi

    section "Repeated Lists"

    echo "Running 10 sequential list --all..."
    local total_list_time=0
    local list_times=()

    for i in $(seq 1 10); do
        local start
        start=$(date +%s.%N)
        "$WK_BIN" list --all >/dev/null 2>&1
        local end
        end=$(date +%s.%N)
        local elapsed
        elapsed=$(echo "$end - $start" | bc)
        total_list_time=$(echo "$total_list_time + $elapsed" | bc)
        list_times+=("$elapsed")
    done

    local avg_list
    avg_list=$(echo "scale=3; $total_list_time / 10" | bc)
    echo "  Average: ${avg_list}s"

    # Check for consistency (times should be similar)
    echo "  Individual times: ${list_times[*]}"

    section "Concurrent Lists"

    echo "Running 5 concurrent list --all..."
    local concurrent_start
    concurrent_start=$(date +%s.%N)

    for _ in $(seq 1 5); do
        "$WK_BIN" list --all >/dev/null 2>&1 &
    done
    wait

    local concurrent_end
    concurrent_end=$(date +%s.%N)
    local concurrent_time
    concurrent_time=$(echo "$concurrent_end - $concurrent_start" | bc)

    echo "  Total time: ${concurrent_time}s"
    echo "  Speedup vs serial: $(echo "scale=2; ($avg_list * 5) / $concurrent_time" | bc)x"

    section "Results"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    local issues_per_sec
    issues_per_sec=$(echo "scale=0; $final_count / $list_all_time" | bc 2>/dev/null || echo "N/A")
    echo "  List throughput: $issues_per_sec issues/sec"

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Listed $final_count issues in ${list_all_time}s"
    else
        print_result "FAIL" "Issues during list test"
    fi
}
