// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Terminal color utilities for help output.
//!
//! Respects environment variables:
//! - `NO_COLOR=1`: Disables colors
//! - `COLOR=1`: Forces colors even without TTY

use std::io::IsTerminal;

/// ANSI 256-color codes matching v0 help conventions
pub mod codes {
    /// Section headers: pastel cyan/steel blue
    pub const HEADER: u8 = 74;
    /// Commands/literals: light grey
    pub const LITERAL: u8 = 250;
    /// Default values/context: medium grey
    pub const CONTEXT: u8 = 245;

    /// Pre-formatted ANSI escape sequences for use in tests
    pub const HEADER_START: &str = "\x1b[38;5;74m";
    pub const LITERAL_START: &str = "\x1b[38;5;250m";
    pub const CONTEXT_START: &str = "\x1b[38;5;245m";
    pub const RESET: &str = "\x1b[0m";
}

/// Check if colors should be enabled based on TTY and environment variables.
pub fn should_colorize() -> bool {
    // NO_COLOR=1 disables colors
    if std::env::var("NO_COLOR").is_ok_and(|v| v == "1") {
        return false;
    }

    // COLOR=1 forces colors even without TTY
    if std::env::var("COLOR").is_ok_and(|v| v == "1") {
        return true;
    }

    // Default: enable colors only if stdout is a TTY
    std::io::stdout().is_terminal()
}

/// Format a 256-color ANSI escape sequence for foreground color.
fn fg256(code: u8) -> String {
    format!("\x1b[38;5;{code}m")
}

/// ANSI reset sequence.
const RESET: &str = "\x1b[0m";

/// Apply header color (section titles) to text.
pub fn header(text: &str) -> String {
    format!("{}{}{}", fg256(codes::HEADER), text, RESET)
}

/// Apply literal color (commands, options) to text.
pub fn literal(text: &str) -> String {
    format!("{}{}{}", fg256(codes::LITERAL), text, RESET)
}

/// Apply context color (default values, hints) to text.
pub fn context(text: &str) -> String {
    format!("{}{}{}", fg256(codes::CONTEXT), text, RESET)
}

/// Colorize an examples help block.
///
/// Expects format like:
/// ```text
/// Examples:
///   wok command args    Description here
///   wok other args      Another description
///
/// Filter Expressions:
///   Syntax: FIELD [OPERATOR VALUE]
///   Fields: age, activity, completed
/// ```
///
/// Colorizes:
/// - Section headers (lines ending with `:`) as header color
/// - Commands (before `  `) as literal color
/// - Documentation labels (e.g., "Syntax:") as literal, values as context
pub fn examples(text: &str) -> String {
    if !should_colorize() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len() + 256);

    for line in text.lines() {
        if !result.is_empty() {
            result.push('\n');
        }

        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];

        // Header line (e.g., "Examples:", "Filter Expressions (-q/--filter):")
        if trimmed.ends_with(':') && !trimmed.contains("  ") {
            result.push_str(indent);
            result.push_str(&header(trimmed));
            continue;
        }

        // Example line: "  wok command args    Description"
        // Find where command ends (multiple spaces before description)
        if let Some(cmd_end) = find_description_start(trimmed) {
            let cmd = &trimmed[..cmd_end];
            let desc = &trimmed[cmd_end..];
            result.push_str(indent);
            result.push_str(&colorize_command(cmd));
            result.push_str(desc);
            continue;
        }

        // Documentation line: "Label: value" (e.g., "Syntax: FIELD [OPERATOR VALUE]")
        // Label stays uncolored, value colored as literal
        if let Some(colon_pos) = trimmed.find(": ") {
            let label = &trimmed[..=colon_pos]; // Include the colon
            let value = &trimmed[colon_pos + 2..]; // Skip ": " to get value
            result.push_str(indent);
            result.push_str(label);
            result.push(' ');
            result.push_str(&literal(value));
            continue;
        }

        // No pattern matched, output as-is
        result.push_str(line);
    }

    result
}

/// Colorize a command string, highlighting quoted content, placeholders, and flag values as context.
pub fn colorize_command(cmd: &str) -> String {
    let mut result = String::with_capacity(cmd.len() + 128);
    let mut chars = cmd.char_indices().peekable();
    let mut current_word_start = 0;
    let mut in_flag_value = false;

    while let Some((i, c)) = chars.next() {
        match c {
            '"' => {
                // Flush any pending literal content before the quote
                if i > current_word_start {
                    let before = &cmd[current_word_start..i];
                    if in_flag_value {
                        result.push_str(&context(before));
                        in_flag_value = false;
                    } else {
                        result.push_str(&literal(before));
                    }
                }

                // Find closing quote
                let quote_start = i;
                let mut quote_end = cmd.len();
                for (j, ch) in chars.by_ref() {
                    if ch == '"' {
                        quote_end = j + 1;
                        break;
                    }
                }
                result.push_str(&context(&cmd[quote_start..quote_end]));
                current_word_start = quote_end;
            }
            '<' => {
                // Flush any pending literal content before the angle bracket
                if i > current_word_start {
                    let before = &cmd[current_word_start..i];
                    if in_flag_value {
                        result.push_str(&context(before));
                        in_flag_value = false;
                    } else {
                        result.push_str(&literal(before));
                    }
                }

                // Find closing angle bracket for placeholder like <id>
                let bracket_start = i;
                let mut bracket_end = cmd.len();
                for (j, ch) in chars.by_ref() {
                    if ch == '>' {
                        bracket_end = j + 1;
                        break;
                    }
                }
                result.push_str(&context(&cmd[bracket_start..bracket_end]));
                current_word_start = bracket_end;
            }
            ' ' => {
                // Flush current segment
                if i > current_word_start {
                    let segment = &cmd[current_word_start..i];
                    if in_flag_value {
                        result.push_str(&context(segment));
                        in_flag_value = false;
                    } else {
                        result.push_str(&literal(segment));
                        // Check if this segment is a flag (starts with -)
                        if segment.starts_with('-') && !segment.contains('=') {
                            in_flag_value = true;
                        }
                    }
                }
                result.push(' ');
                current_word_start = i + 1;
            }
            _ => {}
        }
    }

    // Flush remaining content
    if current_word_start < cmd.len() {
        let remaining = &cmd[current_word_start..];
        if in_flag_value {
            result.push_str(&context(remaining));
        } else {
            result.push_str(&literal(remaining));
        }
    }

    result
}

/// Find where the description starts (after 2+ spaces following the command).
pub fn find_description_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_spaces = false;
    let mut space_start = 0;

    while i < bytes.len() {
        if bytes[i] == b' ' {
            if !in_spaces {
                in_spaces = true;
                space_start = i;
            }
        } else {
            if in_spaces && i - space_start >= 2 {
                // Found 2+ spaces, command ends at space_start
                return Some(space_start);
            }
            in_spaces = false;
        }
        i += 1;
    }

    None
}

#[cfg(test)]
#[path = "colors_tests.rs"]
mod tests;
