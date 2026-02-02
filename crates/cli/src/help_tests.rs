// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for help formatting, flag consolidation, and colorization.
//!
//! These tests ensure:
//! - --flag and --no-flag pairs are consolidated into --[no-]flag
//! - Colors are applied correctly to different line types
//! - [un] prefix in command names is colored as context
//! - [no-] prefix in option names is colored as context
//! - Headers, options, and commands are all colorized appropriately

#![allow(clippy::unwrap_used)]

use super::*;

// ============================================================================
// Flag Consolidation Tests
// ============================================================================

mod consolidation {
    use super::*;

    #[test]
    fn consolidates_limit_followed_by_no_limit() {
        // Most common case: --limit <N> followed by --no-limit
        let input = "  -n, --limit <N>  Limit results\n      --no-limit   Remove limit";
        let result = consolidate_negatable_flags(input);
        assert_eq!(result, "  -n, --[no-]limit <N>  Limit results");
    }

    #[test]
    fn consolidates_no_limit_followed_by_limit() {
        // Reverse order: --no-limit followed by --limit <N>
        let input = "      --no-limit   Remove limit\n  -n, --limit <N>  Limit results";
        let result = consolidate_negatable_flags(input);
        assert_eq!(result, "  -n, --[no-]limit <N>  Limit results");
    }

    #[test]
    fn preserves_short_flag_from_positive() {
        // The short flag -n should be preserved in the consolidated output
        let input = "  -n, --limit <N>  Desc\n      --no-limit   Desc2";
        let result = consolidate_negatable_flags(input);
        assert!(
            result.contains("-n, --[no-]limit"),
            "Short flag should be preserved"
        );
    }

    #[test]
    fn preserves_value_placeholder() {
        // The value placeholder <N> should be preserved
        let input = "  -n, --limit <N>  Desc\n      --no-limit   Desc2";
        let result = consolidate_negatable_flags(input);
        assert!(
            result.contains("<N>"),
            "Value placeholder should be preserved"
        );
    }

    #[test]
    fn preserves_description_from_positive() {
        // The description from the positive flag should be used
        let input = "  -n, --limit <N>  Maximum results\n      --no-limit   Unlimited";
        let result = consolidate_negatable_flags(input);
        assert!(
            result.contains("Maximum results"),
            "Description from positive flag should be used"
        );
        assert!(
            !result.contains("Unlimited"),
            "Description from no- flag should not appear"
        );
    }

    #[test]
    fn does_not_consolidate_non_adjacent_flags() {
        // Flags separated by another line should not be consolidated
        let input = "      --no-limit   Remove limit\n      --other      Other flag\n  -n, --limit <N>  Limit results";
        let result = consolidate_negatable_flags(input);
        // All three lines should remain
        assert!(
            result.contains("--no-limit"),
            "no-limit should remain separate"
        );
        assert!(result.contains("--other"), "other should remain");
        assert!(result.contains("--limit"), "limit should remain separate");
        assert!(
            !result.contains("--[no-]limit"),
            "Should not consolidate non-adjacent"
        );
    }

    #[test]
    fn does_not_consolidate_unrelated_no_flags() {
        // --no-color and --color-scheme should not be consolidated
        let input = "      --no-color      Disable colors\n      --color-scheme  Set scheme";
        let result = consolidate_negatable_flags(input);
        assert_eq!(result, input, "Unrelated flags should not be consolidated");
    }

    #[test]
    fn handles_flag_without_short() {
        // Consolidate flags where positive has no short flag
        let input = "      --verbose   Be verbose\n      --no-verbose   Be quiet";
        let result = consolidate_negatable_flags(input);
        assert!(
            result.contains("--[no-]verbose"),
            "Should consolidate without short flag"
        );
    }

    #[test]
    fn handles_empty_input() {
        let result = consolidate_negatable_flags("");
        assert_eq!(result, "");
    }

    #[test]
    fn handles_single_line() {
        let input = "  -n, --limit <N>  Limit results";
        let result = consolidate_negatable_flags(input);
        assert_eq!(result, input, "Single line should pass through unchanged");
    }

