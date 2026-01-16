// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Claude Code hooks management commands.
//!
//! Commands for installing, uninstalling, and checking status of
//! Claude Code hooks that integrate wk with AI assistants.

use std::io::{self, IsTerminal, Write};

use crossterm::{
    cursor, event,
    style::{Print, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};

use crate::error::{Error, Result};
use wk_core::hooks::{
    check_all_hooks, install_hooks, should_use_interactive, uninstall_hooks, HookScope,
};

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod tests;

/// Run the hooks install command.
pub fn install(scope: Option<String>, force_interactive: bool, force_yes: bool) -> Result<()> {
    // Determine the scope
    let scope = match scope {
        Some(s) => HookScope::parse(&s).ok_or_else(|| {
            Error::InvalidInput(format!(
                "Invalid scope '{}'. Use: local, project, or user",
                s
            ))
        })?,
        None => {
            // No scope provided - determine mode
            if force_yes {
                // Default to local in quiet mode
                HookScope::Local
            } else if force_interactive {
                // Must have TTY for interactive
                if !std::io::stdout().is_terminal() {
                    return Err(Error::InvalidInput(
                        "Interactive mode requires a terminal (TTY)".to_string(),
                    ));
                }
                run_interactive_picker()?
            } else if should_use_interactive() {
                run_interactive_picker()?
            } else {
                // Auto-detected non-interactive, use default
                HookScope::Local
            }
        }
    };

    // Install hooks
    let path = install_hooks(scope).map_err(|e| {
        if e.kind() == io::ErrorKind::PermissionDenied {
            Error::InvalidInput(format!(
                "Permission denied writing to {}. Check directory permissions.",
                scope.display_name()
            ))
        } else {
            Error::Io(e)
        }
    })?;

    println!(
        "Installed hooks to {} ({})",
        scope.display_name(),
        path.display()
    );
    Ok(())
}

/// Run the hooks uninstall command.
pub fn uninstall(scope: Option<String>) -> Result<()> {
    let scope = match scope {
        Some(s) => HookScope::parse(&s).ok_or_else(|| {
            Error::InvalidInput(format!(
                "Invalid scope '{}'. Use: local, project, or user",
                s
            ))
        })?,
        None => HookScope::Local,
    };

    uninstall_hooks(scope).map_err(Error::Io)?;

    println!("Uninstalled hooks from {}", scope.display_name());
    Ok(())
}

/// Run the hooks status command.
pub fn status() -> Result<()> {
    let statuses = check_all_hooks();
    let installed: Vec<_> = statuses.iter().filter(|s| s.installed).collect();

    if installed.is_empty() {
        println!("No hooks installed.");
        println!();
        println!("To install hooks, run:");
        println!("  wk hooks install [local|project|user]");
    } else {
        println!("Installed hooks:");
        for status in &installed {
            println!(
                "  - {} ({})",
                status.scope.display_name(),
                status.path.display()
            );
        }

        let not_installed: Vec<_> = statuses.iter().filter(|s| !s.installed).collect();
        if !not_installed.is_empty() {
            println!();
            println!("Not installed:");
            for status in not_installed {
                println!("  - {}", status.scope.display_name());
            }
        }
    }

    Ok(())
}

/// Scope picker items with their descriptions.
const SCOPE_ITEMS: [(&str, &str); 3] = [
    ("local", "Per-project, git-ignored"),
    ("project", "Per-project, committed"),
    ("user", "Per-machine (~/.claude)"),
];

/// Draw the picker UI at the current cursor position.
fn draw_picker(stdout: &mut io::Stdout, selected: usize) -> io::Result<()> {
    stdout.queue(Print("Select hooks installation scope:\r\n"))?;

    for (i, (name, desc)) in SCOPE_ITEMS.iter().enumerate() {
        let marker = if i == selected { "●" } else { "○" };
        if i == selected {
            stdout.queue(Print(
                format!("  {} {} - {}\r\n", marker, name, desc).bold(),
            ))?;
        } else {
            stdout.queue(Print(format!("  {} {} - {}\r\n", marker, name, desc)))?;
        }
    }

    stdout.queue(Print("\r\n"))?;
    stdout.queue(Print("↑/↓: Navigate  Enter: Select  q: Cancel".dark_grey()))?;
    stdout.flush()
}

/// Run the interactive inline picker for scope selection.
///
/// Uses crossterm for terminal UI with radio button selection.
fn run_interactive_picker() -> Result<HookScope> {
    let mut stdout = io::stdout();
    let mut selected: usize = 0;

    // Enable raw mode for immediate key handling
    terminal::enable_raw_mode().map_err(Error::Io)?;

    // Hide cursor during selection
    let _ = stdout.execute(cursor::Hide);

    // Draw initial state
    let draw_result = draw_picker(&mut stdout, selected);
    if let Err(e) = draw_result {
        let _ = stdout.execute(cursor::Show);
        let _ = terminal::disable_raw_mode();
        return Err(Error::Io(e));
    }

    // Total lines we draw (header + 3 items + blank + hint = 6 lines)
    let total_lines: u16 = 6;

    let result = loop {
        // Wait for key event
        let evt = match event::read() {
            Ok(e) => e,
            Err(e) => {
                let _ = stdout.execute(cursor::Show);
                let _ = terminal::disable_raw_mode();
                return Err(Error::Io(e));
            }
        };

        if let event::Event::Key(key) = evt {
            match key.code {
                event::KeyCode::Up | event::KeyCode::Char('k') => {
                    let len = SCOPE_ITEMS.len();
                    selected = (selected + len - 1) % len;
                }
                event::KeyCode::Down | event::KeyCode::Char('j') => {
                    selected = (selected + 1) % SCOPE_ITEMS.len();
                }
                event::KeyCode::Enter => {
                    break Some(selected);
                }
                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                    break None;
                }
                event::KeyCode::Char('c')
                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    break None;
                }
                _ => continue,
            }

            // Redraw: move cursor up to start of our output, clear, and redraw
            let _ = stdout.execute(cursor::MoveUp(total_lines));
            let _ = stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown));
            let _ = draw_picker(&mut stdout, selected);
        }
    };

    // Clean up: clear the picker UI
    let _ = stdout.execute(cursor::MoveUp(total_lines));
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown));
    let _ = stdout.execute(cursor::Show);
    let _ = terminal::disable_raw_mode();

    match result {
        Some(idx) => Ok(match SCOPE_ITEMS[idx].0 {
            "local" => HookScope::Local,
            "project" => HookScope::Project,
            "user" => HookScope::User,
            _ => HookScope::Local,
        }),
        None => Err(Error::InvalidInput("Cancelled".to_string())),
    }
}
