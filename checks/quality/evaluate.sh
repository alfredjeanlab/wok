#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CACHE_DIR="${ROOT_DIR}/.cache/quality"
REPORT_DIR="${CACHE_DIR}/$(date +%Y%m%d_%H%M%S)"
OUTPUT_DIR="${ROOT_DIR}/reports/quality"

usage() {
    cat << EOF
Usage: $0 [OPTIONS] [METRIC...]

Quality evaluation suite for the wk CLI.

Options:
    -o, --output DIR    Output directory for reports
    -h, --help          Show this help

Metrics:
    loc           Lines of code
    file_size     Average/max LOC per file
    size          Binary/bundle size
    memory        Memory usage profiling
    coverage      Code coverage
    escapes       Escape hatch analysis
    test_time     Test suite run time
    compile_time  Cold and clean re-compile times
    commits       Git commit analysis (by conventional commit type)
    issues        Issue tracker analysis (from .beads/issues.jsonl)
    all           Run all automated metrics (default)

Agent Reviews (run separately):
    idiomaticness   Run idiomaticness review (requires Claude)
    security        Run security review (requires Claude)

Examples:
    $0                      # Run all automated metrics
    $0 loc size             # Just LOC and size
    $0 coverage             # Coverage only
EOF
}

METRICS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output) REPORT_DIR="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) METRICS+=("$1"); shift ;;
    esac
done

[ ${#METRICS[@]} -eq 0 ] && METRICS=(all)

mkdir -p "$REPORT_DIR"

# Quiet mode for running metrics (log to file only, not stdout)
QUIET=false

# Track if any metric failed (so we can exit non-zero at the end)
METRIC_FAILURES=0

run_metric() {
    local metric="$1"
    local script="${SCRIPT_DIR}/metrics/${metric}.sh"

    if [ ! -f "$script" ]; then
        echo "Error: Metric script not found: $script" >&2
        return 1
    fi

    if [ "$QUIET" = true ]; then
        if ! bash "$script" > "$REPORT_DIR/${metric}.txt" 2>&1; then
            METRIC_FAILURES=$((METRIC_FAILURES + 1))
        fi
    else
        echo "Running: $metric"
        echo "---"
        if ! bash "$script" | tee "$REPORT_DIR/${metric}.txt"; then
            METRIC_FAILURES=$((METRIC_FAILURES + 1))
        fi
        echo ""
    fi
}

# Enable quiet mode for summary generation
QUIET=true

for metric in "${METRICS[@]}"; do
    case $metric in
        all)
            run_metric loc
            run_metric file_size
            run_metric size
            run_metric memory
            run_metric coverage
            run_metric escapes
            run_metric test_time
            run_metric compile_time
            run_metric commits
            run_metric issues
            ;;
        loc|file_size|size|memory|coverage|escapes|test_time|compile_time|commits|issues)
            run_metric "$metric"
            ;;
        idiomaticness|security)
            echo "Agent review '$metric' requires manual execution."
            echo "See: ${SCRIPT_DIR}/agents/run-review.sh"
            ;;
        *)
            echo "Unknown metric: $metric"
            exit 1
            ;;
    esac
done

# --- Parse metrics into JSON ---
parse_number() {
    echo "$1" | grep -oE '[0-9]+\.?[0-9]*' | head -1
}

# Initialize JSON values - totals
json_source_loc=0
json_test_loc=0
json_source_files=0
json_test_files=0
json_source_avg=0
json_source_max=0
json_test_avg=0
json_test_max=0
json_binary_size=0
json_binary_stripped=0
json_memory_help=0
json_memory_list=0
json_coverage=0
json_escapes_unsafe=0
json_escapes_unwrap=0
json_escapes_total=0
json_test_count=0
json_test_time_cold=0
json_test_time_warm=0
json_compile_cold=0
json_compile_clean=0

