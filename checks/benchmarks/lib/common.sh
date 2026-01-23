#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/lib/common.sh - Shared utilities for wk benchmarks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WK_BIN="${WK_BIN:-wk}"
RESULTS_DIR="${RESULTS_DIR:-$SCRIPT_DIR/results}"

# ============================================================================
# Output Functions
# ============================================================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[OK]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# ============================================================================
# Database Setup Functions
# ============================================================================

# Setup a test database from a pre-generated SQL file
# Usage: setup_db <size>
# Example: setup_db large
setup_db() {
    local size="$1"
    local sql_file="$SCRIPT_DIR/setup/${size}.sql"

    if [[ ! -f "$sql_file" ]]; then
        error "Database file not found: $sql_file"
        error "Run 'generate_db.sh $size' first"
        return 1
    fi

    rm -rf .wok
    mkdir -p .wok
    sqlite3 .wok/issues.db < "$sql_file"

    cat > .wok/config.toml << EOF
prefix = "bench"
EOF
}

# Restore a database from SQL (same as setup_db but more explicit name)
# Usage: restore_db <size>
restore_db() {
    setup_db "$@"
}

# ============================================================================
# Dependency Checking
# ============================================================================

# Check if required tools are installed
check_dependencies() {
    local missing=()

    if ! command -v hyperfine &> /dev/null; then
        missing+=("hyperfine")
    fi

    if ! command -v jq &> /dev/null; then
        missing+=("jq")
    fi

    if ! command -v bc &> /dev/null; then
        missing+=("bc")
    fi

    if ! command -v sqlite3 &> /dev/null; then
        missing+=("sqlite3")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing required tools: ${missing[*]}"
        echo "Install with:"
        echo "  brew install ${missing[*]}"
        return 1
    fi

    success "All dependencies found"
}

# Verify the wk binary exists and is executable
check_wk_binary() {
    if ! command -v "$WK_BIN" &> /dev/null; then
        error "wk binary not found: $WK_BIN"
        echo "Build with: cargo build --release"
        echo "Then set: WK_BIN=./target/release/wk"
        return 1
    fi

    success "Using wk binary: $(command -v "$WK_BIN")"
}

# ============================================================================
# Results Aggregation
# ============================================================================

# Aggregate all benchmark results into latest.json
# Usage: generate_latest_json
generate_latest_json() {
    local output_file="$RESULTS_DIR/latest.json"
    local temp_file="$RESULTS_DIR/latest.json.tmp"

    info "Generating latest.json..."

    # Remove old latest.json if it exists to avoid including it in the glob
    rm -f "$output_file"

    # Combine all individual result files into one
    jq -s '{ results: [.[].results[]] | unique_by(.command) }' "$RESULTS_DIR"/*.json > "$temp_file"
    mv "$temp_file" "$output_file"

    success "Generated: $output_file"
}
