// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema command implementation.
//!
//! Outputs JSON Schema specifications for commands that support JSON output.

use crate::cli::SchemaCommand;
use crate::error::Result;
use crate::schema::{list, ready, search, show};
use schemars::schema_for;

/// Run the schema command.
pub fn run(cmd: SchemaCommand) -> Result<()> {
    let schema = match cmd {
        SchemaCommand::List => schema_for!(list::ListOutputJson),
        SchemaCommand::Show => schema_for!(show::IssueDetails),
        SchemaCommand::Ready => schema_for!(ready::ReadyOutputJson),
        SchemaCommand::Search => schema_for!(search::SearchOutputJson),
    };

    let json = serde_json::to_string_pretty(&schema)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;
