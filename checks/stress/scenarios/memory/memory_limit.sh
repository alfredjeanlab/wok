#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Memory Limit Stress Test
#
# Tests behavior under memory constraints.
# Monitors memory usage during operations.

stress_memory_limit() {
    local target_issues="${1:-5000}"

    echo "=== Memory Limit Stress Test ==="
    echo "Target issues: $target_issues"

    init_workspace "stress"

    # Check available monitoring tools
    local has_time=0
    if /usr/bin/time --version 2>&1 | grep -q "GNU\|BSD" || /usr/bin/time -l true 2>/dev/null; then
        has_time=1
        echo "Memory monitoring: /usr/bin/time available"
    else
        echo "Memory monitoring: basic (no /usr/bin/time)"
    fi

    section "Issue Creation Memory"

    echo "Creating $target_issues issues and monitoring memory..."

    local start_time
    start_time=$(date +%s)
    local created=0
    local batch=500

    while [ $created -lt $target_issues ]; do
        # Create batch
        for i in $(seq 1 $batch); do
            "$WK_BIN" new task "Memory test $((created + i))" >/dev/null 2>&1
        done
        created=$((created + batch))

        # Check memory periodically
        if [ $((created % 1000)) -eq 0 ]; then
            echo "  Created $created issues..."

            if ! check_system_health >/dev/null; then
                echo "  Stopping due to resource pressure"
                break
            fi
        fi
    done

    local create_time=$(($(date +%s) - start_time))
    local final_count
    final_count=$(get_issue_count)

    echo "Created $final_count issues in ${create_time}s"

    section "List Memory Usage"

    if [ "$has_time" -eq 1 ]; then
        echo "Measuring memory for 'list --all'..."

        local list_mem
        if [[ "$OSTYPE" == "darwin"* ]]; then
            list_mem=$({ /usr/bin/time -l "$WK_BIN" list --all >/dev/null; } 2>&1 | grep "maximum resident" | awk '{print $1}')
            list_mem=$((list_mem / 1024 / 1024))  # Convert to MB
        else
            list_mem=$({ /usr/bin/time -v "$WK_BIN" list --all >/dev/null; } 2>&1 | grep "Maximum resident" | awk '{print $6}')
            list_mem=$((list_mem / 1024))  # Convert to MB
        fi

        echo "  Peak memory: ${list_mem}MB"
        echo "  Memory per issue: $((list_mem * 1024 / final_count)) KB"
    else
        echo "Timing 'list --all'..."
        local list_time
        list_time=$(time_cmd "$WK_BIN" list --all)
        echo "  Time: ${list_time}s"
    fi

    section "Show Memory Usage"

    local show_id
    show_id=$(get_random_issue_id)

    if [ -n "$show_id" ] && [ "$has_time" -eq 1 ]; then
        echo "Measuring memory for 'show'..."

        local show_mem
        if [[ "$OSTYPE" == "darwin"* ]]; then
            show_mem=$({ /usr/bin/time -l "$WK_BIN" show "$show_id" >/dev/null; } 2>&1 | grep "maximum resident" | awk '{print $1}')
            show_mem=$((show_mem / 1024 / 1024))
        else
            show_mem=$({ /usr/bin/time -v "$WK_BIN" show "$show_id" >/dev/null; } 2>&1 | grep "Maximum resident" | awk '{print $6}')
            show_mem=$((show_mem / 1024))
        fi

        echo "  Peak memory: ${show_mem}MB"
    fi

    section "Export Memory Usage"

    if [ "$has_time" -eq 1 ]; then
        echo "Measuring memory for 'export'..."

        local export_mem
        if [[ "$OSTYPE" == "darwin"* ]]; then
            export_mem=$({ /usr/bin/time -l "$WK_BIN" export /tmp/mem_test.jsonl >/dev/null; } 2>&1 | grep "maximum resident" | awk '{print $1}')
            export_mem=$((export_mem / 1024 / 1024))
        else
            export_mem=$({ /usr/bin/time -v "$WK_BIN" export /tmp/mem_test.jsonl >/dev/null; } 2>&1 | grep "Maximum resident" | awk '{print $6}')
            export_mem=$((export_mem / 1024))
        fi

        echo "  Peak memory: ${export_mem}MB"

        if [ -f /tmp/mem_test.jsonl ]; then
            echo "  Export size: $(du -h /tmp/mem_test.jsonl | cut -f1)"
            rm -f /tmp/mem_test.jsonl
        fi
    fi

    section "Results"

    echo "  Issues: $final_count"
    echo "  Database size: $(get_db_size)"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$integrity" = "OK" ] && [ "$final_count" -gt 0 ]; then
        print_result "PASS" "Memory test completed with $final_count issues"
    else
        print_result "FAIL" "Issues during memory test"
    fi
}
