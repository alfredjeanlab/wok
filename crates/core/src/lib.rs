// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! wk-core: Shared library for wk distributed issue tracker
//!
//! This crate provides the core data structures, database operations, and sync
//! primitives used by both the wk CLI and wk-remote server.

pub mod db;
pub mod detect;
pub mod error;
pub mod hlc;
pub mod hooks;
pub mod identity;
pub mod issue;
pub mod jsonl;
pub mod merge;
pub mod op;
pub mod oplog;
pub mod protocol;

pub use db::Database;
pub use error::{Error, Result};
pub use hlc::{ClockSource, Hlc, HlcClock, SystemClock};
pub use issue::{Event, Issue, IssueType, Status};
pub use merge::Merge;
pub use op::{Op, OpId, OpPayload};
pub use oplog::Oplog;
pub use protocol::{ClientMessage, ServerMessage};
