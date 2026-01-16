// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use yare::parameterized;

#[parameterized(
    empty = { "", "" },
    no_trim_needed = { "hello", "hello" },
    leading_space = { "  hello", "hello" },
    trailing_space = { "hello  ", "hello" },
    both_spaces = { "  hello  ", "hello" },
    leading_tab = { "\thello", "hello" },
    trailing_tab = { "hello\t", "hello" },
    leading_newline = { "\nhello", "hello" },
    trailing_newline = { "hello\n", "hello" },
    mixed_whitespace = { " \t\n hello \n\t ", "hello" },
    internal_preserved = { "hello world", "hello world" },
    internal_newline_preserved = { "hello\nworld", "hello\nworld" },
    only_whitespace = { "   \t\n  ", "" },
    unicode_spaces = { "\u{00A0}hello\u{00A0}", "hello" },
)]
fn test_trim_field(input: &str, expected: &str) {
    assert_eq!(trim_field(input), expected);
}

#[parameterized(
    empty = { "", "", None },
    simple = { "hello", "hello", None },
    trim_leading = { "  hello", "hello", None },
    trim_trailing = { "hello  ", "hello", None },
    trim_both = { "  hello  ", "hello", None },
)]
fn test_normalize_title_trimming(input: &str, exp_title: &str, exp_desc: Option<&str>) {
    let result = normalize_title(input);
    assert_eq!(result.title, exp_title);
    assert_eq!(result.extracted_description.as_deref(), exp_desc);
}

#[parameterized(
    single_newline = { "hello\nworld", "hello world", None },
    crlf = { "hello\r\nworld", "hello world", None },
    cr_only = { "hello\rworld", "hello world", None },
    multiple_newlines = { "a\nb\nc", "a b c", None },
    newline_at_word_boundary = { "hello\n world", "hello world", None },
)]
fn test_normalize_title_newline_to_space(input: &str, exp_title: &str, exp_desc: Option<&str>) {
    let result = normalize_title(input);
    assert_eq!(result.title, exp_title);
    assert_eq!(result.extracted_description.as_deref(), exp_desc);
}

#[parameterized(
    double_space = { "hello  world", "hello world", None },
    triple_space = { "hello   world", "hello world", None },
    tab = { "hello\tworld", "hello world", None },
    mixed = { "hello \t \n world", "hello world", None },
    multiple_gaps = { "a  b   c    d", "a b c d", None },
)]
fn test_normalize_title_whitespace_collapse(input: &str, exp_title: &str, exp_desc: Option<&str>) {
    let result = normalize_title(input);
    assert_eq!(result.title, exp_title);
    assert_eq!(result.extracted_description.as_deref(), exp_desc);
}

#[parameterized(
    three_words_split = {
        "Fix the bug\n\nDetailed description here",
        "Fix the bug",
        Some("Detailed description here")
    },
    twenty_chars_split = {
        "A very long title!!\n\nDescription",
        "A very long title!!",
        Some("Description")
    },
    one_word_no_split = { "Hi\n\nthere", "Hi there", None },
    two_words_no_split = { "Hello world\n\nmore", "Hello world more", None },
    short_no_split = { "Short\n\ntext", "Short text", None },
    exactly_three_words = {
        "One two three\n\nfour five",
        "One two three",
        Some("four five")
    },
    exactly_twenty_chars = {
        "12345678901234567890\n\nrest",
        "12345678901234567890",
        Some("rest")
    },
    multiple_splits = {
        "Fix the bug\n\nPart one\n\nPart two",
        "Fix the bug",
        Some("Part one\n\nPart two")
    },
    description_needs_trim = {
        "Fix the bug\n\n  Desc with spaces  ",
        "Fix the bug",
        Some("Desc with spaces")
    },
)]
fn test_normalize_title_splitting(input: &str, exp_title: &str, exp_desc: Option<&str>) {
    let result = normalize_title(input);
    assert_eq!(
        result.title, exp_title,
        "title mismatch for input: {:?}",
        input
    );
    assert_eq!(
        result.extracted_description.as_deref(),
        exp_desc,
        "desc mismatch for input: {:?}",
        input
    );
}

