#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Resource monitoring utilities for stress tests
#
# Provides functions to monitor system resources during stress tests:
# - Memory usage tracking
# - CPU/load monitoring
# - Disk space monitoring
# - Process resource tracking

set -euo pipefail

# Get current process memory usage in KB
# Usage: mem_kb=$(get_process_memory $$)
get_process_memory() {
    local pid="${1:-$$}"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use ps
        ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ' || echo 0
    else
        # Linux: read from /proc
        if [ -f "/proc/$pid/status" ]; then
            grep VmRSS "/proc/$pid/status" 2>/dev/null | awk '{print $2}' || echo 0
        else
            ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ' || echo 0
        fi
    fi
}

# Get peak memory usage of a command
# Usage: peak_kb=$(measure_peak_memory command arg1 arg2)
measure_peak_memory() {
    local peak_kb=0

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: use /usr/bin/time -l
        local output
        output=$({ /usr/bin/time -l "$@" >/dev/null; } 2>&1)
        peak_kb=$(echo "$output" | grep "maximum resident" | awk '{print $1}')
        # Convert from bytes to KB
        peak_kb=$((peak_kb / 1024))
    else
        # Linux: use /usr/bin/time -v
        local output
        output=$({ /usr/bin/time -v "$@" >/dev/null; } 2>&1)
        peak_kb=$(echo "$output" | grep "Maximum resident" | awk '{print $6}')
    fi

    echo "${peak_kb:-0}"
}

# Get system memory stats
# Usage: read total used free <<< "$(get_memory_stats)"
get_memory_stats() {
    local total_kb used_kb free_kb

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        local page_size
        page_size=$(pagesize 2>/dev/null || echo 4096)

        local stats
        stats=$(vm_stat 2>/dev/null)

        local pages_active pages_wired pages_free pages_inactive
        pages_active=$(echo "$stats" | awk '/Pages active/ {gsub(/\./,"",$3); print $3}')
        pages_wired=$(echo "$stats" | awk '/Pages wired/ {gsub(/\./,"",$4); print $4}')
        pages_free=$(echo "$stats" | awk '/Pages free/ {gsub(/\./,"",$3); print $3}')
        pages_inactive=$(echo "$stats" | awk '/Pages inactive/ {gsub(/\./,"",$3); print $3}')

        pages_active=${pages_active:-0}
        pages_wired=${pages_wired:-0}
        pages_free=${pages_free:-0}
        pages_inactive=${pages_inactive:-0}

        total_kb=$(( (pages_active + pages_wired + pages_free + pages_inactive) * page_size / 1024 ))
        used_kb=$(( (pages_active + pages_wired) * page_size / 1024 ))
        free_kb=$(( (pages_free + pages_inactive) * page_size / 1024 ))
    else
        # Linux
        read -r total_kb used_kb free_kb <<< "$(free -k | awk '/^Mem:/ {print $2, $3, $4}')"
    fi

    echo "$total_kb $used_kb $free_kb"
}

# Get disk usage stats for a path
# Usage: read total used avail percent <<< "$(get_disk_stats /tmp)"
get_disk_stats() {
    local path="${1:-.}"

    df -k "$path" 2>/dev/null | awk 'NR==2 {
        gsub(/%/,"",$5);
        print $2, $3, $4, $5
    }'
}

# Get CPU load averages
# Usage: read load1 load5 load15 <<< "$(get_load_averages)"
get_load_averages() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sysctl -n vm.loadavg 2>/dev/null | awk '{print $2, $3, $4}'
    else
        cat /proc/loadavg 2>/dev/null | awk '{print $1, $2, $3}'
    fi
}

# Monitor a process until it exits
# Usage: monitor_process $pid 1 > metrics.log &
monitor_process() {
    local pid="$1"
    local interval="${2:-1}"

    echo "timestamp,memory_kb,cpu_percent"

    while kill -0 "$pid" 2>/dev/null; do
        local timestamp mem_kb cpu_pct

        timestamp=$(date +%s)
        mem_kb=$(get_process_memory "$pid")

        if [[ "$OSTYPE" == "darwin"* ]]; then
            cpu_pct=$(ps -o %cpu= -p "$pid" 2>/dev/null | tr -d ' ' || echo 0)
        else
            cpu_pct=$(ps -o %cpu= -p "$pid" 2>/dev/null | tr -d ' ' || echo 0)
        fi

        echo "$timestamp,$mem_kb,$cpu_pct"
        sleep "$interval"
    done
}

# Start background resource monitor
# Usage: start_monitor output.csv 1
#        ... do work ...
#        stop_monitor
start_monitor() {
    local output_file="${1:-/tmp/stress_monitor.csv}"
    local interval="${2:-1}"

    {
        echo "timestamp,load1,mem_used_mb,disk_used_pct"

        while true; do
            local timestamp load1 mem_stats disk_stats
            timestamp=$(date +%s)

            read -r load1 _ _ <<< "$(get_load_averages)"
            read -r _ used_kb _ <<< "$(get_memory_stats)"
            read -r _ _ _ disk_pct <<< "$(get_disk_stats "${STRESS_SANDBOX:-/tmp}")"

            local used_mb=$((used_kb / 1024))
            echo "$timestamp,$load1,$used_mb,${disk_pct:-0}"

            sleep "$interval"
        done
    } > "$output_file" &

    echo $!
}

# Stop background monitor
stop_monitor() {
    local monitor_pid="$1"
    if [ -n "$monitor_pid" ] && kill -0 "$monitor_pid" 2>/dev/null; then
        kill "$monitor_pid" 2>/dev/null || true
    fi
}

# Print resource summary
print_resource_summary() {
    local label="${1:-Current resources}"

    echo "$label:"

    # Memory
    read -r total_kb used_kb free_kb <<< "$(get_memory_stats)"
    local total_mb=$((total_kb / 1024))
    local used_mb=$((used_kb / 1024))
    local free_mb=$((free_kb / 1024))
    local mem_pct=$((used_kb * 100 / total_kb))
    echo "  Memory: ${used_mb}MB / ${total_mb}MB (${mem_pct}% used)"

    # Disk
    read -r disk_total disk_used disk_avail disk_pct <<< "$(get_disk_stats "${STRESS_SANDBOX:-/tmp}")"
    local disk_total_gb=$((disk_total / 1024 / 1024))
    local disk_avail_gb=$((disk_avail / 1024 / 1024))
    echo "  Disk: ${disk_avail_gb}GB available (${disk_pct}% used)"

    # Load
    read -r load1 load5 load15 <<< "$(get_load_averages)"
    echo "  Load: $load1 (1m), $load5 (5m), $load15 (15m)"
}

# Wait and report progress
wait_with_progress() {
    local total="$1"
    local description="${2:-Processing}"
    local interval="${3:-5}"

    local elapsed=0
    while [ "$(jobs -p | wc -l)" -gt 0 ]; do
        sleep "$interval"
        elapsed=$((elapsed + interval))
        echo "  $description: ${elapsed}s elapsed..."

        if [ "$elapsed" -ge "$total" ]; then
            echo "  Timeout reached, killing remaining jobs..."
            jobs -p | xargs -r kill -9 2>/dev/null || true
            return 1
        fi
    done

    return 0
}
