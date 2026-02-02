// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Help text generation with colorization support.
//!
//! This module provides help output formatting that:
//! - Consolidates `--flag` and `--no-flag` pairs into `--[no-]flag` format
//! - Applies v0 color conventions consistently
//!
//! The consolidation works by:
//! 1. Using `Styles::plain()` so clap generates uncolored output
//! 2. Parsing option lines with regex to find `--flag`/`--no-flag` pairs
//! 3. Merging them into `--[no-]flag` format
//! 4. Applying colors manually to the final output

use std::io::Write;
use std::sync::LazyLock;

use clap::builder::styling::Styles;
use clap::Command;
use regex::Regex;

use crate::colors;

/// Regex to parse option lines in help output.
/// Captures: 1=indent, 2=short+comma, 3=long flag name, 4=value placeholder, 5=description
/// These are compile-time constant patterns that are verified at test time.
/// Using match with unreachable! since these patterns are hard-coded and known-valid.
static OPTION_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    match Regex::new(
        r"(?x)
        ^(\s*)                          # 1: indent
        ((?:-\w,\s+)?)                  # 2: optional short flag with comma and space
        --(\S+)                         # 3: long flag name (without --)
        (\s+<[^>]+>|\s+\[[^\]]+\])?     # 4: optional value placeholder
        (\s{2,}.+)?$                    # 5: description (preceded by 2+ spaces)
    ",
    ) {
        Ok(re) => re,
        Err(_) => unreachable!("static regex pattern"),
    }
});

/// Regex to detect `--no-X` flags.
static NO_FLAG_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(r"^no-(.+)$") {
    Ok(re) => re,
    Err(_) => unreachable!("static regex pattern"),
});

/// Format help output for a command with flag consolidation.
///
/// This is the main entry point for generating help text. It captures
/// clap's help output, consolidates negatable flags, and applies colors.
pub fn format_help(cmd: &mut Command) -> String {
    // Capture clap's help output
    // Writing to Vec<u8> and converting UTF-8 are infallible for clap help output
    let mut buf = Vec::new();
    match cmd.write_help(&mut buf) {
        Ok(()) => {}
        Err(_) => unreachable!("write_help to Vec<u8> is infallible"),
    }
    let raw_help = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => unreachable!("clap help output is always valid UTF-8"),
    };

    // Consolidate --flag/--no-flag pairs
    let consolidated = consolidate_negatable_flags(&raw_help);

    // Apply colors if enabled
    let output = if colors::should_colorize() {
        colorize_help_forced(&consolidated)
    } else {
        consolidated
    };

    // Ensure trailing newline for clean shell output (avoids zsh '%' marker)
    if output.ends_with('\n') {
        output
    } else {
        format!("{}\n", output)
    }
}

/// Print formatted help to stdout.
pub fn print_help(cmd: &mut Command) {
    let help = format_help(cmd);
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(help.as_bytes());
    let _ = stdout.flush();
}

/// Print formatted help to stderr (for usage errors).
pub fn eprint_help(cmd: &mut Command) {
    let help = format_help(cmd);
    let mut stderr = std::io::stderr();
    let _ = stderr.write_all(help.as_bytes());
    let _ = stderr.flush();
}

