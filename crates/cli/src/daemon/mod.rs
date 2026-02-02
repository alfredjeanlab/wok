// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Daemon lifecycle management for the wokd process.
//!
//! The CLI communicates with the wokd daemon via a Unix socket for
//! user-level mode database operations.

mod ipc;
mod lifecycle;

pub use lifecycle::{detect_daemon, get_daemon_status, spawn_daemon, stop_daemon_forcefully};

#[cfg(test)]
#[path = "ipc_tests.rs"]
mod ipc_tests;

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
