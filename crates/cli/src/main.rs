// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use clap::Parser;
use wkrs::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = wkrs::run(cli.command) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