/// Consolidate `--flag` and `--no-flag` pairs into `--[no-]flag` format.
///
/// Scans the help text looking for adjacent option lines where one is `--no-X`
/// and the other is `--X`. Merges them into a single `--[no-]X` line.
fn consolidate_negatable_flags(text: &str) -> String {
    let option_re = &OPTION_LINE_RE;
    let no_re = &NO_FLAG_RE;

    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Try to match as an option line
        if let Some(caps) = option_re.captures(line) {
            let flag_name = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            // Check if this is a --no-X flag
            if let Some(no_caps) = no_re.captures(flag_name) {
                let base_name = no_caps.get(1).map(|m| m.as_str()).unwrap_or("");

                // Look at the next line to see if it's the positive flag
                if i + 1 < lines.len() {
                    if let Some(next_caps) = option_re.captures(lines[i + 1]) {
                        let next_flag = next_caps.get(3).map(|m| m.as_str()).unwrap_or("");

                        if next_flag == base_name {
                            // Found a pair: --no-X followed by --X
                            // Use the positive flag's line as the base, merge into --[no-]X
                            let merged = merge_flag_pair(&next_caps, base_name);
                            result.push(merged);
                            i += 2; // Skip both lines
                            continue;
                        }
                    }
                }
            }

            // Check if this is a positive flag with --no-X following
            if i + 1 < lines.len() {
                if let Some(next_caps) = option_re.captures(lines[i + 1]) {
                    let next_flag = next_caps.get(3).map(|m| m.as_str()).unwrap_or("");

                    if let Some(no_caps) = no_re.captures(next_flag) {
                        let base_name = no_caps.get(1).map(|m| m.as_str()).unwrap_or("");

                        if base_name == flag_name {
                            // Found a pair: --X followed by --no-X
                            // Use the positive flag's line as the base
                            let merged = merge_flag_pair(&caps, flag_name);
                            result.push(merged);
                            i += 2; // Skip both lines
                            continue;
                        }
                    }
                }
            }
        }

        // No consolidation, keep the line as-is
        result.push(line.to_string());
        i += 1;
    }

    result.join("\n")
}

/// Merge a flag pair into `--[no-]flag` format.
fn merge_flag_pair(positive_caps: &regex::Captures, flag_name: &str) -> String {
    let indent = positive_caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let short = positive_caps.get(2).map(|m| m.as_str()).unwrap_or("");
    let value = positive_caps.get(4).map(|m| m.as_str()).unwrap_or("");
    let desc = positive_caps.get(5).map(|m| m.as_str()).unwrap_or("");

    format!("{}{}--[no-]{}{}{}", indent, short, flag_name, value, desc)
}

/// Apply colors to help text unconditionally.
/// This is the main colorization function - always applies colors.
/// Used by format_help when colorization is enabled, and directly by tests.
pub fn colorize_help_forced(text: &str) -> String {
    let option_re = &OPTION_LINE_RE;
    let mut result = Vec::new();

    for line in text.lines() {
        // Skip lines that already have ANSI escape codes
        if line.contains("\x1b[") {
            result.push(line.to_string());
            continue;
        }

        // Check if this is a header line (words followed by colon at end, no double-space)
        if line.ends_with(':') && !line.contains("  ") {
            result.push(colors::header(line));
            continue;
        }

        // Check if this looks like a usage line
        if line.starts_with("Usage:") {
            result.push(colorize_usage_line(line));
            continue;
        }

        // Check if this is an example line (starts with "  wok ") - before command line check
        if let Some(colored) = colorize_example_line(line) {
            result.push(colored);
            continue;
        }

        // Check if this is a doc label line like "  Syntax: VALUE" - before command line check
        if let Some(colored) = colorize_doc_label_line(line) {
            result.push(colored);
            continue;
        }

        // Check if this is a command list line (indented command name + description)
        if let Some(colored) = colorize_command_line(line) {
            result.push(colored);
            continue;
        }

        // Check if this is an option line
        if let Some(caps) = option_re.captures(line) {
            result.push(colorize_option_line(&caps));
            continue;
        }

        // Keep other lines as-is
        result.push(line.to_string());
    }

    result.join("\n")
}

/// Colorize a command list line unconditionally.
fn colorize_command_line(line: &str) -> Option<String> {
    // Command lines start with exactly 2 spaces followed by a command name
    if !line.starts_with("  ") || line.starts_with("   ") {
        return None;
    }

    let trimmed = line.trim_start();

    // Option lines start with - (like "-n, --limit" or "--help")
    // These should be handled by colorize_option_line instead
    if trimmed.starts_with('-') {
        return None;
    }

    // Find where the command ends (before multiple spaces)
    let cmd_end = trimmed.find("  ").unwrap_or(trimmed.len());

    if cmd_end == 0 {
        return None;
    }

    let cmd = &trimmed[..cmd_end];
    let rest = &trimmed[cmd_end..];

    // Handle [un]command format
    let colored_cmd = if let Some(base) = cmd.strip_prefix("[un]") {
        format!("{}{}", colors::context("[un]"), colors::literal(base))
    } else {
        colors::literal(cmd)
    };

    Some(format!("  {}{}", colored_cmd, rest))
}

