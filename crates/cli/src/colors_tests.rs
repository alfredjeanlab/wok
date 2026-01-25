// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;

// =============================================================================
// Helper functions for testing
// =============================================================================

/// Strip all ANSI escape sequences from a string
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm'
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Expected escape sequence for a color code
fn expected_fg(code: u8) -> String {
    format!("\x1b[38;5;{}m", code)
}

// =============================================================================
// Color code constants
// =============================================================================

#[test]
fn color_codes_match_v0_conventions() {
    assert_eq!(codes::HEADER, 74, "Header should be pastel cyan/steel blue");
    assert_eq!(codes::LITERAL, 250, "Literal should be light grey");
    assert_eq!(codes::CONTEXT, 245, "Context should be medium grey");
}

#[test]
fn fg256_produces_correct_escape_sequence() {
    assert_eq!(fg256(0), "\x1b[38;5;0m");
    assert_eq!(fg256(74), "\x1b[38;5;74m");
    assert_eq!(fg256(245), "\x1b[38;5;245m");
    assert_eq!(fg256(250), "\x1b[38;5;250m");
    assert_eq!(fg256(255), "\x1b[38;5;255m");
}

#[test]
fn reset_sequence_is_correct() {
    assert_eq!(RESET, "\x1b[0m");
}

// =============================================================================
// find_description_start
// =============================================================================

#[test]
fn find_description_start_with_two_spaces() {
    assert_eq!(find_description_start("cmd  desc"), Some(3));
    assert_eq!(find_description_start("wok init  Initialize"), Some(8));
}

#[test]
fn find_description_start_with_many_spaces() {
    assert_eq!(find_description_start("cmd     desc"), Some(3));
    assert_eq!(
        find_description_start("wok list --all   List all"),
        Some(14)
    );
}

#[test]
fn find_description_start_single_space_returns_none() {
    assert_eq!(find_description_start("cmd desc"), None);
    assert_eq!(find_description_start("wok init"), None);
    assert_eq!(find_description_start("just some words here"), None);
}

#[test]
fn find_description_start_empty_input() {
    assert_eq!(find_description_start(""), None);
}

#[test]
fn find_description_start_only_spaces() {
    assert_eq!(find_description_start("   "), None);
    assert_eq!(find_description_start("      "), None);
}

#[test]
fn find_description_start_trailing_spaces() {
    // Trailing spaces without content after shouldn't match
    // (there's no description to start)
    assert_eq!(find_description_start("cmd  "), None);
    assert_eq!(find_description_start("cmd    "), None);
}

#[test]
fn find_description_start_at_various_positions() {
    assert_eq!(find_description_start("a  b"), Some(1));
    assert_eq!(find_description_start("ab  cd"), Some(2));
    assert_eq!(find_description_start("abc  def"), Some(3));
    assert_eq!(find_description_start("a b c  d"), Some(5));
}

// =============================================================================
// colorize_command - structure tests
// =============================================================================

#[test]
fn colorize_command_simple_command() {
    let result = colorize_command("wok list");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok list");
}