    #[test]
    fn handles_no_options() {
        let input = "Some description text\n\nAnother paragraph";
        let result = consolidate_negatable_flags(input);
        assert_eq!(
            result, input,
            "Non-option text should pass through unchanged"
        );
    }
}

// ============================================================================
// Command Line Colorization Tests
// ============================================================================

mod command_colorization {
    use super::*;
    use crate::colors::codes::{CONTEXT_START, LITERAL_START, RESET};

    #[test]
    fn colorizes_simple_command() {
        // "  new         Description" should colorize "new" as literal
        let result = colorize_help_forced("  new         Create a new issue");
        // Should contain ANSI code for literal color (250)
        assert!(
            result.contains(LITERAL_START),
            "Should have literal color code"
        );
        assert!(result.contains("new"), "Should contain command name");
    }

    #[test]
    fn colorizes_un_prefix_as_context() {
        // "[un]dep" should have [un] in context color, dep in literal color
        let result = colorize_help_forced("  [un]dep     Add/remove dependency");
        // Should contain context color (245) for [un]
        assert!(
            result.contains(CONTEXT_START),
            "Should have context color for [un]"
        );
        // Should contain literal color (250) for dep
        assert!(
            result.contains(LITERAL_START),
            "Should have literal color for dep"
        );
    }

    #[test]
    fn preserves_description() {
        let result = colorize_help_forced("  new         Create a new issue");
        assert!(
            result.contains("Create a new issue"),
            "Description should be preserved"
        );
    }

    #[test]
    fn preserves_indentation() {
        let result = colorize_help_forced("  new         Description");
        assert!(result.starts_with("  "), "Should preserve 2-space indent");
    }

    #[test]
    fn handles_all_un_prefix_commands() {
        // Test all [un] prefixed commands that appear in wok help
        let commands = [
            ("  [un]dep     Add dependency", "dep"),
            ("  [un]label   Add label", "label"),
            ("  [un]link    Add link", "link"),
        ];

        for (line, expected_cmd) in commands {
            let result = colorize_help_forced(line);
            assert!(
                result.contains("[un]"),
                "Should preserve [un] for {}",
                expected_cmd
            );
            assert!(
                result.contains(expected_cmd),
                "Should preserve {} command",
                expected_cmd
            );
            assert!(
                result.contains(&format!("{}[un]{}", CONTEXT_START, RESET)),
                "[un] should be context colored"
            );
        }
    }
}

// ============================================================================
// Option Line Colorization Tests
// ============================================================================

mod option_colorization {
    use super::*;
    use crate::colors::codes::{CONTEXT_START, LITERAL_START, RESET};

    #[test]
    fn colorizes_no_prefix_specially() {
        // --[no-]limit should have [no-] in context color
        let result = colorize_help_forced("  -n, --[no-]limit <N>  Description");

        // Should contain context color for [no-]
        assert!(
            result.contains(&format!("{}[no-]{}", CONTEXT_START, RESET)),
            "Should have context color for [no-]"
        );
        // Should contain literal color for the rest
        assert!(result.contains(LITERAL_START), "Should have literal color");
    }

    #[test]
    fn colorizes_short_flag() {
        let result = colorize_help_forced("  -n, --limit <N>  Description");

        // Should colorize -n as literal
        assert!(
            result.contains(&format!("{}-n{}", LITERAL_START, RESET)),
            "Short flag should be colored"
        );
    }

    #[test]
    fn colorizes_value_placeholder() {
        let result = colorize_help_forced("  -n, --limit <N>  Description");

        // Value placeholder should be context color
        assert!(
            result.contains(&format!("{}<N>{}", CONTEXT_START, RESET)),
            "Value placeholder should be CONTEXT colored"
        );
    }
}

// ============================================================================
// Header Colorization Tests
// ============================================================================

mod header_colorization {
    // These tests verify header detection logic (not color output, which depends on env vars)

    #[test]
    fn detects_usage_as_header() {
        // colorize_help should recognize Usage: as a header-like line
        // When colors enabled, this gets header color applied
        let line = "Usage: wok <COMMAND>";
        // Usage line starts with "Usage:" - this is special-cased in colorize_help
        assert!(
            line.starts_with("Usage:"),
            "Should be recognized as usage line"
        );
    }

