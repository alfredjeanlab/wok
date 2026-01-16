#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Note Length Stress Test
#
# Finds the maximum note content length that can be stored and displayed.

stress_note_length() {
    echo "=== Note Length Stress Test ==="

    init_workspace "stress"

    # Create issue to attach notes to
    local issue_output
    issue_output=$("$WK_BIN" new task "Note length test issue" 2>&1)
    local issue_id
    issue_id=$(extract_issue_id "$issue_output")

    local lengths=(100 1000 5000 10000 50000 100000 500000 1000000)
    local max_successful=0
    local failed_at=0

    for len in "${lengths[@]}"; do
        echo -n "  Length $len: "

        # Generate note content of exact length
        local content
        content=$(generate_pattern_string "$len" "Note content ")

        if "$WK_BIN" note "$issue_id" "$content" >/dev/null 2>&1; then
            # Verify it can be shown
            if "$WK_BIN" show "$issue_id" >/dev/null 2>&1; then
                echo "OK"
                max_successful=$len
            else
                echo "ADDED but SHOW FAILED"
                failed_at=$len
                break
            fi
        else
            echo "FAILED (limit found)"
            failed_at=$len
            break
        fi

        # Health check for large sizes
        if [ "$len" -ge 50000 ]; then
            if ! check_system_health >/dev/null; then
                echo "  Stopping due to resource pressure"
                break
            fi
        fi
    done

    section "Results"
    echo "  Maximum successful: $max_successful characters"
    if [ "$failed_at" -gt 0 ]; then
        echo "  Failed at: $failed_at characters"
    fi

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    local db_size
    db_size=$(get_db_size)
    echo "  Database size: $db_size"

    if [ "$max_successful" -ge 100 ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "Note length limit found at ~$max_successful chars"
    else
        print_result "FAIL" "Unexpected failure"
    fi
}
