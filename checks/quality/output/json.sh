#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
#
# JSON output generation for quality evaluation.
# Sourced by evaluate.sh - uses global variables set during metric parsing.

generate_metrics_json() {
    local report_dir="$1"

    cat > "$report_dir/metrics.json" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "report_id": "$(basename "$report_dir")",
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

    echo "Metrics JSON written to: $report_dir/metrics.json"
}