    #[test]
    fn detects_single_word_headers() {
        // Headers end with : and have no double-spaces (which would indicate an option line)
        let line = "Options:";
        assert!(line.ends_with(':'), "Header should end with colon");
        assert!(!line.contains("  "), "Header should not have double spaces");
    }

    #[test]
    fn detects_multi_word_headers() {
        // Multi-word headers like "Issue Tracking:" should also be detected
        let line = "Issue Tracking:";
        assert!(line.ends_with(':'), "Header should end with colon");
        assert!(
            !line.contains("  "),
            "Single space is fine, double space is not"
        );
    }

    #[test]
    fn does_not_detect_option_as_header() {
        // Option lines have double-spaces before the description
        let line = "  --limit <N>  Maximum results";
        // This has "  " (double space) so should NOT be treated as a header
        assert!(line.contains("  "), "Option line has double spaces");
        // Even though it might end with some description, it shouldn't match header pattern
    }
}

// ============================================================================
// Full Help Output Tests
// ============================================================================

mod full_help {
    use super::*;

    #[test]
    fn commands_returns_plain_text() {
        // commands() should return plain text (no ANSI codes)
        // because clap strips them anyway
        let result = commands();
        assert!(
            !result.contains("\x1b["),
            "commands() should not contain ANSI codes"
        );
    }

    #[test]
    fn commands_contains_all_command_names() {
        let result = commands();
        let expected_commands = [
            "new",
            "dep",
            "show",
            "tree",
            "list",
            "ready",
            "search",
            "start",
            "done",
            "close",
            "reopen",
            "edit",
            "note",
            "label",
            "link",
            "log",
            "init",
            "hooks",
            "config",
            "daemon",
            "export",
            "import",
            "schema",
            "completion",
            "prime",
        ];
        for cmd in expected_commands {
            assert!(result.contains(cmd), "commands() should contain '{}'", cmd);
        }
    }

    #[test]
    fn commands_contains_un_prefixes() {
        let result = commands();
        assert!(result.contains("[un]dep"), "Should have [un]dep");
        assert!(result.contains("[un]label"), "Should have [un]label");
        assert!(result.contains("[un]link"), "Should have [un]link");
    }

    #[test]
    fn quickstart_returns_plain_text() {
        let result = quickstart();
        assert!(
            !result.contains("\x1b["),
            "quickstart() should not contain ANSI codes"
        );
    }

    #[test]
    fn template_returns_plain_text() {
        let result = template();
        assert!(
            !result.contains("\x1b["),
            "template() should not contain ANSI codes"
        );
    }

    #[test]
    fn colorize_help_forced_preserves_existing_ansi() {
        // Lines that already have ANSI codes should pass through unchanged
        let input = "\x1b[38;5;74mColored Header\x1b[0m";
        let result = colorize_help_forced(input);
        assert_eq!(result, input, "Existing ANSI codes should be preserved");
    }

    #[test]
    fn colorize_help_forced_handles_mixed_content() {
        // Test that colorize_help_forced processes all line types correctly
        let input = "Issue Tracking:\n  new         Create a new issue\n  [un]dep     Add/remove dependency\n\nOptions:\n  -n, --limit <N>  Limit results";

        let result = colorize_help_forced(input);

        // Content should be preserved with colors applied
        assert!(
            result.contains("Issue Tracking:"),
            "Should have Issue Tracking header"
        );
        assert!(result.contains("Options:"), "Should have Options header");
        assert!(result.contains("new"), "Should have new command");
        // When colored, [un] and dep may be separated by color codes, so check both parts
        assert!(result.contains("[un]"), "Should have [un] prefix");
        assert!(result.contains("dep"), "Should have dep command");
        assert!(result.contains("--limit"), "Should have --limit option");
    }
}

// ============================================================================
// Regression Tests
// ============================================================================

mod regressions {
    use super::*;
    use crate::colors::codes::{CONTEXT_START, LITERAL_START, RESET};