# Work tracking values
json_commits_total=0
json_commits_feat=0
json_commits_fix=0
json_commits_chore=0
json_commits_refactor=0
json_commits_docs=0
json_commits_other=0
json_commits_since=""
json_commits_until=""
json_bugs_open=0
json_bugs_closed=0
json_bugs_fixed=0
json_tasks_open=0
json_tasks_closed=0
json_chores_open=0
json_chores_closed=0
json_epics_open=0
json_epics_done=0
json_features_open=0
json_features_closed=0
json_issues_since=""

# Per-package values (Bash 3.2 compatible - no associative arrays)
# crates/cli
pkg_bin_cli_source_loc=0
pkg_bin_cli_test_loc=0
pkg_bin_cli_source_files=0
pkg_bin_cli_test_files=0
pkg_bin_cli_source_avg=0
pkg_bin_cli_source_max=0
pkg_bin_cli_test_avg=0
pkg_bin_cli_test_max=0
pkg_bin_cli_escapes_unsafe=0
pkg_bin_cli_escapes_unwrap=0
pkg_bin_cli_escapes_total=0
pkg_bin_cli_coverage=0
pkg_bin_cli_test_count=0

# crates/remote
pkg_bin_remote_source_loc=0
pkg_bin_remote_test_loc=0
pkg_bin_remote_source_files=0
pkg_bin_remote_test_files=0
pkg_bin_remote_source_avg=0
pkg_bin_remote_source_max=0
pkg_bin_remote_test_avg=0
pkg_bin_remote_test_max=0
pkg_bin_remote_escapes_unsafe=0
pkg_bin_remote_escapes_unwrap=0
pkg_bin_remote_escapes_total=0
pkg_bin_remote_coverage=0
pkg_bin_remote_test_count=0

# crates/core
pkg_lib_core_source_loc=0
pkg_lib_core_test_loc=0
pkg_lib_core_source_files=0
pkg_lib_core_test_files=0
pkg_lib_core_source_avg=0
pkg_lib_core_source_max=0
pkg_lib_core_test_avg=0
pkg_lib_core_test_max=0
pkg_lib_core_escapes_unsafe=0
pkg_lib_core_escapes_unwrap=0
pkg_lib_core_escapes_total=0
pkg_lib_core_coverage=0
pkg_lib_core_test_count=0

# Helper to parse per-package LOC
parse_pkg_loc() {
    local pkg="$1"
    local file="$2"
    local pkg_block=$(grep -A3 "^${pkg}:" "$file" 2>/dev/null || true)
    if [ -n "$pkg_block" ]; then
        echo "$(echo "$pkg_block" | grep "Source:" | grep -oE '[0-9]+' | head -1 || echo 0)"
        echo "$(echo "$pkg_block" | grep "Source:" | grep -oE '[0-9]+' | tail -1 || echo 0)"
        echo "$(echo "$pkg_block" | grep "Test:" | grep -oE '[0-9]+' | head -1 || echo 0)"
        echo "$(echo "$pkg_block" | grep "Test:" | grep -oE '[0-9]+' | tail -1 || echo 0)"
    else
        echo "0"; echo "0"; echo "0"; echo "0"
    fi
}

# Helper to parse per-package file_size
parse_pkg_file_size() {
    local pkg="$1"
    local file="$2"
    local pkg_block=$(grep -A3 "^${pkg}:" "$file" 2>/dev/null || true)
    if [ -n "$pkg_block" ]; then
        local src_line=$(echo "$pkg_block" | grep "Source:")
        local test_line=$(echo "$pkg_block" | grep "Test:")
        echo "$(echo "$src_line" | grep -oE 'avg [0-9]+' | grep -oE '[0-9]+' || echo 0)"
        echo "$(echo "$src_line" | grep -oE 'max [0-9]+' | grep -oE '[0-9]+' || echo 0)"
        echo "$(echo "$test_line" | grep -oE 'avg [0-9]+' | grep -oE '[0-9]+' || echo 0)"
        echo "$(echo "$test_line" | grep -oE 'max [0-9]+' | grep -oE '[0-9]+' || echo 0)"
    else
        echo "0"; echo "0"; echo "0"; echo "0"
    fi
}

