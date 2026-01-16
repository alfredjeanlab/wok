#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Label Length Stress Test
#
# Finds the maximum label length that can be stored.

stress_label_length() {
    echo "=== Label Length Stress Test ==="

    init_workspace "stress"

    # Create issue to attach labels to
    local issue_output
    issue_output=$("$WK_BIN" new task "Label length test issue" 2>&1)
    local issue_id
    issue_id=$(extract_issue_id "$issue_output")

    local lengths=(50 100 200 500 1000 2000 5000)
    local max_successful=0
    local failed_at=0

    for len in "${lengths[@]}"; do
        echo -n "  Label length $len: "

        # Generate label of exact length (format: prefix:value)
        local prefix_len=$((len / 3))
        local value_len=$((len - prefix_len - 1))

        local prefix
        prefix=$(generate_long_string "$prefix_len" "k")
        local value
        value=$(generate_long_string "$value_len" "v")
        local label="${prefix}:${value}"

        if "$WK_BIN" label "$issue_id" "$label" >/dev/null 2>&1; then
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
    done

    section "Results"
    echo "  Maximum successful: $max_successful characters"
    if [ "$failed_at" -gt 0 ]; then
        echo "  Failed at: $failed_at characters"
    fi

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$max_successful" -ge 50 ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "Label length limit found at ~$max_successful chars"
    else
        print_result "FAIL" "Unexpected failure"
    fi
}
