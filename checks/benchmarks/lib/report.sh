#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/lib/report.sh - Generate benchmark reports

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$SCRIPT_DIR/lib/common.sh"

# Convert seconds to human-readable milliseconds
format_time() {
    local seconds="$1"
    printf "%.1fms" "$(echo "$seconds * 1000" | bc -l)"
}

# Extract metrics from a benchmark JSON file
# Returns: mean stddev min max
get_metrics() {
    local file="$1"
    if [[ ! -f "$file" ]]; then
        echo "N/A N/A N/A N/A"
        return
    fi
    jq -r '.results[0] | "\(.mean) \(.stddev) \(.min) \(.max)"' "$file"
}

# Format a row for the scaling table
format_scaling_row() {
    local size="$1"
    local count="$2"

    local default_file="$RESULTS_DIR/list_default_${size}.json"
    local all_file="$RESULTS_DIR/list_all_${size}.json"

    local default_mean="N/A"
    local all_mean="N/A"

    if [[ -f "$default_file" ]]; then
        default_mean=$(format_time "$(jq -r '.results[0].mean' "$default_file")")
    fi

    if [[ -f "$all_file" ]]; then
        all_mean=$(format_time "$(jq -r '.results[0].mean' "$all_file")")
    fi

    echo "| $size | $count | $default_mean | $all_mean |"
}

# Format a row for filter performance table
format_filter_row() {
    local name="$1"
    local file="$RESULTS_DIR/${name}.json"

    if [[ ! -f "$file" ]]; then
        echo "| $name | N/A | N/A |"
        return
    fi

    local mean stddev
    mean=$(format_time "$(jq -r '.results[0].mean' "$file")")
    stddev=$(format_time "$(jq -r '.results[0].stddev' "$file")")

    echo "| $name | $mean | ±$stddev |"
}

