#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
#
# Comparison report generation for quality evaluation.
# Sourced by evaluate.sh - uses global variables set during metric parsing.

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

# Find the previous report directory for comparison
find_previous_report() {
    local cache_dir="$1"
    local current_name="$2"

    for dir in $(ls -1d "$cache_dir"/20* 2>/dev/null | sort -r); do
        dir_name=$(basename "$dir")
        if [ "$dir_name" != "$current_name" ] && [ -f "$dir/metrics.json" ]; then
            echo "$dir"
            return
        fi
    done
}

# Generate comparison report - sets global variables for use in summary
generate_comparison() {
    local report_dir="$1"
    local prev_report="$2"

    # Read previous values
    prev_source_loc=$(jq -r '.loc.source' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_test_loc=$(jq -r '.loc.test' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_coverage=$(jq -r '.coverage.line_percent' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_escapes=$(jq -r '.escapes.unwrap' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_test_count=$(jq -r '.coverage.test_count' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_binary=$(jq -r '.binary.stripped_bytes' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_source_max=$(jq -r '.file_size.source_max' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_test_max=$(jq -r '.file_size.test_max' "$prev_report/metrics.json" 2>/dev/null || echo 0)

    # Read previous work tracking values
    prev_commits_total=$(jq -r '.work_tracking.commits.total // 0' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_bugs_open=$(jq -r '.work_tracking.bugs.open // 0' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_bugs_fixed=$(jq -r '.work_tracking.bugs.fixed // 0' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_tasks_open=$(jq -r '.work_tracking.tasks.open // 0' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_tasks_closed=$(jq -r '.work_tracking.tasks.closed // 0' "$prev_report/metrics.json" 2>/dev/null || echo 0)
    prev_epics_done=$(jq -r '.work_tracking.epics.done // 0' "$prev_report/metrics.json" 2>/dev/null || echo 0)

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

    echo ""
    echo "=== Comparison with $(basename "$prev_report") ==="

    # Write comparison markdown
    cat > "$report_dir/comparison.md" << EOF
# Quality Comparison Report

**Generated:** $(date)
**Current Report:** $(basename "$report_dir")
**Previous Report:** $(basename "$prev_report")

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
            echo "- **Coverage improved** by ${delta_coverage}% (${prev_coverage}% -> ${json_coverage}%)" >> "$report_dir/comparison.md"
        else
            echo "- **Coverage decreased** by ${delta_coverage}% (${prev_coverage}% -> ${json_coverage}%)" >> "$report_dir/comparison.md"
        fi
    fi

    if [ "$delta_escapes" != "0" ]; then
        if [ "$delta_escapes" -lt 0 ]; then
            echo "- **Escape hatches reduced** by $((-delta_escapes)) ($prev_escapes -> $json_escapes_unwrap)" >> "$report_dir/comparison.md"
        else
            echo "- **Escape hatches increased** by $delta_escapes ($prev_escapes -> $json_escapes_unwrap)" >> "$report_dir/comparison.md"
        fi
    fi

    if [ "$delta_test_count" != "0" ]; then
        echo "- **Test count changed** by $(fmt_delta $delta_test_count) ($prev_test_count -> $json_test_count)" >> "$report_dir/comparison.md"
    fi

    echo "" >> "$report_dir/comparison.md"
    echo "---" >> "$report_dir/comparison.md"
    echo "" >> "$report_dir/comparison.md"
    echo "*Generated from reports/quality/$(basename "$report_dir")*" >> "$report_dir/comparison.md"
}
