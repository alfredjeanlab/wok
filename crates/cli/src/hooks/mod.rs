// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Issue hooks system for running scripts on issue events.
//!
//! This module provides:
//! - Configuration loading from `.wok/hooks.toml` and `.wok/hooks.json`
//! - Event name mapping from `Action` enum to hook event names
//! - Filter string parsing (e.g., "-t bug -l urgent")
//! - Payload building for hook stdin
//! - Fire-and-forget hook execution
//!
//! # Configuration Format
//!
//! Hooks are configured in `.wok/hooks.toml`:
//!
//! ```toml
//! [[hooks]]
//! name = "urgent-bugs"
//! events = ["issue.created"]
//! filter = "-t bug -l urgent"
//! run = "./scripts/page-oncall.sh"
//! ```
//!
//! Or in `.wok/hooks.json`:
//!
//! ```json
//! {
//!   "hooks": [{
//!     "name": "urgent-bugs",
//!     "events": ["issue.created"],
//!     "filter": "-t bug -l urgent",
//!     "run": "./scripts/page-oncall.sh"
//!   }]
//! }
//! ```

pub mod config;
pub mod event;
pub mod executor;
pub mod filter;
pub mod payload;
pub mod runner;

pub use config::{load_hooks_config, HookConfig, HooksConfig};
pub use event::HookEvent;
pub use filter::HookFilter;
pub use payload::HookPayload;
pub use runner::{run_hooks_for_event, test_hook};
