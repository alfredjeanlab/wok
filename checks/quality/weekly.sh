#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REPORTS_DIR="${ROOT_DIR}/reports/quality"
OUTPUT_FILE="${ROOT_DIR}/reports/weekly.md"

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Generate a weekly quality report table from recent evaluations.

Options:
    -n, --days N    Number of days to include (default: 7)
    -h, --help      Show this help

Examples:
    $0              # Last 7 days
    $0 -n 14        # Last 14 days
EOF
}

DAYS=7

while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--days) DAYS="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1"; usage; exit 1 ;;
    esac
done

# --- Cleanup old reports within 1 hour of newer ones ---
cleanup_old_reports() {
    local reports_dir="$1"
    local one_hour=3600  # seconds

    local dirs=()
    while IFS= read -r dir; do
        [ -d "$dir" ] && dirs+=("$dir")
    done < <(ls -1d "$reports_dir"/20* 2>/dev/null | sort -r)

    local i=0
    while [ $i -lt ${#dirs[@]} ]; do
        local newer="${dirs[$i]}"
        local newer_name=$(basename "$newer")
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
                    rm -rf "$older"
                    unset 'dirs[$j]'
                    dirs=("${dirs[@]}")
                    continue
                fi
            fi
            j=$((j + 1))
        done
        i=$((i + 1))
    done
}

# Run cleanup first
cleanup_old_reports "$REPORTS_DIR"

# Calculate cutoff timestamp
if [[ "$OSTYPE" == "darwin"* ]]; then
    CUTOFF=$(date -v-${DAYS}d +%Y%m%d)
else
    CUTOFF=$(date -d "-${DAYS} days" +%Y%m%d)
fi

# Get current report timestamp for "now" detection
CURRENT_TS=$(date +%Y%m%d%H%M)

# Convert hour to time-of-day label
time_label() {
    local hour=$1
    local report_ts=$2

    # Check if this is the most recent report (within 10 minutes)
    local current_mins=$((${CURRENT_TS:8:2} * 60 + ${CURRENT_TS:10:2}))
    local report_mins=$((${report_ts:9:2} * 60 + ${report_ts:11:2}))
    local report_date="${report_ts:0:8}"
    local current_date="${CURRENT_TS:0:8}"

    if [ "$report_date" = "$current_date" ] && [ $((current_mins - report_mins)) -lt 30 ] && [ $((current_mins - report_mins)) -ge 0 ]; then
        echo "now"
        return
    fi

    # Remove leading zero for comparison
    hour=$((10#$hour))

    if [ $hour -lt 5 ]; then
        echo "night"
    elif [ $hour -lt 8 ]; then
        echo "early"
    elif [ $hour -lt 12 ]; then
        echo "morning"
    elif [ $hour -lt 14 ]; then
        echo "noon"
    elif [ $hour -lt 18 ]; then
        echo "afternoon"
    elif [ $hour -lt 21 ]; then
        echo "evening"
    else
        echo "night"
    fi
}

# Redirect output to file
mkdir -p "$(dirname "$OUTPUT_FILE")"
exec > "$OUTPUT_FILE"

echo "# Weekly Quality Report"
echo ""
echo "**Generated:** $(date)"
echo "**Period:** Last $DAYS days (since $CUTOFF)"
echo ""

# Collect JSON report files (YYYY-MM-DD.json or YYYY-MM-DD_HHMMSS.json format)
REPORTS=()
for json_file in $(ls -1 "$REPORTS_DIR"/*.json 2>/dev/null | sort); do
    file_name=$(basename "$json_file" .json)
    # Extract date part (handles both YYYY-MM-DD and YYYY-MM-DD_HHMMSS)
    date_part="${file_name:0:10}"
    # Convert YYYY-MM-DD to YYYYMMDD for comparison
    file_date=$(echo "$date_part" | tr -d '-')

    if [ "$file_date" -ge "$CUTOFF" ]; then
        REPORTS+=("$json_file")
    fi
done

if [ ${#REPORTS[@]} -eq 0 ]; then
    echo "No JSON reports found in the last $DAYS days."
    echo ""
    echo "Run ./evaluate.sh to generate a report first."
    exit 0
fi

# Collect all row data first
ROWS=()
for report in "${REPORTS[@]}"; do
    file_name=$(basename "$report" .json)
    # Parse YYYY-MM-DD or YYYY-MM-DD_HHMMSS format
    year="${file_name:0:4}"
    month="${file_name:5:2}"
    day="${file_name:8:2}"
    # Include time if present (for same-day reports)
    if [ ${#file_name} -gt 10 ]; then
        time_part="${file_name:11:6}"
        hour="${time_part:0:2}"
        formatted_date="${month}/${day} ${hour}h"
    else
        formatted_date="${month}/${day}"
    fi

    source_loc=$(jq -r '.loc.source // 0' "$report")
    test_loc=$(jq -r '.loc.test // 0' "$report")
    coverage=$(jq -r '.coverage.line_percent // 0' "$report")
    escapes=$(jq -r '.escapes.unwrap // 0' "$report")
    tests=$(jq -r '.coverage.test_count // 0' "$report")
    binary=$(jq -r '.binary.stripped_bytes // 0' "$report")

    binary_mb=$(echo "scale=1; $binary / 1048576" | bc 2>/dev/null || echo "0")

    [ "$(echo "$coverage >= 80" | bc)" -eq 1 ] && coverage_status="$coverage%" || coverage_status="$coverage% ⚠"
    [ "$escapes" -lt 3 ] && escapes_status="$escapes" || escapes_status="$escapes ⚠"

    ROWS+=("$formatted_date|$source_loc|$test_loc|$coverage_status|$escapes_status|$tests|${binary_mb}MB")
done

# Calculate visual width (accounts for wide emoji characters)
visual_width() {
    local s="$1"
    local w=${#s}
    # Add 1 for each ⚠ emoji (displays as 2 columns but counts as 1 char)
    local emoji_count=$(echo "$s" | grep -o '⚠' | wc -l | tr -d ' ')
    echo $((w + emoji_count))
}

# Right-pad string to target visual width
pad_left() {
    local s="$1"
    local target_width="$2"
    local vw=$(visual_width "$s")
    local pad=$((target_width - vw))
    printf "%${pad}s%s" "" "$s"
}

# Left-pad string to target visual width
pad_right() {
    local s="$1"
    local target_width="$2"
    local vw=$(visual_width "$s")
    local pad=$((target_width - vw))
    printf "%s%${pad}s" "$s" ""
}

# Calculate column widths (start with header lengths)
W1=4 W2=10 W3=8 W4=8 W5=7 W6=5 W7=6  # Date, Source LOC, Test LOC, Coverage, Escapes, Tests, Binary
for row in "${ROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 c5 c6 c7 <<< "$row"
    vw1=$(visual_width "$c1"); [ $vw1 -gt $W1 ] && W1=$vw1
    vw2=$(visual_width "$c2"); [ $vw2 -gt $W2 ] && W2=$vw2
    vw3=$(visual_width "$c3"); [ $vw3 -gt $W3 ] && W3=$vw3
    vw4=$(visual_width "$c4"); [ $vw4 -gt $W4 ] && W4=$vw4
    vw5=$(visual_width "$c5"); [ $vw5 -gt $W5 ] && W5=$vw5
    vw6=$(visual_width "$c6"); [ $vw6 -gt $W6 ] && W6=$vw6
    vw7=$(visual_width "$c7"); [ $vw7 -gt $W7 ] && W7=$vw7
done

# Print table
echo "## Summary Table"
echo ""
printf "| %-${W1}s | %${W2}s | %${W3}s | %${W4}s | %${W5}s | %${W6}s | %${W7}s |\n" "Date" "Source LOC" "Test LOC" "Coverage" "Escapes" "Tests" "Binary"
printf "| %s | %s | %s | %s | %s | %s | %s |\n" "$(printf '%*s' $W1 '' | tr ' ' '-')" "$(printf '%*s' $W2 '' | tr ' ' '-')" "$(printf '%*s' $W3 '' | tr ' ' '-')" "$(printf '%*s' $W4 '' | tr ' ' '-')" "$(printf '%*s' $W5 '' | tr ' ' '-')" "$(printf '%*s' $W6 '' | tr ' ' '-')" "$(printf '%*s' $W7 '' | tr ' ' '-')"
for row in "${ROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 c4 c5 c6 c7 <<< "$row"
    printf "| %s | %s | %s | %s | %s | %s | %s |\n" "$(pad_right "$c1" $W1)" "$(pad_left "$c2" $W2)" "$(pad_left "$c3" $W3)" "$(pad_left "$c4" $W4)" "$(pad_left "$c5" $W5)" "$(pad_left "$c6" $W6)" "$(pad_left "$c7" $W7)"
done

echo ""

# Calculate trends if we have at least 2 reports
if [ ${#REPORTS[@]} -ge 2 ]; then
    FIRST="${REPORTS[0]}"
    LAST="${REPORTS[${#REPORTS[@]}-1]}"

    first_source=$(jq -r '.loc.source // 0' "$FIRST")
    last_source=$(jq -r '.loc.source // 0' "$LAST")
    first_coverage=$(jq -r '.coverage.line_percent // 0' "$FIRST")
    last_coverage=$(jq -r '.coverage.line_percent // 0' "$LAST")
    first_escapes=$(jq -r '.escapes.unwrap // 0' "$FIRST")
    last_escapes=$(jq -r '.escapes.unwrap // 0' "$LAST")
    first_tests=$(jq -r '.coverage.test_count // 0' "$FIRST")
    last_tests=$(jq -r '.coverage.test_count // 0' "$LAST")

    delta_source=$((last_source - first_source))
    delta_coverage=$(echo "$last_coverage - $first_coverage" | bc 2>/dev/null || echo "0")
    delta_escapes=$((last_escapes - first_escapes))
    delta_tests=$((last_tests - first_tests))

    # Format with sign
    fmt() {
        local val=$1
        if [ "$val" -gt 0 ] 2>/dev/null; then echo "+$val"
        elif [ "$val" -lt 0 ] 2>/dev/null; then echo "$val"
        else echo "0"
        fi
    }

    fmt_float() {
        local val=$1
        if (( $(echo "$val > 0" | bc -l 2>/dev/null || echo 0) )); then echo "+$val"
        elif (( $(echo "$val < 0" | bc -l 2>/dev/null || echo 0) )); then echo "$val"
        else echo "0"
        fi
    }

    # Format report names as human-friendly dates (handles YYYY-MM-DD and YYYY-MM-DD_HHMMSS)
    first_name=$(basename "$FIRST" .json)
    last_name=$(basename "$LAST" .json)
    first_formatted="${first_name:5:2}/${first_name:8:2}"
    last_formatted="${last_name:5:2}/${last_name:8:2}"
    # Append time if present
    [ ${#first_name} -gt 10 ] && first_formatted="${first_formatted} ${first_name:11:2}h"
    [ ${#last_name} -gt 10 ] && last_formatted="${last_formatted} ${last_name:11:2}h"
    echo "## Trends ($first_formatted → $last_formatted)"
    echo ""

    # Build trends table data
    TROWS=("Metric|Change" "Source LOC|$(fmt $delta_source)" "Coverage|$(fmt_float $delta_coverage)%" "Escape Hatches|$(fmt $delta_escapes)" "Test Count|$(fmt $delta_tests)")
    TW1=6 TW2=6
    for row in "${TROWS[@]}"; do
        IFS='|' read -r c1 c2 <<< "$row"
        [ ${#c1} -gt $TW1 ] && TW1=${#c1}
        [ ${#c2} -gt $TW2 ] && TW2=${#c2}
    done

    first=true
    for row in "${TROWS[@]}"; do
        IFS='|' read -r c1 c2 <<< "$row"
        printf "| %-${TW1}s | %${TW2}s |\n" "$c1" "$c2"
        if $first; then
            printf "| %-${TW1}s | %${TW2}s |\n" "$(printf '%*s' $TW1 '' | tr ' ' '-')" "$(printf '%*s' $TW2 '' | tr ' ' '-')"
            first=false
        fi
    done
    echo ""

    # Work tracking trends (if work_tracking data exists)
    first_commits=$(jq -r '.work_tracking.commits.total // 0' "$FIRST" 2>/dev/null)
    last_commits=$(jq -r '.work_tracking.commits.total // 0' "$LAST" 2>/dev/null)
    first_bugs_open=$(jq -r '.work_tracking.bugs.open // 0' "$FIRST" 2>/dev/null)
    last_bugs_open=$(jq -r '.work_tracking.bugs.open // 0' "$LAST" 2>/dev/null)
    first_bugs_fixed=$(jq -r '.work_tracking.bugs.fixed // 0' "$FIRST" 2>/dev/null)
    last_bugs_fixed=$(jq -r '.work_tracking.bugs.fixed // 0' "$LAST" 2>/dev/null)
    first_tasks_closed=$(jq -r '.work_tracking.tasks.closed // 0' "$FIRST" 2>/dev/null)
    last_tasks_closed=$(jq -r '.work_tracking.tasks.closed // 0' "$LAST" 2>/dev/null)
    first_epics_done=$(jq -r '.work_tracking.epics.done // 0' "$FIRST" 2>/dev/null)
    last_epics_done=$(jq -r '.work_tracking.epics.done // 0' "$LAST" 2>/dev/null)

    # Only show work tracking if we have any data
    if [ "$last_commits" != "null" ] && [ "$last_commits" != "0" ] || [ "$last_bugs_open" != "0" ] || [ "$last_tasks_closed" != "0" ]; then
        delta_commits=$((last_commits - first_commits))
        delta_bugs_open=$((last_bugs_open - first_bugs_open))
        delta_bugs_fixed=$((last_bugs_fixed - first_bugs_fixed))
        delta_tasks_closed=$((last_tasks_closed - first_tasks_closed))
        delta_epics_done=$((last_epics_done - first_epics_done))

        echo "## Work Tracking"
        echo ""

        WROWS=(
            "Metric|This Week|Previous|Delta"
            "Commits|$last_commits|$first_commits|$(fmt $delta_commits)"
            "Bugs Open|$last_bugs_open|$first_bugs_open|$(fmt $delta_bugs_open)"
            "Bugs Fixed|$last_bugs_fixed|$first_bugs_fixed|$(fmt $delta_bugs_fixed)"
            "Tasks Closed|$last_tasks_closed|$first_tasks_closed|$(fmt $delta_tasks_closed)"
            "Epics Done|$last_epics_done|$first_epics_done|$(fmt $delta_epics_done)"
        )

        WW1=12 WW2=9 WW3=8 WW4=5
        for row in "${WROWS[@]}"; do
            IFS='|' read -r c1 c2 c3 c4 <<< "$row"
            [ ${#c1} -gt $WW1 ] && WW1=${#c1}
            [ ${#c2} -gt $WW2 ] && WW2=${#c2}
            [ ${#c3} -gt $WW3 ] && WW3=${#c3}
            [ ${#c4} -gt $WW4 ] && WW4=${#c4}
        done

        first=true
        for row in "${WROWS[@]}"; do
            IFS='|' read -r c1 c2 c3 c4 <<< "$row"
            printf "| %-${WW1}s | %${WW2}s | %${WW3}s | %${WW4}s |\n" "$c1" "$c2" "$c3" "$c4"
            if $first; then
                printf "| %s | %s | %s | %s |\n" "$(printf '%*s' $WW1 '' | tr ' ' '-')" "$(printf '%*s' $WW2 '' | tr ' ' '-')" "$(printf '%*s' $WW3 '' | tr ' ' '-')" "$(printf '%*s' $WW4 '' | tr ' ' '-')"
                first=false
            fi
        done
        echo ""
    fi
fi

# Target status
echo "## Target Status"
echo ""

last_report="${REPORTS[${#REPORTS[@]}-1]}"
source_avg=$(jq -r '.file_size.source_avg // 0' "$last_report")
source_max=$(jq -r '.file_size.source_max // 0' "$last_report")
test_avg=$(jq -r '.file_size.test_avg // 0' "$last_report")
test_max=$(jq -r '.file_size.test_max // 0' "$last_report")
coverage=$(jq -r '.coverage.line_percent // 0' "$last_report")
escapes=$(jq -r '.escapes.unwrap // 0' "$last_report")
test_warm=$(jq -r '.timing.test_warm_seconds // 0' "$last_report")
test_cold=$(jq -r '.timing.test_cold_seconds // 0' "$last_report")
compile_cold=$(jq -r '.timing.compile_cold_seconds // 0' "$last_report")

# Work tracking targets
commits_total=$(jq -r '.work_tracking.commits.total // 0' "$last_report")
bugs_open=$(jq -r '.work_tracking.bugs.open // 0' "$last_report")

pf() { [ "$1" = "true" ] && echo "PASS" || echo "**FAIL**"; }

# Build target status table
SROWS=(
    "Target|Value|Status"
    "Src avg ln <500|$source_avg|$(pf $([ "$source_avg" -lt 500 ] && echo true || echo false))"
    "Src max ln <900|$source_max|$(pf $([ "$source_max" -lt 900 ] && echo true || echo false))"
    "Test avg ln <700|$test_avg|$(pf $([ "$test_avg" -lt 700 ] && echo true || echo false))"
    "Test max ln <1100|$test_max|$(pf $([ "$test_max" -lt 1100 ] && echo true || echo false))"
    "Coverage >85%|$coverage%|$(pf $([ "$(echo "$coverage >= 85" | bc)" -eq 1 ] && echo true || echo false))"
    "Escapes <3|$escapes|$(pf $([ "$escapes" -lt 3 ] && echo true || echo false))"
    "Test warm <5s|${test_warm}s|$(pf $([ "$(echo "$test_warm < 5" | bc)" -eq 1 ] && echo true || echo false))"
    "Test cold <20s|${test_cold}s|$(pf $([ "$(echo "$test_cold < 20" | bc)" -eq 1 ] && echo true || echo false))"
    "Compile 5-30s|${compile_cold}s|$(pf $([ "$(echo "$compile_cold >= 5 && $compile_cold <= 30" | bc)" -eq 1 ] && echo true || echo false))"
    "Commits >=5|$commits_total|$([ "$commits_total" -ge 5 ] && echo "PASS" || echo "info")"
    "Bugs <=5|$bugs_open|$([ "$bugs_open" -le 5 ] && echo "PASS" || echo "⚠ warn")"
)

SW1=6 SW2=5 SW3=6
for row in "${SROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 <<< "$row"
    [ ${#c1} -gt $SW1 ] && SW1=${#c1}
    [ ${#c2} -gt $SW2 ] && SW2=${#c2}
    [ ${#c3} -gt $SW3 ] && SW3=${#c3}
done

first=true
for row in "${SROWS[@]}"; do
    IFS='|' read -r c1 c2 c3 <<< "$row"
    printf "| %-${SW1}s | %${SW2}s | %${SW3}s |\n" "$c1" "$c2" "$c3"
    if $first; then
        printf "| %-${SW1}s | %${SW2}s | %${SW3}s |\n" "$(printf '%*s' $SW1 '' | tr ' ' '-')" "$(printf '%*s' $SW2 '' | tr ' ' '-')" "$(printf '%*s' $SW3 '' | tr ' ' '-')"
        first=false
    fi
done
echo ""

# Count passes/fails
passes=0
fails=0
[ "$source_avg" -lt 500 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$source_max" -lt 900 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$test_avg" -lt 700 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$test_max" -lt 1100 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$(echo "$coverage >= 80" | bc)" -eq 1 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$escapes" -lt 3 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$(echo "$test_warm < 5" | bc)" -eq 1 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$(echo "$test_cold < 20" | bc)" -eq 1 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$(echo "$compile_cold >= 5 && $compile_cold <= 30" | bc)" -eq 1 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$commits_total" -ge 5 ] && passes=$((passes+1)) || fails=$((fails+1))
[ "$bugs_open" -le 5 ] && passes=$((passes+1)) || fails=$((fails+1))

echo "**Overall: $passes/11 targets passing**"

# Output message to stderr
echo "Weekly report written to: $OUTPUT_FILE" >&2