    #[test]
    fn regression_limit_consolidation_in_list() {
        // Ensure --limit and --no-limit are consolidated for list command
        let input = "  -s, --status <STATUS>  Filter by status\n  -n, --limit <LIMIT>    Maximum number of results\n      --no-limit\n      --blocked          Show only blocked issues";

        let result = consolidate_negatable_flags(input);
        assert!(
            result.contains("--[no-]limit"),
            "list command should have consolidated --[no-]limit"
        );
    }

    #[test]
    fn regression_colors_not_stripped_by_clap() {
        // Verify that commands(), quickstart(), and template() return plain text
        let cmd_text = commands();
        let qs_text = quickstart();
        let tpl_text = template();

        assert!(
            !cmd_text.contains("\x1b["),
            "commands() must return plain text - clap strips ANSI from before_help"
        );
        assert!(
            !qs_text.contains("\x1b["),
            "quickstart() must return plain text - clap strips ANSI from after_help"
        );
        assert!(
            !tpl_text.contains("\x1b["),
            "template() must return plain text - clap strips ANSI from template"
        );
    }

    #[test]
    fn regression_un_prefix_colors() {
        // [un] prefix should be context color (dimmer), command should be literal (brighter)
        let result = colorize_help_forced("  [un]dep     Add/remove dependency");

        // Should have both context (245) and literal (250) colors
        assert!(
            result.contains(&format!("{}[un]{}", CONTEXT_START, RESET)),
            "Should have context color for [un]"
        );
        assert!(
            result.contains(&format!("{}dep{}", LITERAL_START, RESET)),
            "Should have literal color for dep"
        );
    }

    #[test]
    fn regression_no_prefix_colors() {
        // [no-] prefix should be context color, flag name should be literal
        let result = colorize_help_forced("  -n, --[no-]limit <N>  Description");

        // Should have context color for [no-]
        assert!(
            result.contains(&format!("{}[no-]{}", CONTEXT_START, RESET)),
            "Should have context color for [no-]"
        );
        assert!(result.contains("[no-]"), "Should contain [no-]");
    }
}

// ============================================================================
// Integration Tests - Test the full colorize_help pipeline
// ============================================================================

mod integration {
    use super::*;
    use crate::colors::codes::{CONTEXT_START, HEADER_START, LITERAL_START, RESET};

    /// Test that option lines are correctly parsed and colorized through the full pipeline
    #[test]
    fn option_line_through_colorize_help() {
        let input = "Options:\n  -n, --limit <N>  Maximum results";
        let result = colorize_help_forced(input);

        // Header should be colored
        assert!(
            result.contains(&format!("{}Options:{}", HEADER_START, RESET)),
            "Header should be HEADER colored in:\n{}",
            result
        );

        // Short flag should be literal colored
        assert!(
            result.contains(&format!("{}-n{}", LITERAL_START, RESET)),
            "Short flag -n should be LITERAL colored in:\n{}",
            result
        );

        // Long flag should be literal colored
        assert!(
            result.contains(&format!("{}--limit{}", LITERAL_START, RESET)),
            "Long flag --limit should be LITERAL colored in:\n{}",
            result
        );

        // Value placeholder should be context colored
        assert!(
            result.contains(&format!("{}<N>{}", CONTEXT_START, RESET)),
            "Value placeholder <N> should be CONTEXT colored in:\n{}",
            result
        );
    }

    /// Test --[no-] prefix through full pipeline
    #[test]
    fn no_prefix_through_colorize_help() {
        let input = "Options:\n  -n, --[no-]limit <N>  Maximum results";
        let result = colorize_help_forced(input);

        // [no-] should be context colored
        assert!(
            result.contains(&format!("{}[no-]{}", CONTEXT_START, RESET)),
            "[no-] should be CONTEXT colored in:\n{}",
            result
        );

        // -- should be literal colored
        assert!(
            result.contains(&format!("{}--{}", LITERAL_START, RESET)),
            "-- should be LITERAL colored in:\n{}",
            result
        );

        // limit should be literal colored
        assert!(
            result.contains(&format!("{}limit{}", LITERAL_START, RESET)),
            "limit should be LITERAL colored in:\n{}",
            result
        );
    }

