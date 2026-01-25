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
pub fn commands() -> String {
    format!(
        "\
{header_tracking}
  {new}         Create a new issue
  {dep}         Add dependency between issues
  {undep}       Remove dependency between issues
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
  {label}       Add a label to issue(s)
  {unlabel}     Remove a label from issue(s)
  {link}        Add external link to an issue
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
        dep = colors::literal("dep"),
        undep = colors::literal("undep"),
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
        label = colors::literal("label"),
        unlabel = colors::literal("unlabel"),
        link = colors::literal("link"),
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

/// Quickstart help shown after options in main help.
pub fn quickstart() -> String {
    format!(
        "\
{header}
  {init}                 Initialize tracker
  {new}   Create a new task
  {list}                 List all issues
  {start}           Start working on an issue
  {done}            Mark issue as complete",
        header = colors::header("Get started:"),
        init = colors::literal("wok init"),
        new = colors::literal("wok new task \"My task\""),
        list = colors::literal("wok list"),
        start = colors::literal("wok start <id>"),
        done = colors::literal("wok done <id>"),
    )
}
