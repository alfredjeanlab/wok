#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Safety wrapper for stress tests - prevents host system damage
#
# All stress tests MUST use safe_stress_test() wrapper to ensure:
# - Resource limits are applied (memory, processes, files)
# - Tests run in isolated sandbox directory
# - System health is monitored and tests abort if thresholds exceeded
# - Cleanup happens on exit (normal or abnormal)

set -euo pipefail

# Default safety limits (can be overridden with env vars)
STRESS_MAX_MEMORY_MB="${STRESS_MAX_MEMORY_MB:-2048}"       # 2GB max
STRESS_MAX_DISK_MB="${STRESS_MAX_DISK_MB:-5120}"           # 5GB max
STRESS_MAX_PROCS="${STRESS_MAX_PROCS:-100}"                # 100 processes
STRESS_MAX_FILES="${STRESS_MAX_FILES:-1024}"               # 1024 open files
STRESS_TIMEOUT_SEC="${STRESS_TIMEOUT_SEC:-300}"            # 5 min timeout
STRESS_MIN_FREE_DISK_MB="${STRESS_MIN_FREE_DISK_MB:-1024}" # Keep 1GB free

# Monitoring thresholds (abort if exceeded)
ABORT_MEMORY_PERCENT="${ABORT_MEMORY_PERCENT:-80}"    # Abort if system memory > 80% used
ABORT_DISK_PERCENT="${ABORT_DISK_PERCENT:-90}"        # Abort if disk > 90% used
ABORT_LOAD_AVERAGE="${ABORT_LOAD_AVERAGE:-8}"         # Abort if load average > 8

# Sandbox state
STRESS_SANDBOX=""
STRESS_SANDBOX_INITIALIZED=0

# Create isolated sandbox directory
setup_sandbox() {
    if [ "$STRESS_SANDBOX_INITIALIZED" -eq 1 ]; then
        return 0
    fi

    STRESS_SANDBOX=$(mktemp -d "${TMPDIR:-/tmp}/wk-stress-XXXXXX")
    cd "$STRESS_SANDBOX"
    trap cleanup_sandbox EXIT INT TERM
    STRESS_SANDBOX_INITIALIZED=1
    echo "Sandbox: $STRESS_SANDBOX"
}

# Clean up sandbox on exit
cleanup_sandbox() {
    local exit_code=$?

    # Kill any child processes
    pkill -P $$ 2>/dev/null || true

    # Wait briefly for children to exit
    sleep 0.5

    cd /
    if [ -n "$STRESS_SANDBOX" ] && [ -d "$STRESS_SANDBOX" ]; then
        rm -rf "$STRESS_SANDBOX"
        echo "Sandbox cleaned up"
    fi

    exit $exit_code
}

# Apply resource limits via ulimit
apply_limits() {
    local applied=""

    # Memory limit (in KB) - virtual memory
    if ulimit -v $((STRESS_MAX_MEMORY_MB * 1024)) 2>/dev/null; then
        applied="${applied}mem=${STRESS_MAX_MEMORY_MB}MB "
    else
        echo "Warning: Could not set memory limit" >&2
    fi

    # Max processes
    if ulimit -u "$STRESS_MAX_PROCS" 2>/dev/null; then
        applied="${applied}procs=$STRESS_MAX_PROCS "
    else
        echo "Warning: Could not set process limit" >&2
    fi

    # Max open files
    if ulimit -n "$STRESS_MAX_FILES" 2>/dev/null; then
        applied="${applied}files=$STRESS_MAX_FILES "
    else
        echo "Warning: Could not set file limit" >&2
    fi

    # Core dump size (disable to save disk)
    ulimit -c 0 2>/dev/null || true

    if [ -n "$applied" ]; then
        echo "Limits applied: $applied"
    fi
}

# Get current memory usage percentage
get_memory_percent() {
    local mem_percent

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use vm_stat
        local page_size
        page_size=$(pagesize 2>/dev/null || echo 4096)

        local stats
        stats=$(vm_stat 2>/dev/null)

        local pages_active pages_wired pages_free pages_inactive
        pages_active=$(echo "$stats" | awk '/Pages active/ {gsub(/\./,"",$3); print $3}')
        pages_wired=$(echo "$stats" | awk '/Pages wired/ {gsub(/\./,"",$4); print $4}')
        pages_free=$(echo "$stats" | awk '/Pages free/ {gsub(/\./,"",$3); print $3}')
        pages_inactive=$(echo "$stats" | awk '/Pages inactive/ {gsub(/\./,"",$3); print $3}')

        # Default to 0 if parsing fails
        pages_active=${pages_active:-0}
        pages_wired=${pages_wired:-0}
        pages_free=${pages_free:-0}
        pages_inactive=${pages_inactive:-0}

        local total=$((pages_active + pages_wired + pages_free + pages_inactive))
        local used=$((pages_active + pages_wired))

        if [ "$total" -gt 0 ]; then
            mem_percent=$((used * 100 / total))
        else
            mem_percent=0
        fi
    else
        # Linux: use free
        mem_percent=$(free 2>/dev/null | awk '/^Mem:/ {printf "%.0f", $3/$2 * 100}')
        mem_percent=${mem_percent:-0}
    fi

    echo "$mem_percent"
}

# Get current disk usage percentage for sandbox
get_disk_percent() {
    local path="${1:-${STRESS_SANDBOX:-/tmp}}"
    local disk_percent

    disk_percent=$(df "$path" 2>/dev/null | awk 'NR==2 {gsub(/%/,"",$5); print $5}')
    echo "${disk_percent:-0}"
}

