#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Lines of Code measurement
# Counts source and test code separately using cloc

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "Lines of Code Analysis"
echo "======================"
echo ""

# Package definitions: name:src_path:test_path
PACKAGES=(
    "crates/cli:$ROOT_DIR/crates/cli/src:$ROOT_DIR/crates/cli/tests"
    "crates/remote:$ROOT_DIR/crates/remote/src:$ROOT_DIR/crates/remote/tests"
    "crates/core:$ROOT_DIR/crates/core/src:"
)

if ! command -v cloc &> /dev/null; then
    echo "Install cloc: brew install cloc"
    exit 1
fi

# Count LOC for a set of files
count_loc() {
    local tmpfile="$1"
    if [ -s "$tmpfile" ]; then
        # CSV format: files,language,blank,comment,code
        local result=$(cloc --list-file="$tmpfile" --csv --quiet 2>/dev/null | grep ",Rust," | cut -d',' -f5)
        echo "${result:-0}"
    else
        echo "0"
    fi
}

# Count files
count_files() {
    local tmpfile="$1"
    if [ -s "$tmpfile" ]; then
        wc -l < "$tmpfile" | tr -d ' '
    else
        echo "0"
    fi
}

echo "=== Per-Package ==="
echo ""

total_src_loc=0
total_src_files=0
total_test_loc=0
total_test_files=0

for pkg_def in "${PACKAGES[@]}"; do
    IFS=':' read -r pkg_name src_path test_path <<< "$pkg_def"

    if [ ! -d "$src_path" ]; then
        continue
    fi

    # Source files (excluding *_tests.rs)
    tmpfile_src=$(mktemp)
    find "$src_path" -type f -name "*.rs" ! -name "*_tests.rs" >> "$tmpfile_src" 2>/dev/null || true
    src_loc=$(count_loc "$tmpfile_src")
    src_files=$(count_files "$tmpfile_src")
    rm -f "$tmpfile_src"

    # Test files (*_tests.rs in src + integration tests)
    tmpfile_test=$(mktemp)
    find "$src_path" -type f -name "*_tests.rs" >> "$tmpfile_test" 2>/dev/null || true
    [ -n "$test_path" ] && [ -d "$test_path" ] && find "$test_path" -type f -name "*.rs" >> "$tmpfile_test" 2>/dev/null || true
    test_loc=$(count_loc "$tmpfile_test")
    test_files=$(count_files "$tmpfile_test")
    rm -f "$tmpfile_test"

    echo "$pkg_name:"
    echo "  Source: $src_files files, $src_loc LOC"
    echo "  Test:   $test_files files, $test_loc LOC"
    echo ""

    total_src_loc=$((total_src_loc + src_loc))
    total_src_files=$((total_src_files + src_files))
    total_test_loc=$((total_test_loc + test_loc))
    total_test_files=$((total_test_files + test_files))
done

echo "=== Total ==="
echo ""
echo "Source code (excluding *_tests.rs):"
echo "  Files: $total_src_files"
echo "  LOC:   $total_src_loc"
echo ""
echo "Test code:"
echo "  Files: $total_test_files"
echo "  LOC:   $total_test_loc"
echo ""
echo "Notes:"
echo "- Excludes target/ directory"
echo "- Unit tests are in sibling *_tests.rs files"
echo "- Packages: crates/cli, crates/remote, crates/core"
