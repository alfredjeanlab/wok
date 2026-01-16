#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# File Size measurement
# Measures average and max LOC per source/test file
# Target: source avg <500, max <900; test avg <700, max <1100

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Compute average and max LOC from cloc --by-file output
# Returns: avg count max max_file
avg_max_loc() {
    local path="$1"
    local lang="$2"
    local match_pattern="${3:-}"
    local not_match="${4:-}"
    local exclude_tests="${5:-true}"

    if [ ! -d "$path" ]; then
        echo "0 0 0 -"
        return
    fi

    local cloc_args=(--by-file --csv --quiet --include-lang="$lang")
    if [ "$exclude_tests" = "true" ]; then
        cloc_args+=(--exclude-dir=vendor,node_modules,target,dist,tests)
    else
        cloc_args+=(--exclude-dir=vendor,node_modules,target,dist)
    fi

    [ -n "$match_pattern" ] && cloc_args+=(--match-f="$match_pattern")
    [ -n "$not_match" ] && cloc_args+=(--not-match-f="$not_match")

    local output
    # Skip header line, filter out empty lines and SUM row
    output=$(cloc "${cloc_args[@]}" "$path" 2>/dev/null | tail -n +2 | grep -v '^$' | grep -v '^SUM,' || true)

    if [ -z "$output" ]; then
        echo "0 0 0 -"
        return
    fi

    # CSV format: language,filename,blank,comment,code
    local total_loc=0
    local file_count=0
    local max_loc=0
    local max_file="-"
    while IFS=, read -r _ filename _ _ code; do
        # Skip header row (code column would be "code" or contain version string)
        [ -n "$code" ] && [[ ! "$code" =~ ^code ]] && [[ ! "$code" =~ github.com ]] && {
            total_loc=$((total_loc + code))
            file_count=$((file_count + 1))
            if [ "$code" -gt "$max_loc" ]; then
                max_loc=$code
                max_file=$(basename "$filename")
            fi
        }
    done <<< "$output"

    if [ "$file_count" -gt 0 ]; then
        local avg=$((total_loc / file_count))
        echo "$avg $file_count $max_loc $max_file"
    else
        echo "0 0 0 -"
    fi
}

# Compute average and max for test directory (Rust integration tests)
# Returns: avg count max max_file
avg_max_loc_dir() {
    local path="$1"
    local lang="$2"

    if [ ! -d "$path" ]; then
        echo "0 0 0 -"
        return
    fi

    local output
    # Skip header line, filter out empty lines and SUM row
    output=$(cloc --by-file --csv --quiet --include-lang="$lang" "$path" 2>/dev/null | tail -n +2 | grep -v '^$' | grep -v '^SUM,' || true)

    if [ -z "$output" ]; then
        echo "0 0 0 -"
        return
    fi

    local total_loc=0
    local file_count=0
    local max_loc=0
    local max_file="-"
    while IFS=, read -r _ filename _ _ code; do
        # Skip header row (code column would be "code" or contain version string)
        [ -n "$code" ] && [[ ! "$code" =~ ^code ]] && [[ ! "$code" =~ github.com ]] && {
            total_loc=$((total_loc + code))
            file_count=$((file_count + 1))
            if [ "$code" -gt "$max_loc" ]; then
                max_loc=$code
                max_file=$(basename "$filename")
            fi
        }
    done <<< "$output"

    if [ "$file_count" -gt 0 ]; then
        local avg=$((total_loc / file_count))
        echo "$avg $file_count $max_loc $max_file"
    else
        echo "0 0 0 -"
    fi
}

# Check if value exceeds threshold
check_threshold() {
    local value="$1"
    local threshold="$2"
    local label="$3"

    if [ "$value" -eq 0 ]; then
        echo "-"
    elif [ "$value" -gt "$threshold" ]; then
        echo "FAIL ($label > $threshold)"
    else
        echo "ok"
    fi
}

echo "File Size Analysis"
echo "=================="
echo ""
echo "Targets:"
echo "  Source: avg <500 LOC, max <900 LOC"
echo "  Test:   avg <700 LOC, max <1100 LOC"
echo ""

if ! command -v cloc &> /dev/null; then
    echo "Install cloc: brew install cloc"
    exit 1
fi

# Package definitions: name:src_path:test_path
PACKAGES=(
    "crates/cli:$ROOT_DIR/crates/cli/src:$ROOT_DIR/crates/cli/tests"
    "crates/remote:$ROOT_DIR/crates/remote/src:$ROOT_DIR/crates/remote/tests"
    "crates/core:$ROOT_DIR/crates/core/src:"
)

FAILURES=0

echo "=== Per-Package ==="
echo ""

# Track totals
total_src_loc=0
total_src_count=0
total_src_max=0
total_src_max_file="-"
total_test_loc=0
total_test_count=0
total_test_max=0
total_test_max_file="-"

