// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Daemon lifecycle management for the wokd process.
//!
//! The CLI communicates with the wokd daemon via a Unix socket for
//! user-level mode database operations.

mod client;
mod lifecycle;

pub use client::DaemonClient;
pub use lifecycle::{
    detect_daemon, get_daemon_status, get_socket_path, spawn_daemon, stop_daemon_forcefully,
};
pub use wk_ipc::{MutateOp, MutateResult, QueryOp, QueryResult};

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
