#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/compare.sh - Compare benchmark results against baseline

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
THRESHOLDS_FILE="$ROOT_DIR/.github/thresholds.json"

# Regression threshold: fail if >20% slower
REGRESSION_THRESHOLD="${REGRESSION_THRESHOLD:-0.20}"

usage() {
    cat << EOF
Usage: $(basename "$0") <results_dir> <baseline_file>

Compare benchmark results against a baseline to detect regressions.

ARGUMENTS:
    results_dir     Directory containing benchmark result JSON files
    baseline_file   Path to baseline JSON file (from main branch)

ENVIRONMENT:
    REGRESSION_THRESHOLD    Percent slower to trigger failure (default: 0.20 = 20%)

EXIT CODES:
    0    No significant regressions detected
    1    One or more regressions exceed threshold
    2    Missing or invalid input files
EOF
}

if [[ $# -lt 2 ]]; then
    usage
    exit 2
fi

RESULTS_DIR="$1"
BASELINE_FILE="$2"

if [[ ! -d "$RESULTS_DIR" ]]; then
    echo "Warning: Results directory not found: $RESULTS_DIR" >&2
    echo "Skipping regression check (no baseline to compare)" >&2
    exit 0
fi

if [[ ! -f "$BASELINE_FILE" ]]; then
    echo "Warning: Baseline file not found: $BASELINE_FILE" >&2
    echo "Skipping regression check (no baseline to compare)" >&2
    exit 0
fi

echo "Comparing benchmarks against baseline..."
echo "Regression threshold: ${REGRESSION_THRESHOLD} ($(echo "$REGRESSION_THRESHOLD * 100" | bc)% slower)"
echo ""

REGRESSIONS=()

# Compare each result file against baseline
for result_file in "$RESULTS_DIR"/*.json; do
    [[ -f "$result_file" ]] || continue

    bench_name=$(basename "$result_file" .json)

    # Get mean from current result
    current_mean=$(jq -r '.results[0].mean // empty' "$result_file" 2>/dev/null)
    [[ -z "$current_mean" ]] && continue

    # Get mean from baseline for same benchmark
    baseline_mean=$(jq -r --arg name "$bench_name" '
        .[$name].mean // empty
    ' "$BASELINE_FILE" 2>/dev/null)

    if [[ -z "$baseline_mean" ]]; then
        echo "  $bench_name: NEW (no baseline)"
        continue
    fi

    # Calculate percent change
    percent_change=$(echo "scale=4; ($current_mean - $baseline_mean) / $baseline_mean" | bc -l)
    percent_display=$(echo "scale=1; $percent_change * 100" | bc -l)

    # Format times in ms
    current_ms=$(echo "scale=1; $current_mean * 1000" | bc -l)
    baseline_ms=$(echo "scale=1; $baseline_mean * 1000" | bc -l)

    # Check if regression
    is_regression=$(echo "$percent_change > $REGRESSION_THRESHOLD" | bc -l)

    if [[ "$is_regression" == "1" ]]; then
        echo "  $bench_name: ${current_ms}ms vs ${baseline_ms}ms (+${percent_display}%) REGRESSION"
        REGRESSIONS+=("$bench_name")
    elif (( $(echo "$percent_change < -$REGRESSION_THRESHOLD" | bc -l) )); then
        echo "  $bench_name: ${current_ms}ms vs ${baseline_ms}ms (${percent_display}%) IMPROVED"
    else
        echo "  $bench_name: ${current_ms}ms vs ${baseline_ms}ms (${percent_display}%) OK"
    fi
done

echo ""

if [[ ${#REGRESSIONS[@]} -gt 0 ]]; then
    echo "Performance regressions detected!"
    echo "Failed benchmarks: ${REGRESSIONS[*]}"
    exit 1
else
    echo "No significant regressions detected."
    exit 0
fi