/// Colorize a usage line unconditionally.
fn colorize_usage_line(line: &str) -> String {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() == 2 {
        format!("{} {}", colors::header(parts[0]), parts[1])
    } else {
        line.to_string()
    }
}

/// Colorize an option line unconditionally.
fn colorize_option_line(caps: &regex::Captures) -> String {
    let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let short = caps.get(2).map(|m| m.as_str()).unwrap_or("");
    let flag_name = caps.get(3).map(|m| m.as_str()).unwrap_or("");
    let value = caps.get(4).map(|m| m.as_str()).unwrap_or("");
    let desc = caps.get(5).map(|m| m.as_str()).unwrap_or("");

    // Colorize short flag if present
    let colored_short = if short.is_empty() {
        String::new()
    } else {
        // Format: "-x, " where -x should be literal colored
        let trimmed = short.trim_end_matches(", ").trim_end();
        format!("{}, ", colors::literal(trimmed))
    };

    // Handle --[no-]flag format
    let colored_flag = if let Some(base) = flag_name.strip_prefix("[no-]") {
        format!(
            "{}{}{}",
            colors::literal("--"),
            colors::context("[no-]"),
            colors::literal(base)
        )
    } else {
        colors::literal(&format!("--{}", flag_name))
    };

    // Colorize value placeholder
    let colored_value = if value.is_empty() {
        String::new()
    } else {
        format!(" {}", colors::context(value.trim()))
    };

    // Colorize description, highlighting [default: ...] and [possible values: ...] as context
    let colored_desc = colorize_option_description(desc);

    format!(
        "{}{}{}{}{}",
        indent, colored_short, colored_flag, colored_value, colored_desc
    )
}

/// Colorize option description unconditionally, highlighting bracketed metadata as context.
fn colorize_option_description(desc: &str) -> String {
    if desc.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(desc.len() + 64);
    let mut i = 0;
    let bytes = desc.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'[' {
            // Find the closing bracket
            let start = i;
            let mut depth = 1;
            i += 1;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'[' {
                    depth += 1;
                } else if bytes[i] == b']' {
                    depth -= 1;
                }
                i += 1;
            }
            // Color the bracketed section as context
            let bracketed = &desc[start..i];
            result.push_str(&colors::context(bracketed));
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Colorize an example line unconditionally.
fn colorize_example_line(line: &str) -> Option<String> {
    // Example lines start with "  wok " (2 spaces + wok + space)
    if !line.starts_with("  wok ") {
        return None;
    }

    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];

    // Find where command ends (multiple spaces before description)
    let desc_start = colors::find_description_start(trimmed);

    let (cmd, desc) = if let Some(pos) = desc_start {
        (&trimmed[..pos], &trimmed[pos..])
    } else {
        (trimmed, "")
    };

    Some(format!("{}{}{}", indent, colorize_command(cmd), desc))
}