# Helper to parse per-package escapes
parse_pkg_escapes() {
    local pkg="$1"
    local file="$2"
    local pkg_block=$(grep -A4 "^${pkg}:" "$file" 2>/dev/null || true)
    if [ -n "$pkg_block" ]; then
        local escape_line=$(echo "$pkg_block" | grep "unsafe:")
        echo "$(echo "$escape_line" | grep -oE 'unsafe: [0-9]+' | grep -oE '[0-9]+' || echo 0)"
        echo "$(echo "$escape_line" | grep -oE 'unwrap: [0-9]+' | grep -oE '[0-9]+' || echo 0)"
        echo "$(echo "$pkg_block" | grep "High-risk total:" | grep -oE '[0-9]+' || echo 0)"
    else
        echo "0"; echo "0"; echo "0"
    fi
}

# Helper to parse per-package coverage
# Format: --- crates/xxx --- followed by test results and TOTAL line
parse_pkg_coverage() {
    local pkg="$1"
    local file="$2"
    # Escape forward slashes for sed pattern
    local pkg_escaped=$(echo "$pkg" | sed 's/\//\\\//g')
    # Extract section from "--- pkg ---" until next "---" or "==="
    # Use sed -E for extended regex (needed on macOS), then remove last line (the next section header)
    local pkg_section=$(sed -n -E "/^--- ${pkg_escaped} ---/,/^---|^===/p" "$file" 2>/dev/null | sed '$d')
    if [ -n "$pkg_section" ]; then
        # Get line coverage from TOTAL line (Lines Cover is at position NF-3)
        local coverage=$(echo "$pkg_section" | grep "^TOTAL" | tail -1 | awk '{print $(NF-3)}' | tr -d '%' || echo 0)
        # Sum all passed tests in section
        local test_count=$(echo "$pkg_section" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | awk '{sum+=$1} END {print sum}' || echo 0)
        echo "${coverage:-0}"
        echo "${test_count:-0}"
    else
        echo "0"; echo "0"
    fi
}

# Parse LOC (new format with per-package)
if [ -f "$REPORT_DIR/loc.txt" ]; then
    # Parse per-package LOC
    read pkg_bin_cli_source_files pkg_bin_cli_source_loc pkg_bin_cli_test_files pkg_bin_cli_test_loc <<< $(parse_pkg_loc "crates/cli" "$REPORT_DIR/loc.txt" | tr '\n' ' ')
    read pkg_bin_remote_source_files pkg_bin_remote_source_loc pkg_bin_remote_test_files pkg_bin_remote_test_loc <<< $(parse_pkg_loc "crates/remote" "$REPORT_DIR/loc.txt" | tr '\n' ' ')
    read pkg_lib_core_source_files pkg_lib_core_source_loc pkg_lib_core_test_files pkg_lib_core_test_loc <<< $(parse_pkg_loc "crates/core" "$REPORT_DIR/loc.txt" | tr '\n' ' ')

    # Parse totals (from === Total === section)
    total_section=$(sed -n '/=== Total ===/,/Notes:/p' "$REPORT_DIR/loc.txt")
    json_source_files=$(echo "$total_section" | grep "Files:" | head -1 | grep -oE '[0-9]+' || echo 0)
    json_source_loc=$(echo "$total_section" | grep "LOC:" | head -1 | grep -oE '[0-9]+' || echo 0)
    json_test_files=$(echo "$total_section" | grep "Files:" | tail -1 | grep -oE '[0-9]+' || echo 0)
    json_test_loc=$(echo "$total_section" | grep "LOC:" | tail -1 | grep -oE '[0-9]+' || echo 0)
fi