# Generate the full report
generate_report() {
    local report_file="$RESULTS_DIR/report.md"
    mkdir -p "$RESULTS_DIR"

    info "Generating benchmark report..."

    cat > "$report_file" << 'HEADER'
# List Benchmark Report

**Generated:** TIMESTAMP
**Database sizes:** small (100), medium (1,000), large (5,000), xlarge (10,000)

## Scaling Analysis

How list performance scales with database size.

| Size | Issues | Default List | List --all |
|------|--------|--------------|------------|
HEADER

    # Replace timestamp
    sed -i '' "s/TIMESTAMP/$(date '+%Y-%m-%d %H:%M:%S')/" "$report_file"

    # Add scaling rows
    {
        format_scaling_row "small" "100"
        format_scaling_row "medium" "1,000"
        format_scaling_row "large" "5,000"
        format_scaling_row "xlarge" "10,000"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

## Filter Performance

Single filter performance on large database (5,000 issues).

### Status Filters

| Filter | Mean | StdDev |
|--------|------|--------|
SECTION

    {
        format_filter_row "filter_status_todo"
        format_filter_row "filter_status_in_progress"
        format_filter_row "filter_status_done"
        format_filter_row "filter_status_multi"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Type Filters

| Filter | Mean | StdDev |
|--------|------|--------|
SECTION

    {
        format_filter_row "filter_type_task"
        format_filter_row "filter_type_bug"
        format_filter_row "filter_type_feature"
        format_filter_row "filter_type_multi"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Label Filters

| Filter | Mean | StdDev |
|--------|------|--------|
SECTION

    {
        format_filter_row "filter_label_project"
        format_filter_row "filter_label_priority"
        format_filter_row "filter_label_area"
        format_filter_row "filter_label_multi_or"
        format_filter_row "filter_label_multi_and"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Assignee Filters

| Filter | Mean | StdDev |
|--------|------|--------|
SECTION

    {
        format_filter_row "filter_assignee_alice"
        format_filter_row "filter_assignee_multi"
        format_filter_row "filter_unassigned"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Other Filters

| Filter | Mean | StdDev |
|--------|------|--------|
SECTION

    {
        format_filter_row "filter_blocked"
        format_filter_row "filter_blocked_todo"
        format_filter_row "filter_age_recent"
        format_filter_row "filter_age_old"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

## Combined Filters

Realistic multi-filter queries on large database.

| Scenario | Mean | StdDev |
|----------|------|--------|
SECTION

    {
        format_filter_row "combined_open_bugs"
        format_filter_row "combined_priority_tasks"
        format_filter_row "combined_my_work"
        format_filter_row "combined_project_all"
        format_filter_row "combined_backlog"
        format_filter_row "combined_complex"
        format_filter_row "combined_restrictive"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

## Limit Performance

Effect of --limit on query time (large database).

| Limit | Mean | StdDev |
|-------|------|--------|
SECTION

    {
        format_filter_row "list_limit_10"
        format_filter_row "list_limit_50"
        format_filter_row "list_limit_100"
        format_filter_row "list_limit_500"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

## Output Format

Comparison of text vs JSON output (large database, --all).

| Format | Mean | StdDev |
|--------|------|--------|
SECTION

    {
        format_filter_row "output_text"
        format_filter_row "output_json"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

## Ready Command

Performance of ready command (hard limit of 5).

| Database | Mean | StdDev |
|----------|------|--------|
SECTION

    {
        format_filter_row "ready_default_small"
        format_filter_row "ready_default_medium"
        format_filter_row "ready_default_large"
        format_filter_row "ready_default_xlarge"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

## Write Operations

Mutation benchmarks with DB restoration between runs.

### Issue Creation

| Operation | Mean | StdDev |
|-----------|------|--------|
SECTION

    {
        format_filter_row "new_sequential_large"
        format_filter_row "new_batch_10"
        format_filter_row "new_batch_50"
        format_filter_row "new_batch_100"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Edit Operations

| Operation | Mean | StdDev |
|-----------|------|--------|
SECTION

    {
        format_filter_row "edit_title"
        format_filter_row "edit_type"
        format_filter_row "edit_assignee"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Lifecycle Operations

| Operation | Mean | StdDev |
|-----------|------|--------|
SECTION

    {
        format_filter_row "start_single"
        format_filter_row "done_single"
        format_filter_row "close_single"
        format_filter_row "close_batch_10"
        format_filter_row "close_batch_50"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

## Search Command

### Search Scaling

| Database | Mean | StdDev |
|----------|------|--------|
SECTION

    {
        format_filter_row "search_basic_small"
        format_filter_row "search_basic_medium"
        format_filter_row "search_basic_large"
        format_filter_row "search_basic_xlarge"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Search Match Rates

| Query Type | Mean | StdDev |
|------------|------|--------|
SECTION

    {
        format_filter_row "search_high_match"
        format_filter_row "search_medium_match"
        format_filter_row "search_low_match"
        format_filter_row "search_no_match"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Search with Filters

| Filter Type | Mean | StdDev |
|-------------|------|--------|
SECTION

    {
        format_filter_row "search_status_todo"
        format_filter_row "search_type_task"
        format_filter_row "search_label_project"
        format_filter_row "search_assignee_alice"
        format_filter_row "search_combined_all"
    } >> "$report_file"

    cat >> "$report_file" << 'SECTION'

### Search Limits

| Limit | Mean | StdDev |
|-------|------|--------|
SECTION

    {
        format_filter_row "search_limit_default"
        format_filter_row "search_limit_10"
        format_filter_row "search_limit_50"
        format_filter_row "search_limit_100"
        format_filter_row "search_limit_unlimited"
    } >> "$report_file"

    cat >> "$report_file" << 'FOOTER'

---

## Notes

- All benchmarks run with 3 warmup iterations and 30 measured runs
- Mean and standard deviation reported
- Times under 100ms generally feel instant to users
- Consider optimization if any filter exceeds 200ms on large database
- Write benchmarks use DB restoration in --prepare phase (not counted in time)
FOOTER

    success "Report generated: $report_file"
}

# Compare two benchmark runs for regression detection
compare_runs() {
    local baseline_dir="$1"
    local current_dir="$2"

    echo "# Regression Analysis"
    echo ""
    echo "Comparing: $baseline_dir vs $current_dir"
    echo ""
    echo "| Benchmark | Baseline | Current | Change |"
    echo "|-----------|----------|---------|--------|"

    for baseline_file in "$baseline_dir"/*.json; do
        local name
        name=$(basename "$baseline_file" .json)
        local current_file="$current_dir/${name}.json"

        if [[ ! -f "$current_file" ]]; then
            continue
        fi

        local baseline_mean current_mean change
        baseline_mean=$(jq -r '.results[0].mean' "$baseline_file")
        current_mean=$(jq -r '.results[0].mean' "$current_file")

        # Calculate percentage change
        change=$(echo "scale=1; (($current_mean - $baseline_mean) / $baseline_mean) * 100" | bc -l)

        local status=""
        if (( $(echo "$change > 20" | bc -l) )); then
            status=" ⚠️"
        fi

        echo "| $name | $(format_time "$baseline_mean") | $(format_time "$current_mean") | ${change}%${status} |"
    done
}

# If run directly
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    if [[ $# -eq 2 ]]; then
        compare_runs "$1" "$2"
    else
        generate_report
    fi
fi