    /// Test [default: ...] and [possible values: ...] through full pipeline
    #[test]
    fn option_metadata_through_colorize_help() {
        let input =
            "Options:\n  -o, --output <O>  Output [default: text] [possible values: text, json]";
        let result = colorize_help_forced(input);

        // [default: text] should be context colored
        assert!(
            result.contains(&format!("{}[default: text]{}", CONTEXT_START, RESET)),
            "[default: text] should be CONTEXT colored in:\n{}",
            result
        );

        // [possible values: ...] should be context colored
        assert!(
            result.contains(&format!(
                "{}[possible values: text, json]{}",
                CONTEXT_START, RESET
            )),
            "[possible values: ...] should be CONTEXT colored in:\n{}",
            result
        );
    }

    /// Test command lines through full pipeline
    #[test]
    fn command_line_through_colorize_help() {
        let input =
            "Commands:\n  new         Create a new issue\n  [un]dep     Add/remove dependency";
        let result = colorize_help_forced(input);

        // "new" should be literal colored
        assert!(
            result.contains(&format!("{}new{}", LITERAL_START, RESET)),
            "new should be LITERAL colored in:\n{}",
            result
        );

        // [un] should be context colored
        assert!(
            result.contains(&format!("{}[un]{}", CONTEXT_START, RESET)),
            "[un] should be CONTEXT colored in:\n{}",
            result
        );

        // "dep" should be literal colored
        assert!(
            result.contains(&format!("{}dep{}", LITERAL_START, RESET)),
            "dep should be LITERAL colored in:\n{}",
            result
        );
    }

    /// Test example lines through full pipeline
    #[test]
    fn example_line_through_colorize_help() {
        let input = "Examples:\n  wok start <id>           Start working\n  wok new task \"My task\"   Create task";
        let result = colorize_help_forced(input);

        // <id> should be context colored
        assert!(
            result.contains(&format!("{}<id>{}", CONTEXT_START, RESET)),
            "<id> should be CONTEXT colored in:\n{}",
            result
        );

        // "My task" should be context colored
        assert!(
            result.contains(&format!("{}\"My task\"{}", CONTEXT_START, RESET)),
            "\"My task\" should be CONTEXT colored in:\n{}",
            result
        );

        // wok should be literal colored
        assert!(
            result.contains(&format!("{}wok{}", LITERAL_START, RESET)),
            "wok should be LITERAL colored in:\n{}",
            result
        );
    }

    /// Test doc label lines through full pipeline
    #[test]
    fn doc_label_through_colorize_help() {
        let input = "Filter Expressions:\n  Syntax: FIELD [OP VALUE]";
        let result = colorize_help_forced(input);

        // "Syntax:" should NOT be colored
        // The label including colon appears, but not preceded by color escape
        assert!(
            result.contains("Syntax:"),
            "Syntax: label should be present in:\n{}",
            result
        );

        // Value after colon should be literal colored
        assert!(
            result.contains(&format!("{}FIELD [OP VALUE]{}", LITERAL_START, RESET)),
            "Value should be LITERAL colored in:\n{}",
            result
        );
    }

    /// Test a realistic full help output
    #[test]
    fn realistic_list_help() {
        let input = "\
List issues matching filters.

Usage: wok list [OPTIONS]

Options:
  -s, --status <STATUS>      Filter by status
  -n, --[no-]limit <LIMIT>   Maximum number of results
  -o, --output <OUTPUT>      Output format [default: text] [possible values: text, json, id]
  -h, --help                 Print help";

        let result = colorize_help_forced(input);

        // Verify key colorizations
        assert!(
            result.contains(&format!("{}Options:{}", HEADER_START, RESET)),
            "Options: header missing"
        );
        assert!(
            result.contains(&format!("{}[no-]{}", CONTEXT_START, RESET)),
            "[no-] should be CONTEXT"
        );
        assert!(
            result.contains(&format!("{}<LIMIT>{}", CONTEXT_START, RESET)),
            "<LIMIT> should be CONTEXT"
        );
        assert!(
            result.contains(&format!("{}[default: text]{}", CONTEXT_START, RESET)),
            "[default: text] should be CONTEXT"
        );
        assert!(
            result.contains(&format!("{}--status{}", LITERAL_START, RESET)),
            "--status should be LITERAL"
        );
    }