# Parse file_size (new format with per-package)
if [ -f "$REPORT_DIR/file_size.txt" ]; then
    # Parse per-package file_size
    read pkg_bin_cli_source_avg pkg_bin_cli_source_max pkg_bin_cli_test_avg pkg_bin_cli_test_max <<< $(parse_pkg_file_size "crates/cli" "$REPORT_DIR/file_size.txt" | tr '\n' ' ')
    read pkg_bin_remote_source_avg pkg_bin_remote_source_max pkg_bin_remote_test_avg pkg_bin_remote_test_max <<< $(parse_pkg_file_size "crates/remote" "$REPORT_DIR/file_size.txt" | tr '\n' ' ')
    read pkg_lib_core_source_avg pkg_lib_core_source_max pkg_lib_core_test_avg pkg_lib_core_test_max <<< $(parse_pkg_file_size "crates/core" "$REPORT_DIR/file_size.txt" | tr '\n' ' ')

    # Parse totals
    total_section=$(sed -n '/=== Total ===/,/Result:/p' "$REPORT_DIR/file_size.txt")
    json_source_avg=$(echo "$total_section" | grep "Average:" | head -1 | grep -oE '[0-9]+' | head -1 || echo 0)
    json_source_max=$(echo "$total_section" | grep "Max:" | head -1 | grep -oE '[0-9]+' | head -1 || echo 0)
    json_test_avg=$(echo "$total_section" | grep "Average:" | tail -1 | grep -oE '[0-9]+' | head -1 || echo 0)
    json_test_max=$(echo "$total_section" | grep "Max:" | tail -1 | grep -oE '[0-9]+' | head -1 || echo 0)
fi

# Parse binary size
if [ -f "$REPORT_DIR/size.txt" ]; then
    json_binary_size=$(grep "Release binary:" "$REPORT_DIR/size.txt" | grep -oE '\([0-9]+ bytes\)' | grep -oE '[0-9]+' || echo 0)
    json_binary_stripped=$(grep "Stripped:" "$REPORT_DIR/size.txt" | grep -oE '\([0-9]+ bytes\)' | grep -oE '[0-9]+' || echo 0)
fi

# Parse memory
if [ -f "$REPORT_DIR/memory.txt" ]; then
    json_memory_help=$(grep -A1 "help:" "$REPORT_DIR/memory.txt" | grep "Peak RSS" | grep -oE '[0-9]+\.?[0-9]*' | head -1 || echo 0)
    json_memory_list=$(grep -A1 "list:" "$REPORT_DIR/memory.txt" | grep "Peak RSS" | head -1 | grep -oE '[0-9]+\.?[0-9]*' | head -1 || echo 0)
fi

# Parse coverage
if [ -f "$REPORT_DIR/coverage.txt" ]; then
    # Parse per-package coverage
    read pkg_bin_cli_coverage pkg_bin_cli_test_count <<< $(parse_pkg_coverage "crates/cli" "$REPORT_DIR/coverage.txt" | tr '\n' ' ')
    read pkg_bin_remote_coverage pkg_bin_remote_test_count <<< $(parse_pkg_coverage "crates/remote" "$REPORT_DIR/coverage.txt" | tr '\n' ' ')
    read pkg_lib_core_coverage pkg_lib_core_test_count <<< $(parse_pkg_coverage "crates/core" "$REPORT_DIR/coverage.txt" | tr '\n' ' ')

    # Get the last TOTAL line (overall total)
    json_coverage=$(grep "^TOTAL" "$REPORT_DIR/coverage.txt" | tail -1 | awk '{print $(NF-3)}' | tr -d '%' || echo 0)
    # Sum all passed tests
    json_test_count=$(grep -oE '[0-9]+ passed' "$REPORT_DIR/coverage.txt" | grep -oE '[0-9]+' | awk '{sum+=$1} END {print sum}' || echo 0)
fi

