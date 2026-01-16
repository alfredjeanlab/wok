#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# checks/benchmarks/setup/generate_db.sh - Generate test databases for benchmarks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Source common utilities
source "$SCRIPT_DIR/lib/common.sh"

# Database size configurations
# Returns: issues max_labels dep_percentage
get_db_config() {
    local size="$1"
    case "$size" in
        small)  echo "100 2 10" ;;
        medium) echo "1000 3 15" ;;
        large)  echo "5000 4 20" ;;
        xlarge) echo "10000 4 20" ;;
        *)      echo "" ;;
    esac
}

# Labels to distribute
PROJECTS="alpha beta gamma"
PRIORITIES="1 2 3"
AREAS="frontend backend infra"
ASSIGNEES="alice bob carol david eve"

# Get nth item from space-separated list
get_item() {
    local list="$1"
    local index="$2"
    echo "$list" | tr ' ' '\n' | sed -n "$((index + 1))p"
}

# Count items in space-separated list
count_items() {
    echo "$1" | tr ' ' '\n' | wc -l | tr -d ' '
}

# Generate a single database
# Usage: generate_db <size>
generate_db() {
    local size="$1"
    local config
    config=$(get_db_config "$size")

    if [[ -z "$config" ]]; then
        error "Unknown size: $size"
        return 1
    fi

    local count max_labels dep_pct
    read -r count max_labels dep_pct <<< "$config"

    info "Generating $size database ($count issues)..."

    # Work in a temp directory
    local work_dir
    work_dir=$(mktemp -d)
    pushd "$work_dir" > /dev/null

    # Initialize wk
    "$WK_BIN" init --prefix bench > /dev/null

    # Store created IDs in a file for later reference
    local ids_file
    ids_file=$(mktemp)

    local num_projects num_priorities num_areas num_assignees
    num_projects=$(count_items "$PROJECTS")
    num_priorities=$(count_items "$PRIORITIES")
    num_areas=$(count_items "$AREAS")
    num_assignees=$(count_items "$ASSIGNEES")

    for i in $(seq 1 "$count"); do
        # Determine type (60% task, 20% bug, 15% feature, 5% epic)
        local type="task"
        local type_roll=$((i % 20))
        if [[ $type_roll -lt 1 ]]; then
            type="epic"
        elif [[ $type_roll -lt 4 ]]; then
            type="feature"
        elif [[ $type_roll -lt 8 ]]; then
            type="bug"
        fi

        # Pick labels
        local project priority area
        project=$(get_item "$PROJECTS" $((i % num_projects)))
        priority=$(get_item "$PRIORITIES" $((i % num_priorities)))
        area=$(get_item "$AREAS" $((i % num_areas)))

        # Determine number of labels for this issue
        local num_labels=$(( (i % max_labels) + 1 ))
        local labels="project:$project"
        [[ $num_labels -ge 2 ]] && labels="$labels,priority:$priority"
        [[ $num_labels -ge 3 ]] && labels="$labels,area:$area"

        # Assignee (80% assigned, 20% unassigned)
        local assignee_opt=""
        if [[ $((i % 5)) -ne 0 ]]; then
            local assignee
            assignee=$(get_item "$ASSIGNEES" $((i % num_assignees)))
            assignee_opt="-a $assignee"
        fi

        # Create the issue
        local output
        output=$("$WK_BIN" new "$type" "Issue $i: $type for $project" -l "$labels" $assignee_opt 2>&1) || true

        # Extract the ID from output (format: "Created bench-XXXX")
        local id
        id=$(echo "$output" | grep -oE 'bench-[0-9a-f]+' | head -1) || true
        if [[ -n "$id" ]]; then
            echo "$id" >> "$ids_file"
        fi

        # Progress indicator
        if [[ $((i % 100)) -eq 0 ]]; then
            echo -ne "\r  Created $i / $count issues..."
        fi
    done
    echo -e "\r  Created $count / $count issues...done"

    # Set statuses (40% todo, 30% in_progress, 30% done)
    info "Setting issue statuses..."
    local num_ids
    num_ids=$(wc -l < "$ids_file" | tr -d ' ')
    local in_progress_count=$((num_ids * 30 / 100))
    local done_count=$((num_ids * 30 / 100))

    # Read IDs into array-like processing
    local line_num=0
    while IFS= read -r id; do
        if [[ $line_num -lt $in_progress_count ]]; then
            "$WK_BIN" start "$id" > /dev/null 2>&1 || true
        elif [[ $line_num -lt $((in_progress_count + done_count)) ]]; then
            "$WK_BIN" start "$id" > /dev/null 2>&1 || true
            "$WK_BIN" done "$id" > /dev/null 2>&1 || true
        fi
        line_num=$((line_num + 1))
    done < "$ids_file"

    # Add dependencies (creates blocked issues)
    info "Adding dependencies ($dep_pct% with blockers)..."
    local dep_count=$((num_ids * dep_pct / 100))

    # Read all IDs into a single string for random access
    local all_ids
    all_ids=$(cat "$ids_file")
    local ids_array
    IFS=$'\n' read -d '' -ra ids_array <<< "$all_ids" || true

    for _ in $(seq 1 "$dep_count"); do
        local blocked_idx=$((RANDOM % num_ids))
        local blocker_idx=$((RANDOM % num_ids))

        # Avoid self-dependencies
        if [[ $blocked_idx -ne $blocker_idx ]]; then
            local blocked="${ids_array[$blocked_idx]}"
            local blocker="${ids_array[$blocker_idx]}"
            "$WK_BIN" dep "$blocker" blocks "$blocked" > /dev/null 2>&1 || true
        fi
    done

    # Export the database as SQL
    local sql_file="$SCRIPT_DIR/setup/${size}.sql"

    # Verify count before exporting
    local issue_count
    issue_count=$(sqlite3 .work/issues.db "SELECT COUNT(*) FROM issues;")

    sqlite3 .work/issues.db ".dump" > "$sql_file"

    popd > /dev/null
    rm -rf "$work_dir"
    rm -f "$ids_file"

    success "Generated $size.sql with $issue_count issues"
}

# Generate all databases
generate_all_databases() {
    for size in small medium large xlarge; do
        generate_db "$size"
    done

    success "All databases generated!"
}

# Validate size argument
is_valid_size() {
    local size="$1"
    case "$size" in
        small|medium|large|xlarge) return 0 ;;
        *) return 1 ;;
    esac
}

# If run directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    check_dependencies
    check_wk_binary

    # Convert WK_BIN to absolute path
    if [[ "$WK_BIN" != /* ]]; then
        WK_BIN="$(cd "$(dirname "$WK_BIN")" && pwd)/$(basename "$WK_BIN")"
    fi

    if [[ $# -eq 0 ]]; then
        generate_all_databases
    else
        for size in "$@"; do
            if is_valid_size "$size"; then
                generate_db "$size"
            else
                error "Unknown size: $size (valid: small, medium, large, xlarge)"
                exit 1
            fi
        done
    fi
fi
