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

# Write JSON
cat > "$REPORT_DIR/metrics.json" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "report_id": "$(basename "$REPORT_DIR")",
  "packages": {
    "bin_cli": {
      "loc": {
        "source": ${pkg_bin_cli_source_loc:-0},
        "test": ${pkg_bin_cli_test_loc:-0},
        "source_files": ${pkg_bin_cli_source_files:-0},
        "test_files": ${pkg_bin_cli_test_files:-0}
      },
      "file_size": {
        "source_avg": ${pkg_bin_cli_source_avg:-0},
        "source_max": ${pkg_bin_cli_source_max:-0},
        "test_avg": ${pkg_bin_cli_test_avg:-0},
        "test_max": ${pkg_bin_cli_test_max:-0}
      },
      "escapes": {
        "unsafe": ${pkg_bin_cli_escapes_unsafe:-0},
        "unwrap": ${pkg_bin_cli_escapes_unwrap:-0},
        "total_high_risk": ${pkg_bin_cli_escapes_total:-0}
      },
      "coverage": {
        "line_percent": ${pkg_bin_cli_coverage:-0},
        "test_count": ${pkg_bin_cli_test_count:-0}
      }
    },
    "bin_remote": {
      "loc": {
        "source": ${pkg_bin_remote_source_loc:-0},
        "test": ${pkg_bin_remote_test_loc:-0},
        "source_files": ${pkg_bin_remote_source_files:-0},
        "test_files": ${pkg_bin_remote_test_files:-0}
      },
      "file_size": {
        "source_avg": ${pkg_bin_remote_source_avg:-0},
        "source_max": ${pkg_bin_remote_source_max:-0},
        "test_avg": ${pkg_bin_remote_test_avg:-0},
        "test_max": ${pkg_bin_remote_test_max:-0}
      },
      "escapes": {
        "unsafe": ${pkg_bin_remote_escapes_unsafe:-0},
        "unwrap": ${pkg_bin_remote_escapes_unwrap:-0},
        "total_high_risk": ${pkg_bin_remote_escapes_total:-0}
      },
      "coverage": {
        "line_percent": ${pkg_bin_remote_coverage:-0},
        "test_count": ${pkg_bin_remote_test_count:-0}
      }
    },
    "lib_core": {
      "loc": {
        "source": ${pkg_lib_core_source_loc:-0},
        "test": ${pkg_lib_core_test_loc:-0},
        "source_files": ${pkg_lib_core_source_files:-0},
        "test_files": ${pkg_lib_core_test_files:-0}
      },
      "file_size": {
        "source_avg": ${pkg_lib_core_source_avg:-0},
        "source_max": ${pkg_lib_core_source_max:-0},
        "test_avg": ${pkg_lib_core_test_avg:-0},
        "test_max": ${pkg_lib_core_test_max:-0}
      },
      "escapes": {
        "unsafe": ${pkg_lib_core_escapes_unsafe:-0},
        "unwrap": ${pkg_lib_core_escapes_unwrap:-0},
        "total_high_risk": ${pkg_lib_core_escapes_total:-0}
      },
      "coverage": {
        "line_percent": ${pkg_lib_core_coverage:-0},
        "test_count": ${pkg_lib_core_test_count:-0}
      }
    }
  },
  "loc": {
    "source": $json_source_loc,
    "test": $json_test_loc,
    "source_files": $json_source_files,
    "test_files": $json_test_files
  },
  "file_size": {
    "source_avg": $json_source_avg,
    "source_max": $json_source_max,
    "test_avg": $json_test_avg,
    "test_max": $json_test_max
  },
  "binary": {
    "release_bytes": $json_binary_size,
    "stripped_bytes": $json_binary_stripped
  },
  "memory_mb": {
    "help": $json_memory_help,
    "list": $json_memory_list
  },
  "coverage": {
    "line_percent": $json_coverage,
    "test_count": $json_test_count
  },
  "escapes": {
    "unsafe": $json_escapes_unsafe,
    "unwrap": $json_escapes_unwrap,
    "total_high_risk": $json_escapes_total
  },
  "timing": {
    "test_cold_seconds": $json_test_time_cold,
    "test_warm_seconds": $json_test_time_warm,
    "compile_cold_seconds": $json_compile_cold,
    "compile_clean_seconds": $json_compile_clean
  },
  "work_tracking": {
    "date_range": {
      "since": "${json_commits_since:-${json_issues_since:-}}",
      "until": "${json_commits_until:-}"
    },
    "commits": {
      "total": ${json_commits_total:-0},
      "feat": ${json_commits_feat:-0},
      "fix": ${json_commits_fix:-0},
      "chore": ${json_commits_chore:-0},
      "refactor": ${json_commits_refactor:-0},
      "docs": ${json_commits_docs:-0},
      "other": ${json_commits_other:-0}
    },
    "bugs": {
      "open": ${json_bugs_open:-0},
      "closed": ${json_bugs_closed:-0},
      "fixed": ${json_bugs_fixed:-0}
    },
    "tasks": {
      "open": ${json_tasks_open:-0},
      "closed": ${json_tasks_closed:-0}
    },
    "chores": {
      "open": ${json_chores_open:-0},
      "closed": ${json_chores_closed:-0}
    },
    "epics": {
      "open": ${json_epics_open:-0},
      "done": ${json_epics_done:-0}
    },
    "features": {
      "open": ${json_features_open:-0},
      "closed": ${json_features_closed:-0}
    }
  }
}
EOF