# Parse escapes (new format with per-package)
if [ -f "$REPORT_DIR/escapes.txt" ]; then
    # Parse per-package escapes
    read pkg_bin_cli_escapes_unsafe pkg_bin_cli_escapes_unwrap pkg_bin_cli_escapes_total <<< $(parse_pkg_escapes "crates/cli" "$REPORT_DIR/escapes.txt" | tr '\n' ' ')
    read pkg_bin_remote_escapes_unsafe pkg_bin_remote_escapes_unwrap pkg_bin_remote_escapes_total <<< $(parse_pkg_escapes "crates/remote" "$REPORT_DIR/escapes.txt" | tr '\n' ' ')
    read pkg_lib_core_escapes_unsafe pkg_lib_core_escapes_unwrap pkg_lib_core_escapes_total <<< $(parse_pkg_escapes "crates/core" "$REPORT_DIR/escapes.txt" | tr '\n' ' ')

    # Parse totals
    json_escapes_unsafe=$(grep "^unsafe blocks:" "$REPORT_DIR/escapes.txt" | grep -oE '[0-9]+' || echo 0)
    json_escapes_unwrap=$(grep "^.unwrap() calls:" "$REPORT_DIR/escapes.txt" | grep -oE '[0-9]+' || echo 0)
    json_escapes_total=$(grep "^Total high-risk" "$REPORT_DIR/escapes.txt" | grep -oE '[0-9]+' | head -1 || echo 0)
fi

# Parse test_time
if [ -f "$REPORT_DIR/test_time.txt" ]; then
    json_test_time_cold=$(grep "Cold test time:" "$REPORT_DIR/test_time.txt" | grep -oE '[0-9]+\.?[0-9]*' || echo 0)
    json_test_time_warm=$(grep "Warm test time:" "$REPORT_DIR/test_time.txt" | grep -oE '[0-9]+\.?[0-9]*' || echo 0)
fi

# Parse compile_time
if [ -f "$REPORT_DIR/compile_time.txt" ]; then
    json_compile_cold=$(grep "Cold compile time:" "$REPORT_DIR/compile_time.txt" | grep -oE '[0-9]+\.?[0-9]*' || echo 0)
    json_compile_clean=$(grep "Clean re-compile time:" "$REPORT_DIR/compile_time.txt" | grep -oE '[0-9]+\.?[0-9]*' || echo 0)
fi

# Parse commits
if [ -f "$REPORT_DIR/commits.txt" ]; then
    json_commits_total=$(grep "^total:" "$REPORT_DIR/commits.txt" | grep -oE '[0-9]+' || echo 0)
    json_commits_feat=$(grep "^feat:" "$REPORT_DIR/commits.txt" | grep -oE '[0-9]+' || echo 0)
    json_commits_fix=$(grep "^fix:" "$REPORT_DIR/commits.txt" | grep -oE '[0-9]+' || echo 0)
    json_commits_chore=$(grep "^chore:" "$REPORT_DIR/commits.txt" | grep -oE '[0-9]+' || echo 0)
    json_commits_refactor=$(grep "^refactor:" "$REPORT_DIR/commits.txt" | grep -oE '[0-9]+' || echo 0)
    json_commits_docs=$(grep "^docs:" "$REPORT_DIR/commits.txt" | grep -oE '[0-9]+' || echo 0)
    json_commits_other=$(grep "^other:" "$REPORT_DIR/commits.txt" | grep -oE '[0-9]+' || echo 0)
    json_commits_since=$(grep "^since:" "$REPORT_DIR/commits.txt" | awk '{print $2}' || echo "")
    json_commits_until=$(grep "^until:" "$REPORT_DIR/commits.txt" | awk '{print $2}' || echo "")
fi

