#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Corrupt Config Stress Test
#
# Tests behavior with corrupted or malformed config.toml files.

stress_corrupt_config() {
    echo "=== Corrupt Config Test ==="

    init_workspace "stress"

    # Create some issues first
    for i in $(seq 1 5); do
        "$WK_BIN" new task "Issue $i" >/dev/null 2>&1
    done

    # Check if config exists
    local config_file=".wok/config.toml"
    if [ ! -f "$config_file" ]; then
        echo "No config.toml found - creating one"
        # Try to trigger config creation via some command
        "$WK_BIN" list >/dev/null 2>&1
    fi

    if [ -f "$config_file" ]; then
        # Back up config
        cp "$config_file" /tmp/backup_config.toml
        echo "Original config backed up"
    else
        echo "No config file to test - testing without config"
        config_file=""
    fi

    section "Scenario 1: Empty Config"

    if [ -n "$config_file" ]; then
        echo "Emptying config file..."
        > "$config_file"

        if "$WK_BIN" list >/dev/null 2>&1; then
            echo "  List with empty config: OK (uses defaults)"
        else
            echo "  List with empty config: FAILED"
        fi

        # Restore
        cp /tmp/backup_config.toml "$config_file" 2>/dev/null || true
    else
        echo "  Skipped (no config file)"
    fi

    section "Scenario 2: Invalid TOML Syntax"

    if [ -n "$config_file" ]; then
        echo "Writing invalid TOML..."
        cat > "$config_file" << 'EOF'
This is not valid TOML
[section
key = "unclosed string
nested = { invalid }
EOF

        local invalid_result
        if invalid_result=$("$WK_BIN" list 2>&1); then
            echo "  List with invalid TOML: OK (ignored config)"
        else
            echo "  List with invalid TOML: Error"
            if echo "$invalid_result" | grep -qi "toml\|parse\|config"; then
                echo "  Error mentions config parsing"
            fi
        fi

        # Restore
        cp /tmp/backup_config.toml "$config_file" 2>/dev/null || true
    else
        echo "  Skipped (no config file)"
    fi

    section "Scenario 3: Wrong Data Types"

    if [ -n "$config_file" ]; then
        echo "Writing config with wrong types..."
        cat > "$config_file" << 'EOF'
[project]
prefix = 123
# prefix should be a string, not a number

[database]
path = true
# path should be a string, not a boolean
EOF

        if "$WK_BIN" list >/dev/null 2>&1; then
            echo "  List with wrong types: OK (handled gracefully)"
        else
            echo "  List with wrong types: Error"
        fi

        # Restore
        cp /tmp/backup_config.toml "$config_file" 2>/dev/null || true
    else
        echo "  Skipped (no config file)"
    fi

    section "Scenario 4: Binary Content"

    if [ -n "$config_file" ]; then
        echo "Writing binary content to config..."
        head -c 100 /dev/urandom > "$config_file"

        if "$WK_BIN" list >/dev/null 2>&1; then
            echo "  List with binary config: OK (ignored)"
        else
            echo "  List with binary config: Error"
        fi

        # Restore
        cp /tmp/backup_config.toml "$config_file" 2>/dev/null || true
    else
        echo "  Skipped (no config file)"
    fi

    section "Scenario 5: Missing Config Directory"

    echo "Testing with missing .wok directory..."

    # Temporarily rename .wok
    mv .wok /tmp/work_backup

    local no_work_result
    if no_work_result=$("$WK_BIN" list 2>&1); then
        echo "  List without .wok: OK (unexpected)"
    else
        echo "  List without .wok: Error (expected)"
        if echo "$no_work_result" | grep -qi "not found\|initialize\|init"; then
            echo "  Error suggests initialization needed"
        fi
    fi

    # Restore
    mv /tmp/work_backup .wok

    section "Recovery"

    # Ensure clean state
    if [ -n "$config_file" ] && [ -f /tmp/backup_config.toml ]; then
        cp /tmp/backup_config.toml "$config_file"
    fi

    echo "Testing operations after recovery..."

    if "$WK_BIN" list >/dev/null 2>&1; then
        echo "  List: OK"
    else
        echo "  List: FAILED"
    fi

    if "$WK_BIN" new task "Post-config-corruption" >/dev/null 2>&1; then
        echo "  Create: OK"
    else
        echo "  Create: FAILED"
    fi

    section "Results"

    local integrity
    integrity=$(check_db_integrity)
    echo "  Database integrity: $integrity"

    # Clean up
    rm -f /tmp/backup_config.toml

    if [ "$integrity" = "OK" ]; then
        print_result "PASS" "Config corruption handled"
    else
        print_result "FAIL" "Issues after config corruption"
    fi
}