#[test]
fn colorize_command_with_quoted_string() {
    let result = colorize_command(r#"wok new "Fix bug""#);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, r#"wok new "Fix bug""#);

    // When colors enabled, quotes should have context color
    if should_colorize() {
        assert!(result.contains(&expected_fg(codes::CONTEXT)));
        assert!(result.contains("\"Fix bug\""));
    }
}

#[test]
fn colorize_command_with_flag_and_value() {
    let result = colorize_command("wok list -s done");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok list -s done");

    // When colors enabled, flag value should have context color
    if should_colorize() {
        // "done" should be colored as context (after -s flag)
        assert!(result.contains(&expected_fg(codes::CONTEXT)));
    }
}

#[test]
fn colorize_command_with_long_flag_and_value() {
    let result = colorize_command("wok list --status done");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok list --status done");
}

#[test]
fn colorize_command_multiple_flags() {
    let result = colorize_command("wok list -s done -t bug");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok list -s done -t bug");
}

#[test]
fn colorize_command_quoted_then_flag() {
    let result = colorize_command(r#"wok search "auth" -s todo"#);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, r#"wok search "auth" -s todo"#);
}

#[test]
fn colorize_command_flag_with_equals() {
    // Flags with = should not trigger flag value coloring for next word
    let result = colorize_command("wok list --format=json next");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok list --format=json next");
}

#[test]
fn colorize_command_empty_string() {
    let result = colorize_command("");
    assert_eq!(result, "");
}

#[test]
fn colorize_command_preserves_multiple_spaces() {
    // Should preserve structure even with unusual spacing
    let result = colorize_command("wok  list");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok  list");
}

#[test]
fn colorize_command_unclosed_quote() {
    // Unclosed quote should still work (quote extends to end)
    let result = colorize_command(r#"wok new "unclosed"#);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, r#"wok new "unclosed"#);
}

#[test]
fn colorize_command_empty_quotes() {
    let result = colorize_command(r#"wok new """#);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, r#"wok new """#);
}

#[test]
fn colorize_command_multiple_quoted_strings() {
    let result = colorize_command(r#"wok cmd "first" "second""#);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, r#"wok cmd "first" "second""#);
}

#[test]
fn colorize_command_with_placeholder() {
    let result = colorize_command("wok start <id>");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok start <id>");

    // When colors enabled, placeholder should have context color
    if should_colorize() {
        assert!(result.contains(&expected_fg(codes::CONTEXT)));
    }
}

#[test]
fn colorize_command_multiple_placeholders() {
    let result = colorize_command("wok dep <blocker> blocks <blocked>");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok dep <blocker> blocks <blocked>");
}

#[test]
fn colorize_command_placeholder_and_quoted() {
    let result = colorize_command(r#"wok new <type> "title""#);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, r#"wok new <type> "title""#);
}

#[test]
fn colorize_command_unclosed_placeholder() {
    // Unclosed angle bracket should still work (extends to end)
    let result = colorize_command("wok start <id");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok start <id");
}

// =============================================================================
// Color helper functions
// =============================================================================

#[test]
fn header_contains_text() {
    let result = header("Examples:");
    assert!(result.contains("Examples:"));
    assert_eq!(strip_ansi(&result), "Examples:");
}

#[test]
fn literal_contains_text() {
    let result = literal("wok list");
    assert!(result.contains("wok list"));
    assert_eq!(strip_ansi(&result), "wok list");
}

#[test]
fn context_contains_text() {
    let result = context("value");
    assert!(result.contains("value"));
    assert_eq!(strip_ansi(&result), "value");
}

#[test]
fn header_with_color_has_correct_codes() {
    if should_colorize() {
        let result = header("Test:");
        assert!(result.starts_with(&expected_fg(codes::HEADER)));
        assert!(result.ends_with(RESET));
    }
}

#[test]
fn literal_with_color_has_correct_codes() {
    if should_colorize() {
        let result = literal("cmd");
        assert!(result.starts_with(&expected_fg(codes::LITERAL)));
        assert!(result.ends_with(RESET));
    }
}

#[test]
fn context_with_color_has_correct_codes() {
    if should_colorize() {
        let result = context("val");
        assert!(result.starts_with(&expected_fg(codes::CONTEXT)));
        assert!(result.ends_with(RESET));
    }
}

// =============================================================================
// examples() function - structure tests
// =============================================================================

#[test]
fn examples_header_line() {
    let input = "Examples:";
    let result = examples(input);
    assert!(result.contains("Examples:"));
    assert_eq!(strip_ansi(&result), "Examples:");
}

#[test]
fn examples_header_with_parens() {
    let input = "Filter Expressions (-q/--filter):";
    let result = examples(input);
    assert_eq!(strip_ansi(&result), input);
}

#[test]
fn examples_command_line() {
    let input = "  wok list  List issues";
    let result = examples(input);
    assert_eq!(strip_ansi(&result), input);
}

#[test]
fn examples_documentation_line() {
    let input = "  Syntax: FIELD OP VALUE";
    let result = examples(input);
    assert_eq!(strip_ansi(&result), input);
}

#[test]
fn examples_plain_line_no_pattern() {
    let input = "  This is just plain text";
    let result = examples(input);
    assert_eq!(result, input); // Should be unchanged
}

#[test]
fn examples_empty_input() {
    let result = examples("");
    assert_eq!(result, "");
}

#[test]
fn examples_blank_lines_preserved() {
    let input = "Examples:\n\n  wok list  List";
    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert!(stripped.contains("\n\n"));
}

#[test]
fn examples_multiline_structure() {
    let input = "\
Examples:
  wok init  Initialize
  wok list  List issues

Filter Expressions:
  Syntax: FIELD OP VALUE
  Fields: age, activity";

    let result = examples(input);
    let stripped = strip_ansi(&result);

    // Verify structure preserved
    assert_eq!(stripped, input);

    // Verify line count preserved
    assert_eq!(result.lines().count(), input.lines().count());
}

#[test]
fn examples_indentation_preserved() {
    let input = "    deeply indented  desc";
    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert!(stripped.starts_with("    "));
}

#[test]
fn examples_mixed_content() {
    let input = "\
Examples:
  wok search \"login\"  Search for login
  wok list -s done  Show done issues

Filter Expressions (-q/--filter):
  Syntax: FIELD [OPERATOR VALUE]
  Fields: age, activity, completed";

    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, input);
}

// =============================================================================
// Edge cases and robustness
// =============================================================================

#[test]
fn examples_colon_in_command() {
    // Colon in the command part shouldn't trigger doc line detection
    let input = "  wok config remote ws://host:7890  Enable WebSocket";
    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, input);
}

#[test]
fn examples_quoted_colon() {
    let input = r#"  wok new "Fix: bug"  Create issue"#;
    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, input);
}

#[test]
fn examples_special_characters_in_command() {
    let input = "  wok list -q \"age < 3d\"  Filter recent";
    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, input);
}

#[test]
fn examples_unicode_content() {
    let input = "  wok new \"Fix emoji: 🐛\"  Create bug";
    let result = examples(input);
    assert!(result.contains("🐛"));
}

#[test]
fn colorize_command_special_chars() {
    let result = colorize_command("wok list -q \"age < 3d\"");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok list -q \"age < 3d\"");
}

#[test]
fn colorize_command_path_argument() {
    let result = colorize_command("wok init --remote ~/tracker");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok init --remote ~/tracker");
}

#[test]
fn colorize_command_url_argument() {
    let result = colorize_command("wok link id https://github.com/org/repo");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "wok link id https://github.com/org/repo");
}

// =============================================================================
// Regression tests
// =============================================================================

#[test]
fn regression_alignment_preserved_after_colorization() {
    // Verify that colorization doesn't change the visual alignment
    // when ANSI codes are stripped
    let input = "\
  wok init                          Initialize
  wok init --prefix myproj          Custom prefix
  wok init --remote ws://host:7890  WebSocket sync";

    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, input);
}

#[test]
fn regression_double_space_detection_accurate() {
    // Ensure we detect exactly 2+ spaces, not just any whitespace
    let with_two = "cmd  desc";
    let with_one = "cmd desc";
    let with_tab = "cmd\tdesc";

    assert!(find_description_start(with_two).is_some());
    assert!(find_description_start(with_one).is_none());
    assert!(find_description_start(with_tab).is_none());
}
