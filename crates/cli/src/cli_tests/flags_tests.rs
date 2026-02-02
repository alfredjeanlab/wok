// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

/// Test that only allowed long-form flags have short forms,
/// all allowed short forms are correctly mapped, and all
/// entries in the allowed set are actually used somewhere.
#[test]
fn test_flag_consistency() {
    use clap::CommandFactory;

    // Allowed short -> long mappings for our manually defined flags
    // Note: -h/--help is auto-managed by clap and not tracked here
    let allowed: std::collections::HashMap<char, &str> = [
        ('C', "directory"), // -C, --directory (top-level, like git -C)
        ('v', "version"),   // -v, --version (top-level only)
        ('r', "reason"),
        ('t', "type"),
        ('l', "label"),
        ('s', "status"),
        ('i', "interactive"),
        ('f', "format"), // import input format
        ('o', "output"), // output format
        ('y', "yes"),
        ('a', "assignee"),
        ('q', "filter"),
        ('n', "limit"),
        ('p', "prefix"), // -p, --prefix (for new command)
    ]
    .into_iter()
    .collect();

    let cmd = Cli::command();
    let mut errors = Vec::new();
    let mut used_shorts: std::collections::HashSet<char> = std::collections::HashSet::new();

    check_command_flags(&cmd, &allowed, &mut errors, &mut used_shorts);

    // Check that all allowed short forms are actually used
    for (short_char, long_name) in &allowed {
        if !used_shorts.contains(short_char) {
            errors.push(format!(
                "-{} (--{}) is in allowed set but never used",
                short_char, long_name
            ));
        }
    }

    if !errors.is_empty() {
        panic!("Flag consistency violations:\n{}", errors.join("\n"));
    }
}

fn check_command_flags(
    cmd: &clap::Command,
    allowed: &std::collections::HashMap<char, &str>,
    errors: &mut Vec<String>,
    used_shorts: &mut std::collections::HashSet<char>,
) {
    let cmd_name = cmd.get_name();

    for arg in cmd.get_arguments() {
        let long = arg.get_long();
        let short = arg.get_short();

        // Skip positional arguments
        if long.is_none() && short.is_none() {
            continue;
        }

        if let Some(short_char) = short {
            // Skip clap's auto-added help flag
            if short_char == 'h' && long == Some("help") {
                continue;
            }

            // Track that this short flag is used
            used_shorts.insert(short_char);

            // (a) Only allowed long forms can have short flags
            if let Some(expected_long) = allowed.get(&short_char) {
                // (b) Short flag must map to the correct long form
                if let Some(actual_long) = long {
                    if actual_long != *expected_long {
                        errors.push(format!(
                            "{}: -{} maps to --{} but should map to --{}",
                            cmd_name, short_char, actual_long, expected_long
                        ));
                    }
                } else {
                    errors.push(format!(
                        "{}: -{} has no long form, expected --{}",
                        cmd_name, short_char, expected_long
                    ));
                }
            } else {
                // (c) No short flags outside the allowed list
                let long_name = long.unwrap_or("(none)");
                errors.push(format!(
                    "{}: -{} (--{}) is not an allowed short flag",
                    cmd_name, short_char, long_name
                ));
            }
        }
    }

    // Recurse into subcommands
    for subcmd in cmd.get_subcommands() {
        check_command_flags(subcmd, allowed, errors, used_shorts);
    }
}