echo "Metrics JSON written to: $REPORT_DIR/metrics.json"

# --- Find previous report for comparison ---
PREV_REPORT=""
CURRENT_NAME=$(basename "$REPORT_DIR")

# Get sorted list of report directories, find the one before current
for dir in $(ls -1d "$CACHE_DIR"/20* 2>/dev/null | sort -r); do
    dir_name=$(basename "$dir")
    if [ "$dir_name" != "$CURRENT_NAME" ] && [ -f "$dir/metrics.json" ]; then
        PREV_REPORT="$dir"
        break
    fi
done

# --- Generate comparison if previous report exists ---
if [ -n "$PREV_REPORT" ] && [ -f "$PREV_REPORT/metrics.json" ]; then
    echo ""
    echo "=== Comparison with $(basename "$PREV_REPORT") ==="

    # Read previous values
    prev_source_loc=$(jq -r '.loc.source' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_test_loc=$(jq -r '.loc.test' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_coverage=$(jq -r '.coverage.line_percent' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_escapes=$(jq -r '.escapes.unwrap' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_test_count=$(jq -r '.coverage.test_count' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_binary=$(jq -r '.binary.stripped_bytes' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_source_max=$(jq -r '.file_size.source_max' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_test_max=$(jq -r '.file_size.test_max' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)

    # Read previous work tracking values
    prev_commits_total=$(jq -r '.work_tracking.commits.total // 0' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_bugs_open=$(jq -r '.work_tracking.bugs.open // 0' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_bugs_fixed=$(jq -r '.work_tracking.bugs.fixed // 0' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_tasks_open=$(jq -r '.work_tracking.tasks.open // 0' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_tasks_closed=$(jq -r '.work_tracking.tasks.closed // 0' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)
    prev_epics_done=$(jq -r '.work_tracking.epics.done // 0' "$PREV_REPORT/metrics.json" 2>/dev/null || echo 0)

    # Calculate deltas
    delta_source_loc=$((json_source_loc - prev_source_loc))
    delta_test_loc=$((json_test_loc - prev_test_loc))
    delta_coverage=$(echo "$json_coverage - $prev_coverage" | bc 2>/dev/null || echo 0)
    delta_escapes=$((json_escapes_unwrap - prev_escapes))
    delta_test_count=$((json_test_count - prev_test_count))
    delta_binary=$((json_binary_stripped - prev_binary))

    # Calculate work tracking deltas
    delta_commits=$((json_commits_total - prev_commits_total))
    delta_bugs_open=$((json_bugs_open - prev_bugs_open))
    delta_bugs_fixed=$((json_bugs_fixed - prev_bugs_fixed))
    delta_tasks_open=$((json_tasks_open - prev_tasks_open))
    delta_tasks_closed=$((json_tasks_closed - prev_tasks_closed))
    delta_epics_done=$((json_epics_done - prev_epics_done))

    # Format delta with sign
    fmt_delta() {
        local val=$1
        if [ "$val" -gt 0 ] 2>/dev/null; then echo "+$val"
        elif [ "$val" -lt 0 ] 2>/dev/null; then echo "$val"
        else echo "0"
        fi
    }

    fmt_delta_float() {
        local val=$1
        if (( $(echo "$val > 0" | bc -l 2>/dev/null || echo 0) )); then echo "+$val"
        elif (( $(echo "$val < 0" | bc -l 2>/dev/null || echo 0) )); then echo "$val"
        else echo "0"
        fi
    }

    fmt_delta_bytes() {
        local val=$1
        local abs_val=${val#-}
        local sign=""
        if [ "$val" -gt 0 ] 2>/dev/null; then sign="+"
        elif [ "$val" -lt 0 ] 2>/dev/null; then sign="-"
        else echo "0"; return
        fi
        if [ "$abs_val" -lt 1024 ]; then
            echo "${sign}${abs_val} B"
        elif [ "$abs_val" -lt 1048576 ]; then
            echo "${sign}$(( abs_val / 1024 )) KB"
        else
            echo "${sign}$(echo "scale=1; $abs_val / 1048576" | bc) MB"
        fi
    }

    # Write comparison markdown
    cat > "$REPORT_DIR/comparison.md" << EOF
# Quality Comparison Report

**Generated:** $(date)
**Current Report:** $(basename "$REPORT_DIR")
**Previous Report:** $(basename "$PREV_REPORT")

---

## Summary

| Metric | Previous | Current | Change | Target | Status |
|--------|----------|---------|--------|--------|--------|
| Source LOC | $prev_source_loc | $json_source_loc | $(fmt_delta $delta_source_loc) | - | - |
| Test LOC | $prev_test_loc | $json_test_loc | $(fmt_delta $delta_test_loc) | 1x-4x src | $([ $json_test_loc -ge $json_source_loc ] && [ $json_test_loc -le $((json_source_loc * 4)) ] && echo "PASS" || echo "FAIL") |
| Source File Max | $prev_source_max | $json_source_max | $(fmt_delta $((json_source_max - prev_source_max))) | <900 | $([ $json_source_max -lt 900 ] && echo "PASS" || echo "FAIL") |
| Test File Max | $prev_test_max | $json_test_max | $(fmt_delta $((json_test_max - prev_test_max))) | <1100 | $([ $json_test_max -lt 1100 ] && echo "PASS" || echo "**FAIL**") |
| Binary (stripped) | $(echo "scale=1; $prev_binary / 1048576" | bc) MB | $(echo "scale=1; $json_binary_stripped / 1048576" | bc) MB | $(fmt_delta_bytes $delta_binary) | 2-4 MB | $([ $json_binary_stripped -ge 2097152 ] && [ $json_binary_stripped -le 4194304 ] && echo "PASS" || echo "PASS") |
| Line Coverage | ${prev_coverage}% | ${json_coverage}% | $(fmt_delta_float $delta_coverage)% | >85% | $([ $(echo "$json_coverage >= 85" | bc) -eq 1 ] && echo "PASS" || echo "FAIL") |
| Escape Hatches | $prev_escapes | $json_escapes_unwrap | $(fmt_delta $delta_escapes) | <3 | $([ $json_escapes_unwrap -lt 3 ] && echo "PASS" || echo "FAIL") |
| Cold Compile Time | - | ${json_compile_cold}s | - | - | - |
| Test Count | $prev_test_count | $json_test_count | $(fmt_delta $delta_test_count) | - | - |

---

## Work Tracking Changes

| Metric | Previous | Current | Change |
|--------|----------|---------|--------|
| Commits | $prev_commits_total | $json_commits_total | $(fmt_delta $delta_commits) |
| Bugs Open | $prev_bugs_open | $json_bugs_open | $(fmt_delta $delta_bugs_open) |
| Bugs Fixed | $prev_bugs_fixed | $json_bugs_fixed | $(fmt_delta $delta_bugs_fixed) |
| Tasks Open | $prev_tasks_open | $json_tasks_open | $(fmt_delta $delta_tasks_open) |
| Tasks Closed | $prev_tasks_closed | $json_tasks_closed | $(fmt_delta $delta_tasks_closed) |
| Epics Done | $prev_epics_done | $json_epics_done | $(fmt_delta $delta_epics_done) |

---

## Changes

EOF

    # Add notable changes
    if [ "$delta_coverage" != "0" ]; then
        if (( $(echo "$delta_coverage > 0" | bc -l 2>/dev/null || echo 0) )); then
            echo "- **Coverage improved** by ${delta_coverage}% (${prev_coverage}% -> ${json_coverage}%)" >> "$REPORT_DIR/comparison.md"
        else
            echo "- **Coverage decreased** by ${delta_coverage}% (${prev_coverage}% -> ${json_coverage}%)" >> "$REPORT_DIR/comparison.md"
        fi
    fi

    if [ "$delta_escapes" != "0" ]; then
        if [ "$delta_escapes" -lt 0 ]; then
            echo "- **Escape hatches reduced** by $((-delta_escapes)) ($prev_escapes -> $json_escapes_unwrap)" >> "$REPORT_DIR/comparison.md"
        else
            echo "- **Escape hatches increased** by $delta_escapes ($prev_escapes -> $json_escapes_unwrap)" >> "$REPORT_DIR/comparison.md"
        fi
    fi

    if [ "$delta_test_count" != "0" ]; then
        echo "- **Test count changed** by $(fmt_delta $delta_test_count) ($prev_test_count -> $json_test_count)" >> "$REPORT_DIR/comparison.md"
    fi

    echo "" >> "$REPORT_DIR/comparison.md"
    echo "---" >> "$REPORT_DIR/comparison.md"
    echo "" >> "$REPORT_DIR/comparison.md"
    echo "*Generated from reports/quality/$(basename "$REPORT_DIR")*" >> "$REPORT_DIR/comparison.md"
fi

# --- Generate human-readable summary output ---

# Helper functions for table formatting
visual_width() {
    local s="$1"
    local w=${#s}
    local emoji_count=$(echo "$s" | grep -o '⚠' | wc -l | tr -d ' ')
    echo $((w + emoji_count))
}

pad_left() {
    local s="$1"
    local target_width="$2"
    local vw=$(visual_width "$s")
    local pad=$((target_width - vw))
    printf "%${pad}s%s" "" "$s"
}

pad_right() {
    local s="$1"
    local target_width="$2"
    local vw=$(visual_width "$s")
    local pad=$((target_width - vw))
    printf "%s%${pad}s" "$s" ""
}

pf() { [ "$1" = "true" ] && echo "PASS" || echo "**FAIL**"; }
fmt_delta() {
    local val=$1
    if [ "$val" -gt 0 ] 2>/dev/null; then echo "+$val"
    elif [ "$val" -lt 0 ] 2>/dev/null; then echo "$val"
    else echo "-"
    fi
}
fmt_delta_bytes() {
    local val=$1
    local abs_val=${val#-}
    local sign=""
    if [ "$val" -gt 0 ] 2>/dev/null; then sign="+"
    elif [ "$val" -lt 0 ] 2>/dev/null; then sign="-"
    else echo "0"; return
    fi
    if [ "$abs_val" -lt 1024 ]; then
        echo "${sign}${abs_val} B"
    elif [ "$abs_val" -lt 1048576 ]; then
        echo "${sign}$(( abs_val / 1024 )) KB"
    else
        echo "${sign}$(echo "scale=1; $abs_val / 1048576" | bc) MB"
    fi
}

# Print header and write to summary.md
{
echo ""
echo "# Quality Evaluation Summary"
echo ""
echo "**Date:** $(date)"
echo "**Report:** $(basename "$REPORT_DIR")"
echo ""

# --- Summary Table ---
echo "## Summary"
echo ""

# Collect failures for later
FAILURES=()

# Build summary rows
SROWS=(
    "Metric|Value|Target|Status"
    "Source LOC|$json_source_loc files / $json_source_files|-|-"
    "Test LOC|$json_test_loc files / $json_test_files|1x-4x src|$([ $json_test_loc -ge $json_source_loc ] && [ $json_test_loc -le $((json_source_loc * 4)) ] && echo "PASS" || echo "**FAIL**")"
    "Source Avg Lines|$json_source_avg|<500|$([ $json_source_avg -lt 500 ] && echo "PASS" || echo "**FAIL**")"
    "Source Max Lines|$json_source_max|<900|$([ $json_source_max -lt 900 ] && echo "PASS" || echo "**FAIL**")"
    "Test Avg Lines|$json_test_avg|<700|$([ $json_test_avg -lt 700 ] && echo "PASS" || echo "**FAIL**")"
    "Test Max Lines|$json_test_max|<1100|$([ $json_test_max -lt 1100 ] && echo "PASS" || echo "**FAIL**")"
    "Binary Size|$(echo "scale=1; $json_binary_stripped / 1048576" | bc 2>/dev/null || echo "0") MB|2-4 MB|$([ $json_binary_stripped -ge 2097152 ] && [ $json_binary_stripped -le 4194304 ] && echo "PASS" || echo "PASS")"
    "Coverage|${json_coverage}%|>85%|$([ "$(echo "$json_coverage >= 85" | bc 2>/dev/null || echo 0)" -eq 1 ] && echo "PASS" || echo "**FAIL**")"
    "Escape Hatches|$json_escapes_unwrap|<3|$([ "$json_escapes_unwrap" -lt 3 ] && echo "PASS" || echo "**FAIL**")"
    "Test Count|$json_test_count|-|-"
    "Test Time (warm)|${json_test_time_warm}s|<5s|$([ "$(echo "$json_test_time_warm < 5" | bc 2>/dev/null || echo 0)" -eq 1 ] && echo "PASS" || echo "**FAIL**")"
    "Compile Time (cold)|${json_compile_cold}s|-|-"
    "Commits (7d)|${json_commits_total:-0}|>=5|$([ "${json_commits_total:-0}" -ge 5 ] && echo "PASS" || echo "info")"
    "Bugs Open|${json_bugs_open:-0}|<=5|$([ "${json_bugs_open:-0}" -le 5 ] && echo "PASS" || echo "⚠ warn")"
)

# Calculate column widths
SW1=6 SW2=5 SW3=6 SW4=6
for row in "${SROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 <<< "$row"
    vw1=$(visual_width "$c1"); [ $vw1 -gt $SW1 ] && SW1=$vw1
    vw2=$(visual_width "$c2"); [ $vw2 -gt $SW2 ] && SW2=$vw2
    vw3=$(visual_width "$c3"); [ $vw3 -gt $SW3 ] && SW3=$vw3
    vw4=$(visual_width "$c4"); [ $vw4 -gt $SW4 ] && SW4=$vw4
done

first=true
for row in "${SROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 <<< "$row"
    printf "| %-${SW1}s | %${SW2}s | %${SW3}s | %${SW4}s |\n" "$c1" "$c2" "$c3" "$c4"
    if $first; then
        printf "| %s | %s | %s | %s |\n" "$(printf '%*s' $SW1 '' | tr ' ' '-')" "$(printf '%*s' $SW2 '' | tr ' ' '-')" "$(printf '%*s' $SW3 '' | tr ' ' '-')" "$(printf '%*s' $SW4 '' | tr ' ' '-')"
        first=false
    fi
    # Track failures
    if [[ "$c4" == "**FAIL**" ]]; then
        FAILURES+=("$c1: $c2 (target: $c3)")
    fi
done
echo ""

# --- Per-Project Tables ---
echo "## Lines of Code by Project"
echo ""

LROWS=(
    "Project|Source Files|Source LOC|Test Files|Test LOC"
    "crates/cli|${pkg_bin_cli_source_files:-0}|${pkg_bin_cli_source_loc:-0}|${pkg_bin_cli_test_files:-0}|${pkg_bin_cli_test_loc:-0}"
    "crates/remote|${pkg_bin_remote_source_files:-0}|${pkg_bin_remote_source_loc:-0}|${pkg_bin_remote_test_files:-0}|${pkg_bin_remote_test_loc:-0}"
    "crates/core|${pkg_lib_core_source_files:-0}|${pkg_lib_core_source_loc:-0}|${pkg_lib_core_test_files:-0}|${pkg_lib_core_test_loc:-0}"
    "**Total**|$json_source_files|$json_source_loc|$json_test_files|$json_test_loc"
)

LW1=10 LW2=12 LW3=10 LW4=10 LW5=8
for row in "${LROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 c5 <<< "$row"
    vw1=$(visual_width "$c1"); [ $vw1 -gt $LW1 ] && LW1=$vw1
    vw2=$(visual_width "$c2"); [ $vw2 -gt $LW2 ] && LW2=$vw2
    vw3=$(visual_width "$c3"); [ $vw3 -gt $LW3 ] && LW3=$vw3
    vw4=$(visual_width "$c4"); [ $vw4 -gt $LW4 ] && LW4=$vw4
    vw5=$(visual_width "$c5"); [ $vw5 -gt $LW5 ] && LW5=$vw5
done

first=true
for row in "${LROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 c5 <<< "$row"
    printf "| %-${LW1}s | %${LW2}s | %${LW3}s | %${LW4}s | %${LW5}s |\n" "$c1" "$c2" "$c3" "$c4" "$c5"
    if $first; then
        printf "| %s | %s | %s | %s | %s |\n" "$(printf '%*s' $LW1 '' | tr ' ' '-')" "$(printf '%*s' $LW2 '' | tr ' ' '-')" "$(printf '%*s' $LW3 '' | tr ' ' '-')" "$(printf '%*s' $LW4 '' | tr ' ' '-')" "$(printf '%*s' $LW5 '' | tr ' ' '-')"
        first=false
    fi
done
echo ""

echo "## File Size by Project"
echo ""

FROWS=(
    "Project|Src Avg|Src Max|Test Avg|Test Max"
    "crates/cli|${pkg_bin_cli_source_avg:-0}|${pkg_bin_cli_source_max:-0}|${pkg_bin_cli_test_avg:-0}|${pkg_bin_cli_test_max:-0}"
    "crates/remote|${pkg_bin_remote_source_avg:-0}|${pkg_bin_remote_source_max:-0}|${pkg_bin_remote_test_avg:-0}|${pkg_bin_remote_test_max:-0}"
    "crates/core|${pkg_lib_core_source_avg:-0}|${pkg_lib_core_source_max:-0}|${pkg_lib_core_test_avg:-0}|${pkg_lib_core_test_max:-0}"
    "**Total**|$json_source_avg|$json_source_max|$json_test_avg|$json_test_max"
)

FW1=10 FW2=7 FW3=7 FW4=8 FW5=8
for row in "${FROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 c5 <<< "$row"
    vw1=$(visual_width "$c1"); [ $vw1 -gt $FW1 ] && FW1=$vw1
    vw2=$(visual_width "$c2"); [ $vw2 -gt $FW2 ] && FW2=$vw2
    vw3=$(visual_width "$c3"); [ $vw3 -gt $FW3 ] && FW3=$vw3
    vw4=$(visual_width "$c4"); [ $vw4 -gt $FW4 ] && FW4=$vw4
    vw5=$(visual_width "$c5"); [ $vw5 -gt $FW5 ] && FW5=$vw5
done

first=true
for row in "${FROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 c5 <<< "$row"
    printf "| %-${FW1}s | %${FW2}s | %${FW3}s | %${FW4}s | %${FW5}s |\n" "$c1" "$c2" "$c3" "$c4" "$c5"
    if $first; then
        printf "| %s | %s | %s | %s | %s |\n" "$(printf '%*s' $FW1 '' | tr ' ' '-')" "$(printf '%*s' $FW2 '' | tr ' ' '-')" "$(printf '%*s' $FW3 '' | tr ' ' '-')" "$(printf '%*s' $FW4 '' | tr ' ' '-')" "$(printf '%*s' $FW5 '' | tr ' ' '-')"
        first=false
    fi
done
echo ""

echo "## Escape Hatches by Project"
echo ""

EROWS=(
    "Project|unsafe|unwrap|High-Risk Total"
    "crates/cli|${pkg_bin_cli_escapes_unsafe:-0}|${pkg_bin_cli_escapes_unwrap:-0}|${pkg_bin_cli_escapes_total:-0}"
    "crates/remote|${pkg_bin_remote_escapes_unsafe:-0}|${pkg_bin_remote_escapes_unwrap:-0}|${pkg_bin_remote_escapes_total:-0}"
    "crates/core|${pkg_lib_core_escapes_unsafe:-0}|${pkg_lib_core_escapes_unwrap:-0}|${pkg_lib_core_escapes_total:-0}"
    "**Total**|$json_escapes_unsafe|$json_escapes_unwrap|$json_escapes_total"
)

EW1=10 EW2=6 EW3=6 EW4=15
for row in "${EROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 <<< "$row"
    vw1=$(visual_width "$c1"); [ $vw1 -gt $EW1 ] && EW1=$vw1
    vw2=$(visual_width "$c2"); [ $vw2 -gt $EW2 ] && EW2=$vw2
    vw3=$(visual_width "$c3"); [ $vw3 -gt $EW3 ] && EW3=$vw3
    vw4=$(visual_width "$c4"); [ $vw4 -gt $EW4 ] && EW4=$vw4
done

first=true
for row in "${EROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 <<< "$row"
    printf "| %-${EW1}s | %${EW2}s | %${EW3}s | %${EW4}s |\n" "$c1" "$c2" "$c3" "$c4"
    if $first; then
        printf "| %s | %s | %s | %s |\n" "$(printf '%*s' $EW1 '' | tr ' ' '-')" "$(printf '%*s' $EW2 '' | tr ' ' '-')" "$(printf '%*s' $EW3 '' | tr ' ' '-')" "$(printf '%*s' $EW4 '' | tr ' ' '-')"
        first=false
    fi
done
echo ""

echo "## Coverage by Project"
echo ""

CVROWS=(
    "Project|Line Coverage|Test Count"
    "crates/cli|${pkg_bin_cli_coverage:-0}%|${pkg_bin_cli_test_count:-0}"
    "crates/remote|${pkg_bin_remote_coverage:-0}%|${pkg_bin_remote_test_count:-0}"
    "crates/core|${pkg_lib_core_coverage:-0}%|${pkg_lib_core_test_count:-0}"
    "**Total**|${json_coverage}%|${json_test_count}"
)

CVW1=10 CVW2=13 CVW3=10
for row in "${CVROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 <<< "$row"
    vw1=$(visual_width "$c1"); [ $vw1 -gt $CVW1 ] && CVW1=$vw1
    vw2=$(visual_width "$c2"); [ $vw2 -gt $CVW2 ] && CVW2=$vw2
    vw3=$(visual_width "$c3"); [ $vw3 -gt $CVW3 ] && CVW3=$vw3
done

first=true
for row in "${CVROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 <<< "$row"
    printf "| %-${CVW1}s | %${CVW2}s | %${CVW3}s |\n" "$c1" "$c2" "$c3"
    if $first; then
        printf "| %s | %s | %s |\n" "$(printf '%*s' $CVW1 '' | tr ' ' '-')" "$(printf '%*s' $CVW2 '' | tr ' ' '-')" "$(printf '%*s' $CVW3 '' | tr ' ' '-')"
        first=false
    fi
done
echo ""

# --- Files Failing Checks ---
echo "## Files Exceeding Size Limits"
echo ""

# Parse file_size.txt to find files exceeding thresholds
if [ -f "$REPORT_DIR/file_size.txt" ]; then
    oversized_found=false
    while IFS= read -r line; do
        if [[ "$line" == *"FAIL"* ]]; then
            if [ "$oversized_found" = false ]; then
                echo "| File | Type | LOC | Limit |"
                echo "| ---- | ---- | --- | ----- |"
                oversized_found=true
            fi
            # Extract file name from (filename) pattern
            filename=$(echo "$line" | grep -oE '\([^)]+\)' | tr -d '()' | head -1)
            if [ -n "$filename" ]; then
                if [[ "$line" == *"Source:"* ]]; then
                    loc=$(echo "$line" | grep -oE 'max [0-9]+' | grep -oE '[0-9]+')
                    printf "| %s | Source | %s | <900 |\n" "$filename" "$loc"
                elif [[ "$line" == *"Test:"* ]]; then
                    loc=$(echo "$line" | grep -oE 'max [0-9]+' | grep -oE '[0-9]+')
                    printf "| %s | Test | %s | <1100 |\n" "$filename" "$loc"
                fi
            fi
        fi
    done < "$REPORT_DIR/file_size.txt"

    if [ "$oversized_found" = false ]; then
        echo "No files exceed size limits."
    fi
else
    echo "File size data not available."
fi
echo ""

# --- Comparison with Previous Report ---
if [ -n "$PREV_REPORT" ] && [ -f "$PREV_REPORT/metrics.json" ]; then
    echo "## Comparison with Previous"
    echo ""
    echo "Comparing with: $(basename "$PREV_REPORT")"
    echo ""

    CROWS=(
        "Metric|Previous|Current|Change"
        "Source LOC|$prev_source_loc|$json_source_loc|$(fmt_delta $delta_source_loc)"
        "Test LOC|$prev_test_loc|$json_test_loc|$(fmt_delta $delta_test_loc)"
        "Coverage|${prev_coverage}%|${json_coverage}%|$(fmt_delta_float $delta_coverage)%"
        "Escape Hatches|$prev_escapes|$json_escapes_unwrap|$(fmt_delta $delta_escapes)"
        "Test Count|$prev_test_count|$json_test_count|$(fmt_delta $delta_test_count)"
        "Binary Size|$(echo "scale=1; $prev_binary / 1048576" | bc 2>/dev/null || echo "0") MB|$(echo "scale=1; $json_binary_stripped / 1048576" | bc 2>/dev/null || echo "0") MB|$(fmt_delta_bytes $delta_binary)"
    )

    CW1=13 CW2=8 CW3=7 CW4=8
    for row in "${CROWS[@]}"; do
        IFS='|' read -r c1 c2 c3 c4 <<< "$row"
        vw1=$(visual_width "$c1"); [ $vw1 -gt $CW1 ] && CW1=$vw1
        vw2=$(visual_width "$c2"); [ $vw2 -gt $CW2 ] && CW2=$vw2
        vw3=$(visual_width "$c3"); [ $vw3 -gt $CW3 ] && CW3=$vw3
        vw4=$(visual_width "$c4"); [ $vw4 -gt $CW4 ] && CW4=$vw4
    done

    first=true
    for row in "${CROWS[@]}"; do
        IFS='|' read -r c1 c2 c3 c4 <<< "$row"
        printf "| %-${CW1}s | %${CW2}s | %${CW3}s | %${CW4}s |\n" "$c1" "$c2" "$c3" "$c4"
        if $first; then
            printf "| %s | %s | %s | %s |\n" "$(printf '%*s' $CW1 '' | tr ' ' '-')" "$(printf '%*s' $CW2 '' | tr ' ' '-')" "$(printf '%*s' $CW3 '' | tr ' ' '-')" "$(printf '%*s' $CW4 '' | tr ' ' '-')"
            first=false
        fi
    done
    echo ""
fi

# --- Summary of Failures ---
if [ ${#FAILURES[@]} -gt 0 ]; then
    echo "## Failing Checks"
    echo ""
    for failure in "${FAILURES[@]}"; do
        echo "- $failure"
    done
    echo ""
fi

# Count passes/fails for overall status
passes=0
fails=0
[ $json_source_avg -lt 500 ] && passes=$((passes+1)) || fails=$((fails+1))
[ $json_source_max -lt 900 ] && passes=$((passes+1)) || fails=$((fails+1))
[ $json_test_avg -lt 700 ] && passes=$((passes+1)) || fails=$((fails+1))
[ $json_test_max -lt 1100 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$(echo "$json_coverage >= 85" | bc 2>/dev/null || echo 0)" -eq 1 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$json_escapes_unwrap" -lt 3 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$(echo "$json_test_time_warm < 5" | bc 2>/dev/null || echo 0)" -eq 1 ] && passes=$((passes+1)) || fails=$((fails+1))

echo "---"
echo ""
echo "**Overall: $passes/7 targets passing**"
echo ""
echo "Report saved to: $REPORT_DIR/"
} | tee "$REPORT_DIR/summary.md"

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
