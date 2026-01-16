#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Data generation utilities for stress tests
#
# Provides functions to generate random data for stress testing:
# - Random strings (titles, tags, notes)
# - Bulk issue creation
# - Dependency chain creation

set -euo pipefail

# Generate random string of specified length
# Usage: str=$(random_string 100)
random_string() {
    local length="${1:-32}"
    # Use /dev/urandom and base64, then trim to exact length
    head -c "$((length * 2))" /dev/urandom 2>/dev/null | base64 | tr -dc 'a-zA-Z0-9 ' | head -c "$length"
}

# Generate random alphanumeric string (no spaces)
# Usage: str=$(random_alphanum 32)
random_alphanum() {
    local length="${1:-32}"
    head -c "$((length * 2))" /dev/urandom 2>/dev/null | base64 | tr -dc 'a-zA-Z0-9' | head -c "$length"
}

# Generate random word-like string
# Usage: word=$(random_word)
random_word() {
    local length="${1:-8}"
    head -c "$((length * 2))" /dev/urandom 2>/dev/null | base64 | tr -dc 'a-z' | head -c "$length"
}

# Generate random title
# Usage: title=$(random_title)
random_title() {
    local words="${1:-5}"
    local title=""
    for _ in $(seq 1 "$words"); do
        title="$title$(random_word $((RANDOM % 8 + 3))) "
    done
    echo "$title"
}

# Generate random tag
# Usage: tag=$(random_tag)
random_tag() {
    local prefixes=("feature" "bug" "priority" "team" "sprint" "area" "type" "status")
    local prefix="${prefixes[$((RANDOM % ${#prefixes[@]}))]}"
    echo "${prefix}:$(random_word 6)"
}

# Generate N random tags
# Usage: tags=$(random_tags 5)
random_tags() {
    local count="${1:-3}"
    local tags=""
    for _ in $(seq 1 "$count"); do
        tags="$tags$(random_tag) "
    done
    echo "$tags"
}

# Generate random note content
# Usage: note=$(random_note 500)
random_note() {
    local length="${1:-200}"
    # Generate sentences
    local content=""
    while [ ${#content} -lt "$length" ]; do
        local sentence_len=$((RANDOM % 50 + 20))
        content="$content$(random_string "$sentence_len"). "
    done
    echo "${content:0:$length}"
}

# Create issues in batch (faster than sequential)
# Usage: create_issues_batch 1000 "Issue prefix"
create_issues_batch() {
    local count="${1:-100}"
    local prefix="${2:-Issue}"
    local batch_size="${3:-50}"

    local created=0
    while [ $created -lt $count ]; do
        local batch_end=$((created + batch_size))
        [ $batch_end -gt $count ] && batch_end=$count

        # Create batch in parallel
        for i in $(seq $((created + 1)) $batch_end); do
            "$WK_BIN" new task "${prefix} $i" >/dev/null 2>&1 &
        done
        wait

        created=$batch_end
    done

    echo "$created"
}

# Create issues sequentially (for consistent timing)
# Usage: create_issues_sequential 100 "Issue prefix"
create_issues_sequential() {
    local count="${1:-100}"
    local prefix="${2:-Issue}"
    local ids=()

    for i in $(seq 1 "$count"); do
        local output
        output=$("$WK_BIN" new task "${prefix} $i" 2>&1)
        local id
        id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)
        ids+=("$id")
    done

    # Return IDs as space-separated string
    echo "${ids[*]}"
}

# Create linear dependency chain
# Usage: ids=$(create_dependency_chain 100)
create_dependency_chain() {
    local depth="${1:-100}"
    local ids=()
    local prev_id=""

    for i in $(seq 1 "$depth"); do
        local output
        output=$("$WK_BIN" new task "Chain link $i" 2>&1)
        local id
        id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)
        ids+=("$id")

        if [ -n "$prev_id" ]; then
            "$WK_BIN" dep "$prev_id" blocks "$id" >/dev/null 2>&1 || true
        fi
        prev_id="$id"
    done

    echo "${ids[*]}"
}

# Create star pattern dependencies (one blocks many)
# Usage: ids=$(create_star_dependencies 100)
create_star_dependencies() {
    local count="${1:-100}"

    # Create blocker
    local blocker_output
    blocker_output=$("$WK_BIN" new task "The Blocker" 2>&1)
    local blocker
    blocker=$(echo "$blocker_output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    local ids=("$blocker")

    # Create blocked issues
    for i in $(seq 1 "$count"); do
        local output
        output=$("$WK_BIN" new task "Blocked $i" 2>&1)
        local id
        id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)
        ids+=("$id")

        "$WK_BIN" dep "$blocker" blocks "$id" >/dev/null 2>&1 || true
    done

    echo "${ids[*]}"
}

# Create diamond pattern dependencies
# Usage: create_diamond_dependencies
create_diamond_dependencies() {
    local width="${1:-10}"

    # Top of diamond
    local top_output
    top_output=$("$WK_BIN" new task "Diamond Top" 2>&1)
    local top
    top=$(echo "$top_output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    # Middle layer
    local middle_ids=()
    for i in $(seq 1 "$width"); do
        local output
        output=$("$WK_BIN" new task "Diamond Middle $i" 2>&1)
        local id
        id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)
        middle_ids+=("$id")
        "$WK_BIN" dep "$top" blocks "$id" >/dev/null 2>&1 || true
    done

    # Bottom of diamond
    local bottom_output
    bottom_output=$("$WK_BIN" new task "Diamond Bottom" 2>&1)
    local bottom
    bottom=$(echo "$bottom_output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    for mid_id in "${middle_ids[@]}"; do
        "$WK_BIN" dep "$mid_id" blocks "$bottom" >/dev/null 2>&1 || true
    done

    echo "$top ${middle_ids[*]} $bottom"
}

# Generate long string for limit testing
# Usage: str=$(generate_long_string 10000)
generate_long_string() {
    local length="${1:-1000}"
    local char="${2:-X}"

    # Use printf for efficiency
    printf '%*s' "$length" '' | tr ' ' "$char"
}

# Generate string with pattern for limit testing
# Usage: str=$(generate_pattern_string 1000 "Test ")
generate_pattern_string() {
    local length="${1:-1000}"
    local pattern="${2:-Test }"
    local pattern_len=${#pattern}

    local repeats=$((length / pattern_len + 1))
    local result=""
    for _ in $(seq 1 "$repeats"); do
        result="$result$pattern"
    done

    echo "${result:0:$length}"
}
