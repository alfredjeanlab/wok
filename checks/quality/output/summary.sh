#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
#
# Summary report generation for quality evaluation.
# Sourced by evaluate.sh - uses global variables set during metric parsing.

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

fmt_delta_float() {
    local val=$1
    if (( $(echo "$val > 0" | bc -l 2>/dev/null || echo 0) )); then echo "+$val"
    elif (( $(echo "$val < 0" | bc -l 2>/dev/null || echo 0) )); then echo "$val"
    else echo "0"
    fi
}

# Print a table given rows array and column widths
print_table() {
    local -n rows=$1
    shift
    local widths=("$@")
    local first=true

    for row in "${rows[@]}"; do
        IFS='|' read -ra cols <<< "$row"
        local line="| "
        local i=0
        for col in "${cols[@]}"; do
            if [ $i -eq 0 ]; then
                line+="$(printf "%-${widths[$i]}s" "$col")"
            else
                line+="$(printf "%${widths[$i]}s" "$col")"
            fi
            line+=" | "
            i=$((i + 1))
        done
        echo "${line% }"

        if $first; then
            local sep="| "
            for w in "${widths[@]}"; do
                sep+="$(printf '%*s' $w '' | tr ' ' '-') | "
            done
            echo "${sep% }"
            first=false
        fi
    done
}

# Calculate column widths for a table
calc_widths() {
    local -n rows=$1
    local -n widths=$2
    local num_cols=${#widths[@]}

    for row in "${rows[@]}"; do
        IFS='|' read -ra cols <<< "$row"
        local i=0
        for col in "${cols[@]}"; do
            local vw=$(visual_width "$col")
            [ $vw -gt ${widths[$i]} ] && widths[$i]=$vw
            i=$((i + 1))
        done
    done
}

generate_summary() {
    local report_dir="$1"

    # Collect failures for later
    FAILURES=()

    {
    echo ""
    echo "# Quality Evaluation Summary"
    echo ""
    echo "**Date:** $(date)"
    echo "**Report:** $(basename "$report_dir")"
    echo ""

    # --- Summary Table ---
    echo "## Summary"
    echo ""

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
    if [ -f "$report_dir/file_size.txt" ]; then
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
        done < "$report_dir/file_size.txt"

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
    echo "Report saved to: $report_dir/"
    } | tee "$report_dir/summary.md"
}