#[parameterized(
    double_quote_newline = {
        "Error: \"line1\nline2\"",
        "Error: \"line1\\nline2\"",
        None
    },
    single_quote_newline = {
        "Error: 'line1\nline2'",
        "Error: 'line1\\nline2'",
        None
    },
    backtick_newline = {
        "Code: `a\nb`",
        "Code: `a\\nb`",
        None
    },
    curly_double = {
        "Says \u{201C}hello\nworld\u{201D}",
        "Says \u{201C}hello\\nworld\u{201D}",
        None
    },
    curly_single = {
        "It\u{2019}s \u{2018}test\nval\u{2019}",
        "It\u{2019}s \u{2018}test\\nval\u{2019}",
        None
    },
    mixed_content = {
        "Fix  \"error\nmsg\"  in   module",
        "Fix \"error\\nmsg\" in module",
        None
    },
    empty_quotes = {
        "Value: \"\"",
        "Value: \"\"",
        None
    },
    quote_at_start = {
        "\"quoted\" unquoted",
        "\"quoted\" unquoted",
        None
    },
    quote_at_end = {
        "unquoted \"quoted\"",
        "unquoted \"quoted\"",
        None
    },
    unclosed_quote = {
        "Error: \"unclosed",
        "Error: \"unclosed\"",
        None
    },
    adjacent_quotes = {
        "\"one\"\"two\"",
        "\"one\"\"two\"",
        None
    },
)]
fn test_normalize_title_quoted(input: &str, exp_title: &str, exp_desc: Option<&str>) {
    let result = normalize_title(input);
    assert_eq!(
        result.title, exp_title,
        "title mismatch for input: {:?}",
        input
    );
    assert_eq!(result.extracted_description.as_deref(), exp_desc);
}

#[parameterized(
    empty_string = { "", "", None },
    only_whitespace = { "   ", "", None },
    only_newlines = { "\n\n\n", "", None },
    single_double_newline = { "\n\n", "", None },
    unicode_content = {
        "Fix bug \u{1F41B} in module",
        "Fix bug \u{1F41B} in module",
        None
    },
    unicode_with_split = {
        "Fix bug \u{1F41B}\n\nDetailed \u{65E5}\u{672C}",
        "Fix bug \u{1F41B}",
        Some("Detailed \u{65E5}\u{672C}")
    },
    escaped_backslash_n = {
        "Already escaped: \\n",
        "Already escaped: \\n",
        None
    },
)]
fn test_normalize_title_edge_cases(input: &str, exp_title: &str, exp_desc: Option<&str>) {
    let result = normalize_title(input);
    assert_eq!(result.title, exp_title);
    assert_eq!(result.extracted_description.as_deref(), exp_desc);
}

#[test]
fn test_tokenize_simple() {
    let tokens = tokenize("hello world");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Unquoted(s) if s == "hello world"));
}

#[test]
fn test_tokenize_quoted() {
    let tokens = tokenize("a \"b\" c");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(&tokens[0], Token::Unquoted(s) if s == "a "));
    assert!(matches!(&tokens[1], Token::Quoted { quote: '"', content } if content == "b"));
    assert!(matches!(&tokens[2], Token::Unquoted(s) if s == " c"));
}

#[test]
fn test_tokenize_nested_quotes() {
    // "a 'b' c" - outer double, inner single (inner treated as content)
    let tokens = tokenize("\"a 'b' c\"");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Quoted { quote: '"', content } if content == "a 'b' c"));
}

#[test]
fn test_tokenize_empty() {
    let tokens = tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn test_tokenize_only_quotes() {
    let tokens = tokenize("\"\"");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Quoted { quote: '"', content } if content.is_empty()));
}

