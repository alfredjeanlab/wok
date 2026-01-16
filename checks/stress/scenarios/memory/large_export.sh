#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Large Export Stress Test
#
# Tests export functionality with large databases.
# Monitors memory and disk usage during export.

stress_large_export() {
    local target_issues="${1:-10000}"

    echo "=== Large Export Stress Test ==="
    echo "Target issues: $target_issues"

    init_workspace "stress"

    # Create many issues with varying content
    echo "Creating $target_issues issues with content..."

    local created=0
    local batch=500

    while [ $created -lt $target_issues ]; do
        for i in $(seq 1 $batch); do
            local num=$((created + i))
            # Add some variation: tags and notes
            local output
            output=$("$WK_BIN" new task "Export test issue $num with some longer title text" 2>&1)
            local id
            id=$(extract_issue_id "$output")

            if [ -n "$id" ] && [ $((num % 5)) -eq 0 ]; then
                "$WK_BIN" tag "$id" "batch:$((num / 100))" >/dev/null 2>&1 || true
            fi
            if [ -n "$id" ] && [ $((num % 10)) -eq 0 ]; then
                "$WK_BIN" note "$id" "Note for issue $num with some content" >/dev/null 2>&1 || true
            fi
        done
        created=$((created + batch))
        echo "  Created $created..."

        if ! check_system_health >/dev/null; then
            echo "  Stopping due to resource pressure"
            break
        fi
    done

    local final_count
    final_count=$(get_issue_count)
    echo "Total issues: $final_count"
    echo "Database size: $(get_db_size)"

    section "Export Tests"

    # JSONL export
    echo "Exporting to JSONL..."
    local jsonl_start
    jsonl_start=$(date +%s.%N)

    "$WK_BIN" export /tmp/large_export.jsonl >/dev/null 2>&1
    local jsonl_status=$?

    local jsonl_end
    jsonl_end=$(date +%s.%N)
    local jsonl_time
    jsonl_time=$(echo "$jsonl_end - $jsonl_start" | bc)

    if [ $jsonl_status -eq 0 ] && [ -f /tmp/large_export.jsonl ]; then
        local jsonl_size
        jsonl_size=$(du -h /tmp/large_export.jsonl | cut -f1)
        local jsonl_lines
        jsonl_lines=$(wc -l < /tmp/large_export.jsonl)
        echo "  Status: OK"
        echo "  Time: ${jsonl_time}s"
        echo "  Size: $jsonl_size"
        echo "  Lines: $jsonl_lines"

        # Verify export content
        if [ "$jsonl_lines" -ge "$final_count" ]; then
            echo "  Verification: Line count matches issue count"
        else
            echo "  Verification: WARNING - fewer lines than issues"
        fi
    else
        echo "  Status: FAILED"
    fi

    # Multiple sequential exports (check for leaks)
    section "Repeated Exports"

    echo "Running 5 sequential exports..."
    local total_export_time=0
    for i in $(seq 1 5); do
        local start
        start=$(date +%s.%N)
        "$WK_BIN" export "/tmp/export_$i.jsonl" >/dev/null 2>&1
        local end
        end=$(date +%s.%N)
        local elapsed
        elapsed=$(echo "$end - $start" | bc)
        total_export_time=$(echo "$total_export_time + $elapsed" | bc)
        echo "  Export $i: ${elapsed}s"
    done

    local avg_time
    avg_time=$(echo "scale=2; $total_export_time / 5" | bc)
    echo "  Average: ${avg_time}s"

    # Clean up export files
    rm -f /tmp/large_export.jsonl /tmp/export_*.jsonl

    section "Import Test"

    # Create a fresh workspace and import
    echo "Testing import of exported data..."

    # Export current database
    "$WK_BIN" export /tmp/reimport_test.jsonl >/dev/null 2>&1

    # Create new workspace
    rm -rf .wok
    "$WK_BIN" init --prefix reimport >/dev/null 2>&1

    local import_start
    import_start=$(date +%s.%N)
    "$WK_BIN" import /tmp/reimport_test.jsonl >/dev/null 2>&1
    local import_status=$?
    local import_end
    import_end=$(date +%s.%N)
    local import_time
    import_time=$(echo "$import_end - $import_start" | bc)

    if [ $import_status -eq 0 ]; then
        local reimport_count
        reimport_count=$(get_issue_count)
        echo "  Import status: OK"
        echo "  Import time: ${import_time}s"
        echo "  Imported issues: $reimport_count"
    else
        echo "  Import status: FAILED"
    fi

    rm -f /tmp/reimport_test.jsonl

    section "Results"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Export/import of $final_count issues"
    else
        print_result "FAIL" "Issues with export/import"
    fi
}
