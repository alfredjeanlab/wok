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
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            {
                // Get the command and subcommand from args to find which help to show
                let args: Vec<String> = std::env::args().collect();
                print_formatted_help(&args);
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
fn print_formatted_help(args: &[String]) {
    use wkrs::help;

    // Find the subcommand being requested help for
    // Args could be: ["wk", "--help"], ["wk", "list", "--help"], ["wk", "help", "list"], etc.
    let mut cmd = Cli::command();

    // Look for subcommand names in the args (skip binary name and flags)
    // Handle both "wk list --help" and "wk help list" patterns
    let non_flags: Vec<&String> = args.iter().skip(1).filter(|arg| !arg.starts_with('-')).collect();

    // If first non-flag is "help", use the second non-flag as the subcommand
    // Otherwise use the first non-flag
    let subcommand_name = if non_flags.first().map(|s| s.as_str()) == Some("help") {
        non_flags.get(1).map(|s| s.as_str())
    } else {
        non_flags.first().map(|s| s.as_str())
    };

    if let Some(name) = subcommand_name {
        // Find the subcommand and format its help
        for sub in cmd.get_subcommands_mut() {
            if sub.get_name() == name || sub.get_all_aliases().any(|a| a == name) {
                help::print_help(sub);
                return;
            }
        }
    }

    // No subcommand or not found - print main help
    help::print_help(&mut cmd);
}
