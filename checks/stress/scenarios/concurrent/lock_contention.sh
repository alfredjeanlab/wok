#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Lock Contention Stress Test
#
# Forces database lock contention to test handling of
# SQLITE_BUSY and lock timeouts.

stress_lock_contention() {
    local contenders="${1:-20}"
    local operations="${2:-50}"

    echo "=== Lock Contention Stress Test ==="
    echo "Contenders: $contenders"
    echo "Operations each: $operations"

    init_workspace "stress"

    # Create a single issue that all processes will fight over
    local output
    output=$("$WK_BIN" new task "Contention target" 2>&1)
    local target_id
    target_id=$(extract_issue_id "$output")
    echo "Target issue: $target_id"

    local start_time
    start_time=$(date +%s.%N)

    # Create temp dir for results
    local results_dir
    results_dir=$(mktemp -d)

    # Launch contending processes
    for c in $(seq 1 "$contenders"); do
        (
            local success=0
            local busy=0
            local other_err=0

            for _ in $(seq 1 "$operations"); do
                # Try various operations on the same issue
                local op=$((RANDOM % 4))
                local result

                case $op in
                    0) result=$("$WK_BIN" show "$target_id" 2>&1) ;;
                    1) result=$("$WK_BIN" tag "$target_id" "contender:$c" 2>&1) ;;
                    2) result=$("$WK_BIN" note "$target_id" "Note from $c at $(date +%s%N)" 2>&1) ;;
                    3) result=$("$WK_BIN" untag "$target_id" "contender:$c" 2>&1) ;;
                esac

                if [ $? -eq 0 ]; then
                    ((success++))
                elif echo "$result" | grep -qi "busy\|locked\|timeout"; then
                    ((busy++))
                else
                    ((other_err++))
                fi
            done

            echo "$success $busy $other_err" > "$results_dir/contender_$c"
        ) &
    done

    echo "Waiting for contenders..."
    wait

    local end_time
    end_time=$(date +%s.%N)
    local elapsed
    elapsed=$(echo "$end_time - $start_time" | bc)

    # Aggregate results
    local total_success=0
    local total_busy=0
    local total_other=0

    for f in "$results_dir"/contender_*; do
        if [ -f "$f" ]; then
            read -r success busy other < "$f"
            total_success=$((total_success + success))
            total_busy=$((total_busy + busy))
            total_other=$((total_other + other))
        fi
    done
    rm -rf "$results_dir"

    section "Results"

    local total_ops=$((contenders * operations))
    echo "  Total operations: $total_ops"
    echo "  Successful: $total_success"
    echo "  Lock busy/timeout: $total_busy"
    echo "  Other errors: $total_other"
    echo "  Time: ${elapsed}s"

    if [ "$total_success" -gt 0 ]; then
        local rate
        rate=$(echo "scale=2; $total_success / $elapsed" | bc)
        echo "  Success rate: $rate ops/sec"
    fi

    local success_percent=$((total_success * 100 / total_ops))
    echo "  Success rate: ${success_percent}%"

    # Check database integrity after contention
    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    # Verify target issue still accessible
    if "$WK_BIN" show "$target_id" >/dev/null 2>&1; then
        echo "  Target issue: accessible"
    else
        echo "  Target issue: NOT ACCESSIBLE"
        integrity="FAIL"
    fi

    if [ "$integrity" = "OK" ] && [ "$success_percent" -gt 50 ]; then
        print_result "PASS" "${success_percent}% success under contention"
    else
        print_result "FAIL" "Integrity: $integrity, success: ${success_percent}%"
    fi
}
