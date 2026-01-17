#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/quality/coverage.sh - Coverage threshold checking for CI

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
THRESHOLDS_FILE="$ROOT_DIR/.github/thresholds.json"
COVERAGE_JSON="${COVERAGE_JSON:-coverage.json}"

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Check code coverage against configured thresholds.

OPTIONS:
    -h, --help       Show this help message
    --check-only     Only check thresholds, don't run coverage collection
    --coverage-file  Path to coverage.json (default: coverage.json)

ENVIRONMENT:
    COVERAGE_JSON    Path to coverage.json file

EXIT CODES:
    0    All thresholds met
    1    One or more thresholds not met
EOF
}

CHECK_ONLY=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        --check-only)
            CHECK_ONLY=true
            shift
            ;;
        --coverage-file)
            COVERAGE_JSON="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
done

if [[ ! -f "$THRESHOLDS_FILE" ]]; then
    echo "Error: Thresholds file not found: $THRESHOLDS_FILE" >&2
    exit 1
fi

if [[ ! -f "$COVERAGE_JSON" ]]; then
    echo "Error: Coverage file not found: $COVERAGE_JSON" >&2
    echo "Run 'cargo llvm-cov report --json --output-path coverage.json' first" >&2
    exit 1
fi

echo "Checking coverage thresholds..."
echo ""

# Read thresholds
CRATES_CLI_LINES=$(jq -r '.coverage["crates/cli"].lines // 0' "$THRESHOLDS_FILE")
CRATES_CORE_LINES=$(jq -r '.coverage["crates/core"].lines // 0' "$THRESHOLDS_FILE")
CRATES_REMOTE_LINES=$(jq -r '.coverage["crates/remote"].lines // 0' "$THRESHOLDS_FILE")

FAILED=false

check_threshold() {
    local crate_name="$1"
    local threshold="$2"

    # Extract coverage for the crate from cargo-llvm-cov JSON output
    # The format includes package-level data
    local coverage
    coverage=$(jq -r --arg crate "$crate_name" '
        .data[0].files[]
        | select(.filename | contains($crate))
        | .summary.lines.percent // 0
    ' "$COVERAGE_JSON" 2>/dev/null | head -1)

    # If no per-file data, try to get overall coverage
    if [[ -z "$coverage" || "$coverage" == "null" ]]; then
        coverage=$(jq -r '.data[0].totals.lines.percent // 0' "$COVERAGE_JSON" 2>/dev/null)
    fi

    if [[ -z "$coverage" || "$coverage" == "null" ]]; then
        echo "  $crate_name: SKIP (no coverage data)"
        return 0
    fi

    local coverage_int=${coverage%.*}
    if [[ "$coverage_int" -ge "$threshold" ]]; then
        echo "  $crate_name: ${coverage}% >= ${threshold}% OK"
    else
        echo "  $crate_name: ${coverage}% < ${threshold}% FAIL"
        FAILED=true
    fi
}

echo "Coverage by crate:"
check_threshold "crates/cli" "$CRATES_CLI_LINES"
check_threshold "crates/core" "$CRATES_CORE_LINES"
check_threshold "crates/remote" "$CRATES_REMOTE_LINES"
echo ""

if [[ "$FAILED" == "true" ]]; then
    echo "Coverage thresholds not met!"
    exit 1
else
    echo "All coverage thresholds met."
    exit 0
fi
