// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! wk-core: Shared library for wk issue tracker
//!
//! This crate provides the core data structures, database operations, and
//! primitives used by both the wk CLI and wokd daemon.

pub mod db;
pub mod detect;
pub mod error;
pub mod hlc;
pub mod hooks;
pub mod identity;
pub mod issue;
pub mod jsonl;
pub mod link;
pub mod merge;
pub mod op;

pub use db::Database;
pub use error::{Error, Result};
pub use hlc::{ClockSource, Hlc, HlcClock, SystemClock};
pub use issue::{Action, Dependency, Event, Issue, IssueType, Note, Relation, Status};
pub use link::{Link, LinkRel, LinkType, PrefixInfo};
pub use merge::Merge;
pub use op::{Op, OpId, OpPayload};