# Parse issues
if [ -f "$REPORT_DIR/issues.txt" ]; then
    json_bugs_open=$(grep "^bugs_open:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_bugs_closed=$(grep "^bugs_closed:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_bugs_fixed=$(grep "^bugs_fixed:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_tasks_open=$(grep "^tasks_open:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_tasks_closed=$(grep "^tasks_closed:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_chores_open=$(grep "^chores_open:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_chores_closed=$(grep "^chores_closed:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_epics_open=$(grep "^epics_open:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_epics_done=$(grep "^epics_done:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_features_open=$(grep "^features_open:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_features_closed=$(grep "^features_closed:" "$REPORT_DIR/issues.txt" | grep -oE '[0-9]+' || echo 0)
    json_issues_since=$(grep "^since:" "$REPORT_DIR/issues.txt" | awk '{print $2}' || echo "")
fi

# --- Source output modules ---
source "${SCRIPT_DIR}/output/json.sh"
source "${SCRIPT_DIR}/output/comparison.sh"
source "${SCRIPT_DIR}/output/summary.sh"

# --- Generate outputs ---
generate_metrics_json "$REPORT_DIR"

# Find previous report for comparison
PREV_REPORT=""
CURRENT_NAME=$(basename "$REPORT_DIR")
PREV_REPORT=$(find_previous_report "$CACHE_DIR" "$CURRENT_NAME")

# Generate comparison if previous report exists
if [ -n "$PREV_REPORT" ] && [ -f "$PREV_REPORT/metrics.json" ]; then
    generate_comparison "$REPORT_DIR" "$PREV_REPORT"
fi

# Generate human-readable summary
generate_summary "$REPORT_DIR"

# --- Cleanup old reports within 1 hour of newer ones ---
cleanup_old_reports() {
    local reports_dir="$1"
    local one_hour=3600  # seconds

    # Get all report directories sorted newest first
    local dirs=()
    while IFS= read -r dir; do
        [ -d "$dir" ] && dirs+=("$dir")
    done < <(ls -1d "$reports_dir"/20* 2>/dev/null | sort -r)

    # Compare each pair and remove older if within 2 hours
    local i=0
    while [ $i -lt ${#dirs[@]} ]; do
        local newer="${dirs[$i]}"
        local newer_name=$(basename "$newer")

        # Parse timestamp from directory name (YYYYMMDD_HHMMSS)
        local newer_date="${newer_name:0:8}"
        local newer_time="${newer_name:9:6}"
        local newer_ts=$(date -j -f "%Y%m%d%H%M%S" "${newer_date}${newer_time}" "+%s" 2>/dev/null || echo 0)

        local j=$((i + 1))
        while [ $j -lt ${#dirs[@]} ]; do
            local older="${dirs[$j]}"
            local older_name=$(basename "$older")

            local older_date="${older_name:0:8}"
            local older_time="${older_name:9:6}"
            local older_ts=$(date -j -f "%Y%m%d%H%M%S" "${older_date}${older_time}" "+%s" 2>/dev/null || echo 0)

            if [ "$newer_ts" -gt 0 ] && [ "$older_ts" -gt 0 ]; then
                local diff=$((newer_ts - older_ts))
                if [ "$diff" -ge 0 ] && [ "$diff" -lt "$one_hour" ]; then
                    echo "Removing old report $older_name (within 1h of $newer_name)"
                    rm -rf "$older"
                    # Remove from array
                    unset 'dirs[$j]'
                    dirs=("${dirs[@]}")
                    continue  # Don't increment j since array shifted
                fi
            fi
            j=$((j + 1))
        done
        i=$((i + 1))
    done
}

# Run cleanup silently
cleanup_old_reports "$CACHE_DIR" >/dev/null 2>&1

# --- Copy summary and metrics to reports/quality with human-friendly name ---
mkdir -p "$OUTPUT_DIR"
HUMAN_DATE=$(date +%Y-%m-%d)
HUMAN_TIME=$(date +%H%M%S)

# Check if a report already exists for today - if so, include time to avoid overwrite
if [ -f "$OUTPUT_DIR/${HUMAN_DATE}.json" ] || [ -f "$OUTPUT_DIR/${HUMAN_DATE}.md" ]; then
    REPORT_NAME="${HUMAN_DATE}_${HUMAN_TIME}"
else
    REPORT_NAME="${HUMAN_DATE}"
fi

cp "$REPORT_DIR/summary.md" "$OUTPUT_DIR/${REPORT_NAME}.md"
cp "$REPORT_DIR/metrics.json" "$OUTPUT_DIR/${REPORT_NAME}.json"
echo "Summary copied to: $OUTPUT_DIR/${REPORT_NAME}.md"
echo "Metrics copied to: $OUTPUT_DIR/${REPORT_NAME}.json"

# Report any metric collection failures (but don't block CI)
if [ "$METRIC_FAILURES" -gt 0 ]; then
    echo ""
    echo "Warning: $METRIC_FAILURES metric(s) failed during evaluation"
fi
