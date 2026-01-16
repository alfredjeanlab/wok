#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Title Length Stress Test
#
# Finds the maximum title length that can be stored and displayed.

stress_title_length() {
    echo "=== Title Length Stress Test ==="

    init_workspace "stress"

    local lengths=(100 500 1000 5000 10000 50000 100000)
    local max_successful=0
    local failed_at=0

    for len in "${lengths[@]}"; do
        echo -n "  Length $len: "

        # Generate title of exact length
        local title
        title=$(generate_long_string "$len" "X")

        local output
        if output=$("$WK_BIN" new task "$title" 2>&1); then
            local id
            id=$(extract_issue_id "$output")

            # Verify it can be shown
            if "$WK_BIN" show "$id" >/dev/null 2>&1; then
                echo "OK"
                max_successful=$len
            else
                echo "CREATED but SHOW FAILED"
                failed_at=$len
                break
            fi
        else
            echo "FAILED (limit found)"
            failed_at=$len
            break
        fi

        # Health check
        if ! check_system_health >/dev/null; then
            echo "  Stopping due to resource pressure"
            break
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

    if [ "$max_successful" -ge 100 ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "Title length limit found at ~$max_successful chars"
    else
        print_result "FAIL" "Unexpected failure"
    fi
}