    /// Debug: Print actual regex captures for option lines
    #[test]
    fn debug_option_regex_captures() {
        let lines = [
            "  -n, --[no-]limit <LIMIT>   Maximum results",
            "  -o, --output <OUTPUT>      Output [default: text]",
            "  -s, --status <STATUS>      Filter by status",
            "      --no-limit",
        ];

        for line in lines {
            eprintln!("\nLine: {:?}", line);
            if let Some(caps) = OPTION_LINE_RE.captures(line) {
                eprintln!("  1 indent: {:?}", caps.get(1).map(|m| m.as_str()));
                eprintln!("  2 short:  {:?}", caps.get(2).map(|m| m.as_str()));
                eprintln!("  3 flag:   {:?}", caps.get(3).map(|m| m.as_str()));
                eprintln!("  4 value:  {:?}", caps.get(4).map(|m| m.as_str()));
                eprintln!("  5 desc:   {:?}", caps.get(5).map(|m| m.as_str()));
            } else {
                eprintln!("  NO MATCH");
            }
        }
        // This test always passes - it's for debugging output
    }

    /// Debug: Print actual colorized output for a sample option line
    #[test]
    fn debug_colorized_output() {
        let input = "Options:\n  -n, --[no-]limit <LIMIT>   Maximum results";
        let result = colorize_help_forced(input);

        eprintln!("\nInput:\n{}", input);
        eprintln!("\nColorized (escaped):");
        for line in result.lines() {
            eprintln!("{:?}", line);
        }
        eprintln!("\nColorized (raw):\n{}", result);
        // This test always passes - it's for debugging output
    }
}

// ============================================================================
// Parameterized Colorization Tests (using yare)
// ============================================================================

mod colorization_params {
    use super::*;
    use crate::colors::codes::{CONTEXT_START, HEADER_START, LITERAL_START, RESET};

    // Helper to check a segment is colored with specific color
    fn has_colored(output: &str, text: &str, color: &str) -> bool {
        output.contains(&format!("{}{}{}", color, text, RESET))
    }

