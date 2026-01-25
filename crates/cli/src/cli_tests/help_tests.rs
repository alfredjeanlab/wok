// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for help output completeness.

use crate::help;
use clap::CommandFactory;

/// Verifies all non-hidden subcommands appear in the help output.
///
/// This test ensures that when new commands are added, they don't get
/// forgotten in the custom help text. Commands shown with `[un]` prefix
/// (like `[un]dep` for `dep`/`undep`) are accounted for.
#[test]
fn all_subcommands_in_help() {
    std::env::set_var("NO_COLOR", "1");
    let cmd = crate::Cli::command();
    let help_text = help::commands();

    // Commands that are shown together with [un] prefix
    // The [un]X format covers both X and unX commands
    let un_prefixed: &[&str] = &["dep", "label", "link"];

    for sub in cmd.get_subcommands() {
        let name = sub.get_name();

        // Skip hidden commands (intentionally not shown)
        if sub.is_hide_set() {
            continue;
        }

        // Check if this is an "un" variant of an [un] prefixed command
        if let Some(base) = name.strip_prefix("un") {
            if un_prefixed.contains(&base) {
                // This command is covered by [un]base format
                assert!(
                    help_text.contains(&format!("[un]{}", base)),
                    "Command '{name}' should be shown as '[un]{base}' in help output"
                );
                continue;
            }
        }

        // Check if this is a base command with [un] prefix
        if un_prefixed.contains(&name) {
            assert!(
                help_text.contains(&format!("[un]{}", name)),
                "Command '{name}' should be shown as '[un]{name}' in help output"
            );
            continue;
        }

        // Regular command - should appear directly in help
        // Match as whole word to avoid false positives (e.g., "remote" in "wok config remote")
        let pattern = format!("  {}", name);
        assert!(
            help_text.contains(&pattern),
            "Command '{name}' not found in help output. \
             If this is intentionally hidden, add `hide = true` to the command. \
             If this should be paired with an 'un' variant, add it to the un_prefixed list."
        );
    }
}

/// Verifies the [un] prefixed commands are displayed correctly.
#[test]
fn un_prefixed_format() {
    std::env::set_var("NO_COLOR", "1");
    let help_text = help::commands();

    // These should appear as [un]X format
    assert!(
        help_text.contains("[un]dep"),
        "Should show [un]dep for dep/undep"
    );
    assert!(
        help_text.contains("[un]label"),
        "Should show [un]label for label/unlabel"
    );
    assert!(
        help_text.contains("[un]link"),
        "Should show [un]link for link/unlink"
    );

    // These should NOT appear as separate entries
    assert!(
        !help_text.contains("  dep "),
        "Should not show 'dep' separately (use [un]dep)"
    );
    assert!(
        !help_text.contains("  undep "),
        "Should not show 'undep' separately (use [un]dep)"
    );
}