# Get current load average (1 minute)
get_load_average() {
    local load_avg
    load_avg=$(uptime | awk -F'load averages?:' '{print $2}' | awk -F, '{print $1}' | tr -d ' ')
    echo "${load_avg:-0}"
}

# Check if safe to continue
check_system_health() {
    local verbose="${1:-0}"

    # Check memory usage
    local mem_percent
    mem_percent=$(get_memory_percent)

    if [ "$mem_percent" -gt "$ABORT_MEMORY_PERCENT" ]; then
        echo "ABORT: Memory usage ${mem_percent}% exceeds ${ABORT_MEMORY_PERCENT}%"
        return 1
    fi
    [ "$verbose" -eq 1 ] && echo "  Memory: ${mem_percent}%"

    # Check disk usage
    local disk_percent
    disk_percent=$(get_disk_percent)

    if [ "$disk_percent" -gt "$ABORT_DISK_PERCENT" ]; then
        echo "ABORT: Disk usage ${disk_percent}% exceeds ${ABORT_DISK_PERCENT}%"
        return 1
    fi
    [ "$verbose" -eq 1 ] && echo "  Disk: ${disk_percent}%"

    # Check load average
    local load_avg
    load_avg=$(get_load_average)
    local load_int=${load_avg%.*}
    load_int=${load_int:-0}

    if [ "$load_int" -gt "$ABORT_LOAD_AVERAGE" ]; then
        echo "ABORT: Load average $load_avg exceeds $ABORT_LOAD_AVERAGE"
        return 1
    fi
    [ "$verbose" -eq 1 ] && echo "  Load: ${load_avg}"

    return 0
}

# Check available disk space before test
check_disk_space() {
    local required_mb="${1:-$STRESS_MAX_DISK_MB}"
    local check_path="${2:-${TMPDIR:-/tmp}}"

    local available_mb
    available_mb=$(df -m "$check_path" 2>/dev/null | awk 'NR==2 {print $4}')
    available_mb=${available_mb:-0}

    local min_required=$((required_mb + STRESS_MIN_FREE_DISK_MB))

    if [ "$available_mb" -lt "$min_required" ]; then
        echo "ABORT: Insufficient disk space. Need ${min_required}MB, have ${available_mb}MB"
        return 1
    fi
    echo "Disk space OK: ${available_mb}MB available"
    return 0
}

# Run command with timeout
run_with_timeout() {
    local timeout_sec="${1:-$STRESS_TIMEOUT_SEC}"
    shift

    # Use gtimeout on macOS if available, otherwise timeout
    local timeout_cmd="timeout"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if command -v gtimeout &>/dev/null; then
            timeout_cmd="gtimeout"
        elif ! command -v timeout &>/dev/null; then
            # Fallback for macOS without coreutils
            echo "Warning: timeout command not available, running without timeout" >&2
            "$@"
            return $?
        fi
    fi

    "$timeout_cmd" "$timeout_sec" "$@"
    local status=$?

    if [ $status -eq 124 ]; then
        echo "TIMEOUT: Command exceeded ${timeout_sec}s limit"
    fi

    return $status
}

# Monitor and abort wrapper - runs command with periodic health checks
run_monitored() {
    local check_interval="${1:-5}"
    shift

    # Start the command in background
    "$@" &
    local cmd_pid=$!

    # Monitor loop
    while kill -0 $cmd_pid 2>/dev/null; do
        sleep "$check_interval"
        if ! check_system_health; then
            echo "Killing stress test due to resource limits..."
            kill -TERM $cmd_pid 2>/dev/null || true
            sleep 2
            kill -KILL $cmd_pid 2>/dev/null || true
            wait $cmd_pid 2>/dev/null || true
            return 1
        fi
    done

    wait $cmd_pid
}

# Safe wrapper for all stress tests
# Usage: safe_stress_test "Test Name" test_function arg1 arg2 ...
safe_stress_test() {
    local test_name="$1"
    shift

    echo "=== Starting: $test_name ==="
    echo "Safety limits:"
    echo "  Memory: ${STRESS_MAX_MEMORY_MB}MB"
    echo "  Disk: ${STRESS_MAX_DISK_MB}MB"
    echo "  Timeout: ${STRESS_TIMEOUT_SEC}s"
    echo ""

    setup_sandbox
    apply_limits

    if ! check_disk_space; then
        return 1
    fi

    if ! check_system_health 1; then
        echo "System already under stress, aborting"
        return 1
    fi

    echo ""

    # Run with monitoring
    local start_time
    start_time=$(date +%s)

    run_monitored 5 run_with_timeout "$STRESS_TIMEOUT_SEC" "$@"
    local status=$?

    local end_time
    end_time=$(date +%s)
    local elapsed=$((end_time - start_time))

    echo ""
    echo "=== Completed: $test_name (exit: $status, ${elapsed}s) ==="

    return $status
}

# Gradual escalation helper
# Usage: escalate_test "checkpoint1 checkpoint2 ..." test_function
escalate_test() {
    local checkpoints_str="$1"
    shift
    local test_func="$1"
    shift

    local -a checkpoints
    read -ra checkpoints <<< "$checkpoints_str"

    for count in "${checkpoints[@]}"; do
        echo "Testing with $count..."

        if ! check_system_health; then
            echo "Stopping escalation at $count due to resource pressure"
            return 1
        fi

        if ! "$test_func" "$count" "$@"; then
            echo "Test failed at $count"
            return 1
        fi

        echo "Checkpoint $count passed"
        # Brief pause to let system recover
        sleep 2
    done

    echo "All checkpoints passed"
    return 0
}
