// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Shared argument structs for CLI commands.
//!
//! These structs are used with `#[command(flatten)]` to reduce duplication
//! across commands that share common filter patterns.

use clap::Args;

/// Common filter arguments for type and label filtering.
#[derive(Args, Clone, Debug, Default)]
pub struct TypeLabelArgs {
    /// Filter by type (comma-separated for OR, repeat for AND)
    #[arg(long, short = 't')]
    pub r#type: Vec<String>,

    /// Filter by label (comma-separated for OR, repeat for AND)
    #[arg(long, short)]
    pub label: Vec<String>,
}

/// Assignee filter arguments.
#[derive(Args, Clone, Debug, Default)]
pub struct AssigneeArgs {
    /// Filter by assignee (comma-separated for OR)
    #[arg(long, short, value_delimiter = ',')]
    pub assignee: Vec<String>,

    /// Show only unassigned issues
    #[arg(long, conflicts_with = "assignee")]
    pub unassigned: bool,
}

/// Limit arguments for paginated results.
#[derive(Args, Clone, Debug, Default)]
pub struct LimitArgs {
    /// Maximum number of results
    #[arg(short = 'n', long, conflicts_with = "no_limit")]
    pub limit: Option<usize>,

    #[arg(long, conflicts_with = "limit")]
    pub no_limit: bool,
}
