#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Path Depth Stress Test
#
# Tests behavior with deeply nested .wok directory paths.
# Some filesystems have path length limits.

stress_path_depth() {
    echo "=== Path Depth Stress Test ==="

    # Create nested directory structure
    local max_depth=50
    local current_path="$STRESS_SANDBOX"

    echo "Creating nested directories (target depth: $max_depth)..."

    local achieved_depth=0
    for i in $(seq 1 "$max_depth"); do
        local dirname="level_$i"
        local new_path="$current_path/$dirname"

        if mkdir -p "$new_path" 2>/dev/null; then
            current_path="$new_path"
            achieved_depth=$i
        else
            echo "  Failed to create at depth $i"
            break
        fi

        if [ $((i % 10)) -eq 0 ]; then
            echo "  Created depth $i..."
        fi
    done

    echo "Achieved depth: $achieved_depth"
    echo "Path length: ${#current_path} characters"

    section "Testing wk at Depth"

    cd "$current_path" || {
        print_result "FAIL" "Could not cd to deep path"
        return 1
    }

    echo "Initializing workspace at depth $achieved_depth..."
    if "$WK_BIN" init --prefix deep >/dev/null 2>&1; then
        echo "  Init: OK"
    else
        echo "  Init: FAILED"
        print_result "FAIL" "Could not init at depth $achieved_depth"
        return 1
    fi

    echo "Creating issue..."
    local output
    if output=$("$WK_BIN" new task "Deep issue" 2>&1); then
        local id
        id=$(extract_issue_id "$output")
        echo "  Create: OK ($id)"
    else
        echo "  Create: FAILED"
        print_result "FAIL" "Could not create issue at depth $achieved_depth"
        return 1
    fi

    echo "Listing issues..."
    if "$WK_BIN" list >/dev/null 2>&1; then
        echo "  List: OK"
    else
        echo "  List: FAILED"
    fi

    echo "Showing issue..."
    if "$WK_BIN" show "$id" >/dev/null 2>&1; then
        echo "  Show: OK"
    else
        echo "  Show: FAILED"
    fi

    section "Results"
    echo "  Maximum depth achieved: $achieved_depth"
    echo "  Path length: ${#current_path} characters"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    if [ "$achieved_depth" -ge 10 ] && [ "$integrity" = "OK" ]; then
        print_result "PASS" "Operations work at depth $achieved_depth"
    else
        print_result "FAIL" "Failed at depth $achieved_depth"
    fi

    # Return to sandbox root
    cd "$STRESS_SANDBOX" || true
}
