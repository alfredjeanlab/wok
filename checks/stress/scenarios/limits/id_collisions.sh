#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# ID Collision Stress Test
#
# Creates many issues with identical titles to test ID collision handling.
# Verifies all IDs are unique.

stress_id_collisions() {
    echo "=== ID Collision Stress Test ==="

    init_workspace "stress"

    local base_title="Test Issue"
    local count=1000

    echo "Creating $count issues with identical titles..."

    local ids=()
    local start_time
    start_time=$(date +%s)

    for i in $(seq 1 "$count"); do
        local output
        output=$("$WK_BIN" new task "$base_title" 2>&1)
        local id
        id=$(extract_issue_id "$output")
        ids+=("$id")

        if [ $((i % 100)) -eq 0 ]; then
            echo "  Created $i..."
        fi
    done

    local create_time=$(($(date +%s) - start_time))
    echo "Created in ${create_time}s"

    section "Uniqueness Check"

    # Check for uniqueness
    local unique_count
    unique_count=$(printf '%s\n' "${ids[@]}" | sort -u | wc -l | tr -d ' ')

    echo "  Total IDs: ${#ids[@]}"
    echo "  Unique IDs: $unique_count"

    if [ "$unique_count" -eq "${#ids[@]}" ]; then
        echo "  Status: All IDs unique"
    else
        echo "  Status: DUPLICATE IDs DETECTED"
        echo ""
        echo "  Duplicate analysis:"
        printf '%s\n' "${ids[@]}" | sort | uniq -c | sort -rn | head -10
    fi

    section "ID Pattern Analysis"

    # Show ID distribution pattern
    echo "  Sample IDs:"
    printf '%s\n' "${ids[@]:0:5}"
    echo "  ..."
    printf '%s\n' "${ids[@]: -3}"

    # Check if IDs have incrementing suffixes
    local with_suffix
    with_suffix=$(printf '%s\n' "${ids[@]}" | grep -c '-[0-9]\+$' || echo 0)
    echo "  IDs with numeric suffix: $with_suffix"

    section "Results"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    local db_count
    db_count=$(get_issue_count)
    echo "  Issues in database: $db_count"

    if [ "$unique_count" -eq "${#ids[@]}" ] && [ "$db_count" -eq "$count" ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "All $count IDs unique"
    else
        print_result "FAIL" "Duplicates or missing issues detected"
    fi
}
