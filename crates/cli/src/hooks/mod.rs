// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Issue hooks system for running scripts when issues change.
//!
//! This module provides a hooks system that triggers shell scripts when issues
//! are created, edited, or change state. Hooks are configured in `.wok/hooks.toml`
//! and/or `.wok/hooks.json`.
//!
//! # Configuration
//!
//! ```toml
//! [[hooks]]
//! name = "urgent-bugs"
//! events = ["issue.created"]
//! filter = "-t bug -l urgent"
//! run = "./scripts/page-oncall.sh"
//! ```
//!
//! # Event Types
//!
//! - `issue.created`, `issue.edited`
//! - `issue.started`, `issue.stopped`
//! - `issue.done`, `issue.closed`, `issue.reopened`
//! - `issue.labeled`, `issue.unlabeled`
//! - `issue.assigned`, `issue.unassigned`
//! - `issue.noted`, `issue.linked`, `issue.unlinked`
//! - `issue.related`, `issue.unrelated`
//! - `issue.blocked`, `issue.unblocked`
//! - `issue.*` (wildcard matching all events)

mod config;
mod event;
mod executor;
mod filter;
mod payload;
mod runner;

pub use config::{load_hooks_config, HookConfig, HooksConfig};
pub use event::HookEvent;
pub use executor::execute_hook;
pub use filter::HookFilter;
pub use payload::{ChangePayload, HookPayload, IssuePayload};
pub use runner::run_hooks_for_event;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