/// Colorize a command string unconditionally.
fn colorize_command(cmd: &str) -> String {
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
                        result.push_str(&colors::context(before));
                        in_flag_value = false;
                    } else {
                        result.push_str(&colors::literal(before));
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
                result.push_str(&colors::context(&cmd[quote_start..quote_end]));
                current_word_start = quote_end;
            }
            '<' => {
                // Flush any pending literal content before the angle bracket
                if i > current_word_start {
                    let before = &cmd[current_word_start..i];
                    if in_flag_value {
                        result.push_str(&colors::context(before));
                        in_flag_value = false;
                    } else {
                        result.push_str(&colors::literal(before));
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
                result.push_str(&colors::context(&cmd[bracket_start..bracket_end]));
                current_word_start = bracket_end;
            }
            ' ' => {
                // Flush current segment
                if i > current_word_start {
                    let segment = &cmd[current_word_start..i];
                    if in_flag_value {
                        result.push_str(&colors::context(segment));
                        in_flag_value = false;
                    } else {
                        result.push_str(&colors::literal(segment));
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
            result.push_str(&colors::context(remaining));
        } else {
            result.push_str(&colors::literal(remaining));
        }
    }

    result
}

/// Colorize a documentation label line unconditionally.
fn colorize_doc_label_line(line: &str) -> Option<String> {
    // Doc label lines are indented and have "Label: value" pattern
    if !line.starts_with("  ") {
        return None;
    }

    let trimmed = line.trim_start();

    // Option lines start with - (like "-n, --limit [default: text]")
    // These should be handled by colorize_option_line instead
    if trimmed.starts_with('-') {
        return None;
    }

    let indent = &line[..line.len() - trimmed.len()];

    // Look for "Label: value" pattern (colon followed by space)
    let colon_pos = trimmed.find(": ")?;

    // Must have content after ": "
    if colon_pos + 2 >= trimmed.len() {
        return None;
    }

    let label = &trimmed[..=colon_pos]; // Include the colon
    let value = &trimmed[colon_pos + 2..]; // Skip ": "

    Some(format!("{}{} {}", indent, label, colors::literal(value)))
}

/// Generate clap Styles for help output.
///
/// Returns `Styles::plain()` because we apply colors manually after
/// consolidating negatable flags.
pub fn styles() -> Styles {
    // Use plain styles - we colorize manually after consolidation
    Styles::plain()
}

/// Main help template.
/// Colors are applied later by colorize_help() since clap strips ANSI codes.
pub fn template() -> String {
    "{about-with-newline}
{usage-heading} {usage}

{before-help}Options:
{options}{after-help}"
        .to_string()
}

/// Commands list shown before options in main help.
///
/// Some command pairs that differ only by an "un" prefix are shown together
/// using the `[un]` convention (e.g., `[un]dep` covers both `dep` and `undep`).
/// This keeps the help output concise while still documenting all commands.
///
/// Note: This is called at runtime when the Command is constructed, but
/// stdout may not be a TTY at that point (e.g., during pipe operations).
/// Colors are applied later by format_help() if needed.
pub fn commands() -> String {
    // Return plain text - colors are applied later by colorize_help()
    // because clap's Styles::plain() strips ANSI codes from template values
    "\
Issue Tracking:
  new         Create a new issue
  [un]dep     Add/remove dependency between issues
  show        Show issue details
  tree        Show dependency tree
  list        List issues
  ready       Show ready issues (unblocked todos)
  search      Search issues by text
  start       Start work on issue(s)
  done        Mark issue(s) as done
  close       Close issue(s) without completing
  reopen      Return issue(s) to todo
  edit        Edit an issue's properties
  note        Add a note to an issue
  [un]label   Add/remove a label from issue(s)
  [un]link    Add/remove external link from an issue
  log         View event log

Setup & Configuration:
  init        Initialize issue tracker
  hooks       Manage Claude Code hooks
  config      Manage configuration
  daemon      Manage wokd daemon
  export      Export issues to JSONL
  import      Import issues from JSONL
  schema      Output JSON Schema for commands
  completion  Generate shell completions
  prime       Generate onboarding template"
        .to_string()
}

/// Quickstart help shown after options in main help.
/// Colors are applied later by colorize_help() since clap strips ANSI codes.
pub fn quickstart() -> String {
    "\
Get started:
  wok init                 Initialize tracker
  wok new task \"My task\"   Create a new task
  wok list                 List all issues
  wok start <id>           Start working on an issue
  wok done <id>            Mark issue as complete"
        .to_string()
}

#[cfg(test)]
#[path = "help_tests.rs"]
mod tests;
