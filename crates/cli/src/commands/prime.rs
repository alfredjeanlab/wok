// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::error::Result;

/// Template content embedded at compile time
pub(crate) const TEMPLATE: &str = include_str!("prime.md");

/// Run the prime command - outputs the template to stdout
///
/// Always outputs the template regardless of whether we're in a work directory.
/// This allows the template to be piped to a file for use in system prompts.
pub fn run() -> Result<()> {
    print!("{}", TEMPLATE);
    Ok(())
}

#[cfg(test)]
#[path = "prime_tests.rs"]
mod tests;