for pkg_def in "${PACKAGES[@]}"; do
    IFS=':' read -r pkg_name src_path test_path <<< "$pkg_def"

    if [ ! -d "$src_path" ]; then
        continue
    fi

    # Source files (exclude *_tests.rs)
    read -r src_avg src_count src_max src_max_file <<< "$(avg_max_loc "$src_path" "Rust" "" "_tests\\.rs$")"

    # Test files (*_tests.rs in src + integration tests)
    test_avg=0
    test_count=0
    test_max=0
    test_max_file="-"
    test_total_loc=0

    # Unit tests in src/
    read -r unit_avg unit_count unit_max unit_max_file <<< "$(avg_max_loc "$src_path" "Rust" "_tests\\.rs$" "" "false")"
    if [ "$unit_count" -gt 0 ]; then
        test_total_loc=$((unit_avg * unit_count))
        test_count=$unit_count
        test_max=$unit_max
        test_max_file=$unit_max_file
    fi

    # Integration tests
    if [ -n "$test_path" ] && [ -d "$test_path" ]; then
        read -r integ_avg integ_count integ_max integ_max_file <<< "$(avg_max_loc_dir "$test_path" "Rust")"
        if [ "$integ_count" -gt 0 ]; then
            test_total_loc=$((test_total_loc + integ_avg * integ_count))
            test_count=$((test_count + integ_count))
            if [ "$integ_max" -gt "$test_max" ]; then
                test_max=$integ_max
                test_max_file=$integ_max_file
            fi
        fi
    fi

    if [ "$test_count" -gt 0 ]; then
        test_avg=$((test_total_loc / test_count))
    fi

    # Check thresholds
    src_avg_status=$(check_threshold "$src_avg" 500 "avg")
    src_max_status=$(check_threshold "$src_max" 900 "max")
    test_avg_status=$(check_threshold "$test_avg" 700 "avg")
    test_max_status=$(check_threshold "$test_max" 1100 "max")

    [[ "$src_avg_status" == FAIL* ]] && FAILURES=$((FAILURES + 1))
    [[ "$src_max_status" == FAIL* ]] && FAILURES=$((FAILURES + 1))
    [[ "$test_avg_status" == FAIL* ]] && FAILURES=$((FAILURES + 1))
    [[ "$test_max_status" == FAIL* ]] && FAILURES=$((FAILURES + 1))

    echo "$pkg_name:"
    printf "  Source: avg %d LOC [%s], max %d LOC (%s) [%s]\n" "$src_avg" "$src_avg_status" "$src_max" "$src_max_file" "$src_max_status"
    printf "  Test:   avg %d LOC [%s], max %d LOC (%s) [%s]\n" "$test_avg" "$test_avg_status" "$test_max" "$test_max_file" "$test_max_status"
    echo ""

    # Accumulate totals
    if [ "$src_count" -gt 0 ]; then
        total_src_loc=$((total_src_loc + src_avg * src_count))
        total_src_count=$((total_src_count + src_count))
        if [ "$src_max" -gt "$total_src_max" ]; then
            total_src_max=$src_max
            total_src_max_file=$src_max_file
        fi
    fi
    if [ "$test_count" -gt 0 ]; then
        total_test_loc=$((total_test_loc + test_total_loc))
        total_test_count=$((total_test_count + test_count))
        if [ "$test_max" -gt "$total_test_max" ]; then
            total_test_max=$test_max
            total_test_max_file=$test_max_file
        fi
    fi
done

# Calculate total averages
total_src_avg=0
total_test_avg=0
if [ "$total_src_count" -gt 0 ]; then
    total_src_avg=$((total_src_loc / total_src_count))
fi
if [ "$total_test_count" -gt 0 ]; then
    total_test_avg=$((total_test_loc / total_test_count))
fi

echo "=== Total ==="
echo ""
echo "Source Files (excluding *_tests.rs):"
printf "  Files:   %d\n" "$total_src_count"
printf "  Average: %d LOC  [%s]\n" "$total_src_avg" "$(check_threshold "$total_src_avg" 500 "avg")"
printf "  Max:     %d LOC (%s)  [%s]\n" "$total_src_max" "$total_src_max_file" "$(check_threshold "$total_src_max" 900 "max")"

echo ""
echo "Test Files (*_tests.rs + tests/):"
printf "  Files:   %d\n" "$total_test_count"
printf "  Average: %d LOC  [%s]\n" "$total_test_avg" "$(check_threshold "$total_test_avg" 700 "avg")"
printf "  Max:     %d LOC (%s)  [%s]\n" "$total_test_max" "$total_test_max_file" "$(check_threshold "$total_test_max" 1100 "max")"

echo ""
if [ "$FAILURES" -gt 0 ]; then
    echo "Result: FAIL ($FAILURES threshold(s) exceeded)"
else
    echo "Result: PASS"
fi
echo ""
echo "Notes:"
echo "- Smaller source files improve LLM context fit"
echo "- Large files may need decomposition"
echo "- Packages: crates/cli, crates/remote, crates/core"
