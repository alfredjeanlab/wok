#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Escape Hatch Detection
# Counts type system escape hatches

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

count_pattern() {
    local pattern="$1"
    local path="$2"
    local include="$3"
    local exclude="${4:-}"

    local count=0
    if [ -d "$path" ]; then
        if [ -n "$exclude" ]; then
            count=$(grep -rn "$pattern" "$path" --include="$include" 2>/dev/null | grep -Ev "$exclude" | wc -l | tr -d ' ')
        else
            count=$(grep -rn "$pattern" "$path" --include="$include" 2>/dev/null | wc -l | tr -d ' ')
        fi
    fi
    echo "$count"
}

show_matches() {
    local pattern="$1"
    local path="$2"
    local include="$3"
    local exclude="${4:-}"
    local limit="${5:-5}"

    if [ -d "$path" ]; then
        if [ -n "$exclude" ]; then
            grep -rn "$pattern" "$path" --include="$include" 2>/dev/null | grep -Ev "$exclude" | head -"$limit" || true
        else
            grep -rn "$pattern" "$path" --include="$include" 2>/dev/null | head -"$limit" || true
        fi
    fi
}

echo "Escape Hatch Analysis"
echo "====================="
echo ""
echo "Escape hatches bypass type safety. Lower counts are better."
echo ""

# Package definitions: name:src_path
PACKAGES=(
    "crates/cli:$ROOT_DIR/crates/cli/src"
    "crates/remote:$ROOT_DIR/crates/remote/src"
    "crates/core:$ROOT_DIR/crates/core/src"
)

# Exclude test files (_tests.rs) and test utilities (testing.rs) from source counts
TEST_EXCLUDE="_tests\.rs|/testing\.rs"

# Pattern for finding mem::transmute (stored as variable to avoid false positives in linters)
# SAFETY: This is a search pattern string, not actual transmute usage
TRANSMUTE_PATTERN='mem::transmute'

echo "=== Per-Package ==="
echo ""

# Track totals
total_unsafe=0
total_unwrap=0
total_expect=0
total_cast=0
total_transmute=0

for pkg_def in "${PACKAGES[@]}"; do
    IFS=':' read -r pkg_name src_path <<< "$pkg_def"

    if [ ! -d "$src_path" ]; then
        continue
    fi

    unsafe_count=$(count_pattern 'unsafe\s*{' "$src_path" "*.rs" "$TEST_EXCLUDE")
    unwrap_count=$(count_pattern '\.unwrap()' "$src_path" "*.rs" "$TEST_EXCLUDE")
    expect_count=$(count_pattern '\.expect(' "$src_path" "*.rs" "$TEST_EXCLUDE")
    cast_count=$(count_pattern ' as [ui]' "$src_path" "*.rs" "$TEST_EXCLUDE")
    transmute_count=$(count_pattern "$TRANSMUTE_PATTERN" "$src_path" "*.rs" "$TEST_EXCLUDE")

    high_risk=$((unsafe_count + unwrap_count + expect_count + transmute_count))

    echo "$pkg_name:"
    echo "  unsafe: $unsafe_count, unwrap: $unwrap_count, expect: $expect_count, transmute: $transmute_count"
    echo "  as casts: $cast_count"
    echo "  High-risk total: $high_risk"
    echo ""

    # Accumulate totals
    total_unsafe=$((total_unsafe + unsafe_count))
    total_unwrap=$((total_unwrap + unwrap_count))
    total_expect=$((total_expect + expect_count))
    total_cast=$((total_cast + cast_count))
    total_transmute=$((total_transmute + transmute_count))
done

total_high_risk=$((total_unsafe + total_unwrap + total_expect + total_transmute))

echo "=== Total ==="
echo ""
echo "unsafe blocks: $total_unsafe"
if [ "$total_unsafe" -gt 0 ]; then
    for pkg_def in "${PACKAGES[@]}"; do
        IFS=':' read -r _ src_path <<< "$pkg_def"
        [ -d "$src_path" ] && show_matches 'unsafe\s*{' "$src_path" "*.rs" "$TEST_EXCLUDE" 3
    done
fi

echo ""
echo ".unwrap() calls: $total_unwrap"

echo ""
echo ".expect() calls: $total_expect"

echo ""
echo "as casts (numeric): $total_cast"

echo ""
echo "$TRANSMUTE_PATTERN: $total_transmute"
if [ "$total_transmute" -gt 0 ]; then
    for pkg_def in "${PACKAGES[@]}"; do
        IFS=':' read -r _ src_path <<< "$pkg_def"
        [ -d "$src_path" ] && show_matches "$TRANSMUTE_PATTERN" "$src_path" "*.rs" "$TEST_EXCLUDE" 3
    done
fi

echo ""
echo "Total high-risk escapes: $total_high_risk (unsafe + unwrap + expect + transmute)"
echo ""

# Check for #[allow(...)] attributes in source files
echo "=== Lint Suppression Attributes ==="
echo ""

total_allow=0
for pkg_def in "${PACKAGES[@]}"; do
    IFS=':' read -r pkg_name src_path <<< "$pkg_def"

    if [ ! -d "$src_path" ]; then
        continue
    fi

    # Count #[allow(...)] and #![allow(...)] in source files (not test files)
    allow_count=$(count_pattern '#\[allow\|#!\[allow' "$src_path" "*.rs" "$TEST_EXCLUDE")
    total_allow=$((total_allow + allow_count))

    if [ "$allow_count" -gt 0 ]; then
        echo "$pkg_name: $allow_count"
        show_matches '#\[allow\|#!\[allow' "$src_path" "*.rs" "$TEST_EXCLUDE" 5
        echo ""
    fi
done

if [ "$total_allow" -eq 0 ]; then
    echo "No lint suppression attributes in source files (good!)"
else
    echo "Total #[allow(...)] in source: $total_allow"
    echo "  Note: These should have justifying comments or be removed"
fi
echo ""

echo "Legend:"
echo "  High risk: unsafe, $TRANSMUTE_PATTERN"
echo "  Medium risk: unwrap, expect (panics on None/Err)"
echo "  Low risk: #[allow(...)] (lint suppression)"
echo "  Context matters: unwrap/expect/allow in tests is acceptable"
echo "  Packages: crates/cli, crates/remote, crates/core"
