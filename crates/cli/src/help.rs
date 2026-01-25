// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Help text generation with colorization support.

use crate::colors;
use clap::builder::styling::Styles;

/// Generate clap Styles for help output with v0 color conventions.
pub fn styles() -> Styles {
    if !colors::should_colorize() {
        return Styles::plain();
    }

    use anstyle::{Ansi256Color, Color, Style};

    let header = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(colors::codes::HEADER))));
    let literal = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(colors::codes::LITERAL))));
    let placeholder =
        Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(colors::codes::CONTEXT))));
    let context = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(colors::codes::CONTEXT))));

    Styles::styled()
        .header(header)
        .usage(header)
        .literal(literal)
        .placeholder(placeholder)
        .valid(context)
}

/// Main help template with colorized Options header.
pub fn template() -> String {
    format!(
        "{{about-with-newline}}
{{usage-heading}} {{usage}}

{{before-help}}{}
{{options}}{{after-help}}",
        colors::header("Options:")
    )
}

/// Commands list shown before options in main help.
///
/// Some command pairs that differ only by an "un" prefix are shown together
/// using the `[un]` convention (e.g., `[un]dep` covers both `dep` and `undep`).
/// This keeps the help output concise while still documenting all commands.
pub fn commands() -> String {
    format!(
        "\
{header_tracking}
  {new}         Create a new issue
  {un_dep}     Add/remove dependency between issues
  {show}        Show issue details
  {tree}        Show dependency tree
  {list}        List issues
  {ready}       Show ready issues (unblocked todos)
  {search}      Search issues by text
  {start}       Start work on issue(s)
  {done}        Mark issue(s) as done
  {close}       Close issue(s) without completing
  {reopen}      Return issue(s) to todo
  {edit}        Edit an issue's properties
  {note}        Add a note to an issue
  {un_label}   Add/remove a label from issue(s)
  {un_link}    Add/remove external link from an issue
  {log}         View event log

{header_setup}
  {init}        Initialize issue tracker
  {hooks}       Manage Claude Code hooks
  {config}      Manage configuration
  {remote}      Manage remote sync
  {export}      Export issues to JSONL
  {import}      Import issues from JSONL
  {schema}      Output JSON Schema for commands
  {completion}  Generate shell completions
  {prime}       Generate onboarding template",
        header_tracking = colors::header("Issue Tracking:"),
        header_setup = colors::header("Setup & Configuration:"),
        new = colors::literal("new"),
        un_dep = un_literal("dep"),
        show = colors::literal("show"),
        tree = colors::literal("tree"),
        list = colors::literal("list"),
        ready = colors::literal("ready"),
        search = colors::literal("search"),
        start = colors::literal("start"),
        done = colors::literal("done"),
        close = colors::literal("close"),
        reopen = colors::literal("reopen"),
        edit = colors::literal("edit"),
        note = colors::literal("note"),
        un_label = un_literal("label"),
        un_link = un_literal("link"),
        log = colors::literal("log"),
        init = colors::literal("init"),
        hooks = colors::literal("hooks"),
        config = colors::literal("config"),
        remote = colors::literal("remote"),
        export = colors::literal("export"),
        import = colors::literal("import"),
        schema = colors::literal("schema"),
        completion = colors::literal("completion"),
        prime = colors::literal("prime"),
    )
}

/// Format a command with `[un]` prefix, e.g., `[un]dep` for dep/undep pair.
/// The brackets and "un" are colored as context (dimmer), while the command name
/// is colored as literal (brighter).
fn un_literal(name: &str) -> String {
    format!("{}{}", colors::context("[un]"), colors::literal(name),)
}

/// Quickstart help shown after options in main help.
pub fn quickstart() -> String {
    colors::examples(
        "\
Get started:
  wok init                 Initialize tracker
  wok new task \"My task\"   Create a new task
  wok list                 List all issues
  wok start <id>           Start working on an issue
  wok done <id>            Mark issue as complete",
    )
}
