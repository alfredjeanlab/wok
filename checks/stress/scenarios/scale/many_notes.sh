#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Many Notes Stress Test
#
# Tests scaling behavior with many notes per issue.

stress_many_notes() {
    local note_count="${1:-100}"

    echo "=== Many Notes Stress Test ==="
    echo "Notes per issue: $note_count"

    init_workspace "stress"

    # Create issue
    local issue_output
    issue_output=$("$WK_BIN" new task "Issue with many notes" 2>&1)
    local issue_id
    issue_id=$(extract_issue_id "$issue_output")

    echo "Created issue: $issue_id"
    echo "Adding $note_count notes..."

    local start_time
    start_time=$(date +%s)

    for i in $(seq 1 "$note_count"); do
        local note_content="Note $i: $(random_string 100)"
        "$WK_BIN" note "$issue_id" "$note_content" >/dev/null 2>&1 || true

        if [ $((i % 20)) -eq 0 ]; then
            echo "  Added $i notes..."
        fi
    done

    local add_time=$(($(date +%s) - start_time))
    echo "Notes added in ${add_time}s"

    section "Testing Operations"

    echo "Show issue with many notes:"
    local show_time
    show_time=$(time_cmd "$WK_BIN" show "$issue_id")
    echo "  Time: ${show_time}s"

    # Check actual note count in database
    local actual_notes
    actual_notes=$(sqlite3 .wok/issues.db "SELECT COUNT(*) FROM notes WHERE issue_id='$issue_id'" 2>/dev/null || echo 0)
    echo "  Notes in database: $actual_notes"

    echo "List all issues:"
    local list_time
    list_time=$(time_cmd "$WK_BIN" list --all)
    echo "  Time: ${list_time}s"

    # Test with large note content
    section "Large Note Content"
    local large_size=10000
    echo "Adding note with ${large_size} characters..."

    local large_note
    large_note=$(generate_pattern_string "$large_size" "Content ")

    local large_start
    large_start=$(date +%s.%N)
    "$WK_BIN" note "$issue_id" "$large_note" >/dev/null 2>&1 || true
    local large_end
    large_end=$(date +%s.%N)
    local large_time
    large_time=$(echo "$large_end - $large_start" | bc)
    echo "  Time: ${large_time}s"

    echo "Show after large note:"
    local show_large_time
    show_large_time=$(time_cmd "$WK_BIN" show "$issue_id")
    echo "  Time: ${show_large_time}s"

    section "Results"
    echo "  Notes per issue: $note_count"
    echo "  Add time: ${add_time}s"
    echo "  Show time: ${show_time}s"
    echo "  Large note time: ${large_time}s"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    local db_size
    db_size=$(get_db_size)
    echo "  Database size: $db_size"

    if [ "$integrity" = "OK" ] && [ "$actual_notes" -ge "$note_count" ]; then
        print_result "PASS" "Note stress test completed"
    else
        print_result "FAIL" "Expected $note_count notes, got $actual_notes (integrity: $integrity)"
    fi
}
