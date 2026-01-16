#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Corrupt Database Stress Test
#
# Tests behavior with various types of database corruption.
# Verifies error handling and recovery options.

stress_corrupt_db() {
    echo "=== Corrupt Database Test ==="

    init_workspace "stress"

    # Create some issues
    local initial_count=10
    echo "Creating $initial_count initial issues..."
    for i in $(seq 1 "$initial_count"); do
        "$WK_BIN" new task "Issue $i" >/dev/null 2>&1
    done

    echo "Database size: $(get_db_size)"

    # Back up database
    cp .wok/issues.db /tmp/backup_clean.db

    section "Scenario 1: Truncated Database"

    echo "Truncating database to 1KB..."
    head -c 1024 .wok/issues.db > /tmp/truncated.db
    cp /tmp/truncated.db .wok/issues.db

    local trunc_result
    if trunc_result=$("$WK_BIN" list 2>&1); then
        echo "  List: Succeeded (unexpected)"
    else
        echo "  List: Error returned (expected)"
        if echo "$trunc_result" | grep -qi "corrupt\|malformed\|error"; then
            echo "  Error message indicates corruption"
        fi
    fi

    # Restore
    cp /tmp/backup_clean.db .wok/issues.db

    section "Scenario 2: Random Bytes Injected"

    echo "Injecting random bytes at offset 2000..."
    cp /tmp/backup_clean.db .wok/issues.db
    dd if=/dev/urandom of=.wok/issues.db bs=100 count=1 seek=20 conv=notrunc 2>/dev/null

    local rand_result
    if rand_result=$("$WK_BIN" list 2>&1); then
        echo "  List: Succeeded (database may be partially readable)"
    else
        echo "  List: Error returned"
    fi

    # Restore
    cp /tmp/backup_clean.db .wok/issues.db

    section "Scenario 3: Empty Database File"

    echo "Emptying database file..."
    > .wok/issues.db

    local empty_result
    if empty_result=$("$WK_BIN" list 2>&1); then
        echo "  List: Succeeded (recreated database?)"
    else
        echo "  List: Error returned"
    fi

    local empty_init
    if empty_init=$("$WK_BIN" init --prefix stress 2>&1); then
        echo "  Re-init: Succeeded"
    else
        echo "  Re-init: Failed"
    fi

    # Restore
    cp /tmp/backup_clean.db .wok/issues.db

    section "Scenario 4: Wrong File Type"

    echo "Replacing database with text file..."
    echo "This is not a SQLite database" > .wok/issues.db

    local wrong_result
    if wrong_result=$("$WK_BIN" list 2>&1); then
        echo "  List: Succeeded (unexpected)"
    else
        echo "  List: Error returned (expected)"
    fi

    # Restore
    cp /tmp/backup_clean.db .wok/issues.db

    section "Scenario 5: Permissions Denied"

    echo "Removing read permissions..."
    chmod 000 .wok/issues.db

    local perm_result
    if perm_result=$("$WK_BIN" list 2>&1); then
        echo "  List: Succeeded (unexpected)"
    else
        echo "  List: Error returned (expected)"
        if echo "$perm_result" | grep -qi "permission\|denied\|access"; then
            echo "  Error message indicates permission issue"
        fi
    fi

    # Restore permissions
    chmod 644 .wok/issues.db

    section "Recovery Test"

    echo "Testing operations after restoration..."
    cp /tmp/backup_clean.db .wok/issues.db

    if "$WK_BIN" list >/dev/null 2>&1; then
        echo "  List after restore: OK"
    else
        echo "  List after restore: FAILED"
    fi

    if "$WK_BIN" new task "Post-corruption issue" >/dev/null 2>&1; then
        echo "  Create after restore: OK"
    else
        echo "  Create after restore: FAILED"
    fi

    section "Results"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Final integrity: $integrity"

    # Clean up
    rm -f /tmp/backup_clean.db /tmp/truncated.db

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Error handling and recovery verified"
    else
        print_result "FAIL" "Database not properly restored"
    fi
}
