// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use clap::{CommandFactory, Parser};
use wkrs::Cli;

fn main() {
    // Try to parse; if help/version is requested, clap will handle it
    // with our plain styles. We then post-process for consolidation.
    let result = Cli::try_parse();

    match result {
        Ok(cli) => {
            if let Err(e) = wkrs::run(cli.command) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            // Check if this is a help or version request
            if e.kind() == clap::error::ErrorKind::DisplayHelp {
                // User explicitly requested help (--help)
                let args: Vec<String> = std::env::args().collect();
                print_formatted_help(&args, false);
            } else if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
                let args: Vec<String> = std::env::args().collect();
                // Check if this is just bare "wk" with no arguments
                let has_subcommand = args.iter().skip(1).any(|a| !a.starts_with('-'));
                if has_subcommand {
                    // Missing required arguments for a subcommand - show help to stderr, exit with error
                    print_formatted_help(&args, true);
                    std::process::exit(2);
                } else {
                    // No subcommand at all (bare "wk") - show help to stdout, exit success
                    print_formatted_help(&args, false);
                }
            } else if e.kind() == clap::error::ErrorKind::DisplayVersion {
                // Let clap handle version display
                e.exit();
            } else {
                // For other errors (invalid args, etc.), let clap handle it
                e.exit();
            }
        }
    }
}

/// Print help with negatable flag consolidation.
fn print_formatted_help(args: &[String], to_stderr: bool) {
    use wkrs::help;

    // Find the subcommand being requested help for
    // Args could be: ["wk", "--help"], ["wk", "list", "--help"], ["wk", "help", "list"], etc.
    let cmd = Cli::command();

    // Look for subcommand names in the args (skip binary name and flags)
    // Handle both "wk list --help" and "wk help list" patterns
    let non_flags: Vec<&String> = args
        .iter()
        .skip(1)
        .filter(|arg| !arg.starts_with('-'))
        .collect();

    // If first non-flag is "help", skip it to get actual subcommand names
    let subcommand_names: Vec<&str> = if non_flags.first().map(|s| s.as_str()) == Some("help") {
        non_flags.iter().skip(1).map(|s| s.as_str()).collect()
    } else {
        non_flags.iter().map(|s| s.as_str()).collect()
    };

    let print_fn = if to_stderr {
        help::eprint_help
    } else {
        help::print_help
    };

    // Find the deepest matching subcommand
    let mut target_cmd = find_subcommand(cmd, &subcommand_names);
    print_fn(&mut target_cmd);
}

/// Recursively find a nested subcommand by name path.
/// Returns an owned Command since we need to extract it from the parent.
fn find_subcommand(mut cmd: clap::Command, names: &[&str]) -> clap::Command {
    for name in names {
        let mut found_sub = None;
        for sub in cmd.get_subcommands() {
            if sub.get_name() == *name || sub.get_all_aliases().any(|a| a == *name) {
                found_sub = Some(sub.get_name().to_string());
                break;
            }
        }
        if let Some(sub_name) = found_sub {
            // Clone the subcommand to get an owned version
            if let Some(sub) = cmd.find_subcommand_mut(&sub_name) {
                cmd = sub.clone();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    cmd
}
