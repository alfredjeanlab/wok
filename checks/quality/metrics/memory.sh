#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Memory Usage Profiling

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Requires /usr/bin/time (not bash builtin)
TIME_CMD="/usr/bin/time"

get_peak_memory() {
    local cmd="$1"
    local peak_kb=0

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use /usr/bin/time -l (reports in bytes)
        local output=$( $TIME_CMD -l $cmd 2>&1 >/dev/null )
        local peak_bytes=$(echo "$output" | grep "maximum resident set size" | awk '{print $1}')
        if [ -n "$peak_bytes" ]; then
            peak_kb=$((peak_bytes / 1024))
        fi
    else
        # Linux: use /usr/bin/time -v (reports in KB)
        local output=$( { $TIME_CMD -v $cmd > /dev/null 2>&1; } 2>&1 )
        peak_kb=$(echo "$output" | grep "Maximum resident set size" | awk '{print $NF}')
    fi

    echo "$peak_kb"
}

format_memory() {
    local kb="$1"
    if [ "$kb" -eq 0 ]; then
        echo "N/A"
    elif [ "$kb" -lt 1024 ]; then
        echo "${kb} KB"
    else
        echo "$(echo "scale=1; $kb / 1024" | bc) MB"
    fi
}

measure_memory() {
    local operation="$1"

    echo "  $operation:"
    local full_cmd="$BINARY $operation"
    local peak_kb=$(get_peak_memory "$full_cmd")
    if [ "$peak_kb" -gt 0 ]; then
        echo "    Peak RSS: $(format_memory $peak_kb)"
    else
        echo "    Could not measure (command may have failed)"
    fi
}

setup_test_db() {
    local size="$1"

    rm -rf /tmp/wk-mem-test
    mkdir -p /tmp/wk-mem-test
    cd /tmp/wk-mem-test

    # Initialize
    $BINARY init --prefix mem > /dev/null 2>&1 || true

    # Create issues
    for i in $(seq 1 $size); do
        $BINARY new task "Test issue $i" > /dev/null 2>&1 || true
    done
}

echo "Memory Usage Analysis"
echo "====================="
echo ""

if [ ! -x "$TIME_CMD" ]; then
    echo "Error: $TIME_CMD not found"
    echo "This is required for memory profiling"
    exit 1
fi

BINARY="$ROOT_DIR/crates/cli/target/release/wk"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found. Building..."
    (cd "$ROOT_DIR/crates/cli" && cargo build --release) > /dev/null 2>&1 || {
        echo "Build failed"
        exit 1
    }
fi

echo "=== Memory Usage ==="

# Test 1: Startup (help command)
measure_memory "help"

# Test 2: Small DB operations
echo "  Setting up small test DB (100 issues)..."
setup_test_db 100

measure_memory "list"
measure_memory "list --all"

# Test 3: Medium DB
echo "  Setting up medium test DB (500 issues)..."
setup_test_db 500

measure_memory "list"
measure_memory "stats"

# Cleanup
rm -rf /tmp/wk-mem-test

echo ""
echo "Note: Lower memory is better"
