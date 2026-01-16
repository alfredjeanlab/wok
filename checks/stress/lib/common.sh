#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Common utilities for stress tests
#
# This script sources safety.sh and provides shared helper functions
# used across all stress test scenarios.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source safety module first
source "$SCRIPT_DIR/safety.sh"

# WK binary path - required
export WK_BIN="${WK_BIN:-wk}"

# Verify wk binary exists and is executable
verify_wk_binary() {
    if [ ! -x "$WK_BIN" ]; then
        if ! command -v "$WK_BIN" &>/dev/null; then
            echo "ERROR: wk binary not found: $WK_BIN" >&2
            echo "Set WK_BIN to the path of your wk binary" >&2
            return 1
        fi
    fi
    echo "Using wk binary: $WK_BIN"
    "$WK_BIN" --version 2>/dev/null || true
}

# Initialize a fresh wk workspace
# Usage: init_workspace [prefix]
init_workspace() {
    local prefix="${1:-stress}"

    rm -rf .wok
    "$WK_BIN" init --prefix "$prefix" >/dev/null 2>&1

    if [ ! -d .wok ]; then
        echo "ERROR: Failed to initialize workspace" >&2
        return 1
    fi
}

# Get issue count from database
get_issue_count() {
    local db="${1:-.wok/issues.db}"
    if [ -f "$db" ]; then
        sqlite3 "$db" "SELECT COUNT(*) FROM issues" 2>/dev/null || echo 0
    else
        echo 0
    fi
}

# Get database size in human-readable format
get_db_size() {
    local db="${1:-.wok/issues.db}"
    if [ -f "$db" ]; then
        du -h "$db" | cut -f1
    else
        echo "0"
    fi
}

# Extract issue ID from wk output
# Usage: id=$(extract_issue_id "$(wk new task Title)")
extract_issue_id() {
    local output="$1"
    echo "$output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1
}

# Run wk command and capture result
# Usage: result=$(wk_cmd new task "Title")
wk_cmd() {
    "$WK_BIN" "$@" 2>&1
}

# Check database integrity
check_db_integrity() {
    local db="${1:-.wok/issues.db}"
    if [ -f "$db" ]; then
        if sqlite3 "$db" "PRAGMA integrity_check" 2>/dev/null | grep -q "ok"; then
            echo "OK"
            return 0
        else
            echo "CORRUPTED"
            return 1
        fi
    else
        echo "NOT_FOUND"
        return 1
    fi
}

# Time a command and return elapsed seconds
# Usage: elapsed=$(time_cmd wk list)
time_cmd() {
    local start end
    start=$(date +%s.%N)
    "$@" >/dev/null 2>&1
    local status=$?
    end=$(date +%s.%N)
    echo "$end - $start" | bc
    return $status
}

# Print test result summary
print_result() {
    local status="$1"
    local message="$2"

    case "$status" in
        PASS)
            echo "  Result: PASS - $message"
            ;;
        FAIL)
            echo "  Result: FAIL - $message"
            ;;
        SKIP)
            echo "  Result: SKIP - $message"
            ;;
        *)
            echo "  Result: $status - $message"
            ;;
    esac
}

# Print progress indicator
print_progress() {
    local current="$1"
    local total="$2"
    local interval="${3:-100}"

    if [ $((current % interval)) -eq 0 ] || [ "$current" -eq "$total" ]; then
        local percent=$((current * 100 / total))
        echo "  Progress: $current/$total ($percent%)"
    fi
}

# Wait for all background jobs with timeout
wait_with_timeout() {
    local timeout="${1:-60}"
    local start
    start=$(date +%s)

    while [ "$(jobs -p | wc -l)" -gt 0 ]; do
        local elapsed=$(($(date +%s) - start))
        if [ "$elapsed" -gt "$timeout" ]; then
            echo "Timeout waiting for background jobs, killing..."
            jobs -p | xargs -r kill -9 2>/dev/null || true
            return 1
        fi
        sleep 0.5
    done

    return 0
}

# Get random issue ID from database
get_random_issue_id() {
    local db="${1:-.wok/issues.db}"
    if [ -f "$db" ]; then
        sqlite3 "$db" "SELECT id FROM issues ORDER BY RANDOM() LIMIT 1" 2>/dev/null
    fi
}

# Get random issue ID with specific status
get_random_issue_by_status() {
    local status="$1"
    local db="${2:-.wok/issues.db}"
    if [ -f "$db" ]; then
        sqlite3 "$db" "SELECT id FROM issues WHERE status='$status' ORDER BY RANDOM() LIMIT 1" 2>/dev/null
    fi
}

# Print section header
section() {
    local title="$1"
    echo ""
    echo "--- $title ---"
}

# Log message with timestamp
log() {
    echo "[$(date '+%H:%M:%S')] $*"
}

# Check if running in container
is_containerized() {
    [ "${STRESS_CONTAINERIZED:-0}" -eq 1 ] || [ -f /.dockerenv ]
}
