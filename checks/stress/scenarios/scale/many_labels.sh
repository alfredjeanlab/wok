#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Many Labels Stress Test
#
# Tests scaling behavior with many labels per issue and across issues.

stress_many_labels() {
    local label_count="${1:-100}"

    echo "=== Many Labels Stress Test ==="
    echo "Labels per issue: $label_count"

    init_workspace "stress"

    # Create issue with many labels
    echo "Creating issue with $label_count labels..."

    local issue_output
    issue_output=$("$WK_BIN" new task "Issue with many labels" 2>&1)
    local issue_id
    issue_id=$(extract_issue_id "$issue_output")

    local start_time
    start_time=$(date +%s)

    for i in $(seq 1 "$label_count"); do
        local label="label-$i:value-$i"
        "$WK_BIN" label "$issue_id" "$label" >/dev/null 2>&1 || true

        if [ $((i % 20)) -eq 0 ]; then
            echo "  Added $i labels..."
        fi
    done

    local add_time=$(($(date +%s) - start_time))
    echo "Labels added in ${add_time}s"

    section "Testing Operations"

    echo "Show issue with many labels:"
    local show_time
    show_time=$(time_cmd "$WK_BIN" show "$issue_id")
    echo "  Time: ${show_time}s"

    echo "List with label filter:"
    local filter_time
    filter_time=$(time_cmd "$WK_BIN" list --label "label-1:value-1")
    echo "  Time: ${filter_time}s"

    # Create many issues with same label
    section "Many Issues with Same Label"
    local issue_count=100
    echo "Creating $issue_count issues with shared label..."

    for i in $(seq 1 "$issue_count"); do
        "$WK_BIN" new task "Labeled issue $i" >/dev/null 2>&1
    done

    # Get all issue IDs and label them
    local ids
    ids=$(sqlite3 .wok/issues.db "SELECT id FROM issues WHERE title LIKE 'Labeled issue%'" 2>/dev/null)

    local label_start
    label_start=$(date +%s)
    for id in $ids; do
        "$WK_BIN" label "$id" "shared:common" >/dev/null 2>&1 || true
    done
    local label_time=$(($(date +%s) - label_start))
    echo "Shared label added to $issue_count issues in ${label_time}s"

    echo "List with shared label filter:"
    local shared_filter_time
    shared_filter_time=$(time_cmd "$WK_BIN" list --label "shared:common")
    echo "  Time: ${shared_filter_time}s"

    # Remove labels
    section "Label Removal"
    echo "Removing label from first issue:"
    local unlabel_time
    unlabel_time=$(time_cmd "$WK_BIN" unlabel "$issue_id" "label-1:value-1")
    echo "  Time: ${unlabel_time}s"

    section "Results"
    echo "  Labels per issue: $label_count"
    echo "  Add time: ${add_time}s"
    echo "  Show time: ${show_time}s"
    echo "  Filter time: ${filter_time}s"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Label stress test completed"
    else
        print_result "FAIL" "Database corruption detected"
    fi
}