#[test]
fn test_tokenize_multiple_quoted() {
    let tokens = tokenize("'a' and \"b\"");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(&tokens[0], Token::Quoted { quote: '\'', content } if content == "a"));
    assert!(matches!(&tokens[1], Token::Unquoted(s) if s == " and "));
    assert!(matches!(&tokens[2], Token::Quoted { quote: '"', content } if content == "b"));
}

#[parameterized(
    empty = { "", false },
    one_word = { "hello", false },
    two_words = { "hello world", false },
    three_words = { "one two three", true },
    four_words = { "one two three four", true },
    nineteen_chars = { "1234567890123456789", false },
    twenty_chars = { "12345678901234567890", true },
    twenty_one_chars = { "123456789012345678901", true },
    two_long_words = { "longword1 longword2!", true },
)]
fn test_is_past_threshold(input: &str, expected: bool) {
    assert_eq!(
        is_past_threshold(input),
        expected,
        "threshold check for: {:?}",
        input
    );
}

#[parameterized(
    no_newlines = { "hello", "hello" },
    single_lf = { "a\nb", "a\\nb" },
    single_cr = { "a\rb", "a\\rb" },
    crlf = { "a\r\nb", "a\\r\\nb" },
    multiple = { "a\nb\nc", "a\\nb\\nc" },
    already_escaped = { "a\\nb", "a\\nb" },
)]
fn test_escape_newlines(input: &str, expected: &str) {
    assert_eq!(escape_newlines(input), expected);
}

#[parameterized(
    no_whitespace = { "hello", "hello" },
    single_space = { "hello world", "hello world" },
    double_space = { "hello  world", "hello world" },
    tabs = { "hello\tworld", "hello world" },
    newlines = { "hello\nworld", "hello world" },
    mixed = { "a \t\n b", "a b" },
    leading = { "  hello", " hello" },
    trailing = { "hello  ", "hello " },
)]
fn test_collapse_whitespace(input: &str, expected: &str) {
    assert_eq!(collapse_whitespace(input), expected);
}

#[test]
fn test_closing_quote_ascii() {
    assert_eq!(closing_quote('"'), '"');
    assert_eq!(closing_quote('\''), '\'');
    assert_eq!(closing_quote('`'), '`');
}

#[test]
fn test_closing_quote_typographic() {
    assert_eq!(closing_quote('\u{201C}'), '\u{201D}'); // " -> "
    assert_eq!(closing_quote('\u{2018}'), '\u{2019}'); // ' -> '
}

#[test]
fn test_is_opening_quote() {
    assert!(is_opening_quote('"'));
    assert!(is_opening_quote('\''));
    assert!(is_opening_quote('`'));
    assert!(is_opening_quote('\u{201C}')); // "
    assert!(is_opening_quote('\u{201D}')); // "
    assert!(is_opening_quote('\u{2018}')); // '
    assert!(is_opening_quote('\u{2019}')); // '
    assert!(!is_opening_quote('a'));
    assert!(!is_opening_quote(' '));
}

#[test]
fn test_find_split_point_none() {
    assert!(find_split_point("no double newline").is_none());
    assert!(find_split_point("Hi\n\nthere").is_none()); // below threshold
}

#[test]
fn test_find_split_point_found() {
    let result = find_split_point("Fix the bug\n\ndescription");
    assert!(result.is_some());
    let (title, desc) = result.unwrap();
    assert_eq!(title, "Fix the bug");
    assert_eq!(desc, "description");
}

#[test]
fn test_find_split_point_multiple() {
    // Should split at first valid position
    let result = find_split_point("Fix the bug\n\npart one\n\npart two");
    assert!(result.is_some());
    let (title, desc) = result.unwrap();
    assert_eq!(title, "Fix the bug");
    assert_eq!(desc, "part one\n\npart two");
}