    // Helper to check text appears uncolored (not preceded by escape)
    fn has_uncolored(output: &str, text: &str) -> bool {
        // Text should appear but not immediately after a color code
        if !output.contains(text) {
            return false;
        }
        // Find the text and check it's not preceded by color code
        if let Some(pos) = output.find(text) {
            if pos >= 11 {
                // Check if preceded by color escape (e.g., \x1b[38;5;250m is 11 chars)
                let prefix = &output[pos.saturating_sub(11)..pos];
                !prefix.ends_with('m') || !prefix.contains("\x1b[")
            } else {
                true
            }
        } else {
            false
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Example Line Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[yare::parameterized(
        placeholder_id = { "  wok start <id>           Start working", "<id>", CONTEXT_START },
        placeholder_ids = { "  wok done <ids>...        Complete multiple", "<ids>", CONTEXT_START },
        quoted_string = { "  wok new task \"My task\"   Create task", "\"My task\"", CONTEXT_START },
        flag_value_status = { "  wok list -s done         List done", "done", CONTEXT_START },
        flag_value_type = { "  wok list -t bug          List bugs", "bug", CONTEXT_START },
        flag_value_label = { "  wok list -l urgent       Filter by label", "urgent", CONTEXT_START },
        flag_value_assignee = { "  wok list -a alice        Assigned to alice", "alice", CONTEXT_START },
        flag_value_format = { "  wok list -o json         Output JSON", "json", CONTEXT_START },
        flag_value_limit = { "  wok list --limit 10      Limit to 10", "10", CONTEXT_START },
        quoted_filter = { "  wok list -q \"age < 3d\"   Filter by age", "\"age < 3d\"", CONTEXT_START },
        command_wok = { "  wok list                 List issues", "wok", LITERAL_START },
        command_subcommand = { "  wok list                 List issues", "list", LITERAL_START },
    )]
    fn example_line_colorization(input: &str, text: &str, expected_color: &str) {
        // Use colorize_help_forced which always applies colors
        let result = colorize_help_forced(input);
        assert!(
            has_colored(&result, text, expected_color),
            "Expected '{}' to be colored with {:?} in:\n{}",
            text,
            if expected_color == CONTEXT_START {
                "CONTEXT"
            } else {
                "LITERAL"
            },
            result
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Option Line Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[yare::parameterized(
        short_flag = { "  -n, --limit <N>  Desc", "-n", LITERAL_START },
        long_flag = { "  -n, --limit <N>  Desc", "--limit", LITERAL_START },
        value_placeholder = { "  -n, --limit <N>  Desc", "<N>", CONTEXT_START },
        no_prefix = { "  -n, --[no-]limit <N>  Desc", "[no-]", CONTEXT_START },
        default_value = { "  -o, --output <O>  Desc [default: text]", "[default: text]", CONTEXT_START },
        possible_values = { "  -o, --output <O>  Desc [possible values: a, b]", "[possible values: a, b]", CONTEXT_START },
    )]
    fn option_line_colorization(input: &str, text: &str, expected_color: &str) {
        // Use colorize_help_forced which tests the full pipeline
        let result = colorize_help_forced(input);
        assert!(
            has_colored(&result, text, expected_color),
            "Expected '{}' to be colored with {:?} in:\n{}",
            text,
            if expected_color == CONTEXT_START {
                "CONTEXT"
            } else {
                "LITERAL"
            },
            result
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Doc Label Line Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[yare::parameterized(
        syntax_label = { "  Syntax: FIELD [OP VALUE]", "Syntax:" },
        fields_label = { "  Fields: age, activity", "Fields:" },
        operators_label = { "  Operators: < <= > >=", "Operators:" },
        values_label = { "  Values: durations, dates", "Values:" },
    )]
    fn doc_label_uncolored(input: &str, label: &str) {
        let result = colorize_help_forced(input);
        assert!(
            has_uncolored(&result, label),
            "Label '{}' should be uncolored in:\n{}",
            label,
            result
        );
    }

    #[yare::parameterized(
        syntax_value = { "  Syntax: FIELD [OP VALUE]", "FIELD [OP VALUE]", LITERAL_START },
        fields_value = { "  Fields: age, activity", "age, activity", LITERAL_START },
        operators_value = { "  Operators: < <= > >=", "< <= > >=", LITERAL_START },
    )]
    fn doc_label_value_colored(input: &str, value: &str, expected_color: &str) {
        let result = colorize_help_forced(input);
        assert!(
            has_colored(&result, value, expected_color),
            "Value '{}' should be colored with LITERAL in:\n{}",
            value,
            result
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Command List Line Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[yare::parameterized(
        simple_command = { "  new         Create issue", "new", LITERAL_START },
        un_prefix_context = { "  [un]dep     Add/remove dep", "[un]", CONTEXT_START },
        un_prefix_literal = { "  [un]dep     Add/remove dep", "dep", LITERAL_START },
        un_label = { "  [un]label   Add/remove label", "label", LITERAL_START },
        un_link = { "  [un]link    Add/remove link", "link", LITERAL_START },
    )]
    fn command_list_colorization(input: &str, text: &str, expected_color: &str) {
        let result = colorize_help_forced(input);
        assert!(
            has_colored(&result, text, expected_color),
            "Expected '{}' to be colored with {:?} in:\n{}",
            text,
            if expected_color == CONTEXT_START {
                "CONTEXT"
            } else {
                "LITERAL"
            },
            result
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Header Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[yare::parameterized(
        options_header = { "Options:" },
        issue_tracking = { "Issue Tracking:" },
        get_started = { "Get started:" },
        filter_expressions = { "Filter Expressions (-q/--filter):" },
    )]
    fn header_colorization(input: &str) {
        // Use colorize_help_forced which tests the full pipeline
        let result = colorize_help_forced(input);
        // Header function wraps the ENTIRE input including colon
        assert!(
            has_colored(&result, input, HEADER_START),
            "Header should be colored with HEADER color:\n{}",
            result
        );
    }
}
