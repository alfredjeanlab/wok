// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Background sync daemon for remote mode.
//!
//! The daemon maintains a persistent WebSocket connection to the remote server
//! and handles bidirectional sync. CLI commands communicate with the daemon
//! via a Unix socket.

mod ipc;
mod lifecycle;
mod runner;

pub use lifecycle::{
    detect_daemon, ensure_compatible_daemon, get_daemon_status, request_sync, spawn_daemon,
    stop_daemon,
};
pub use runner::run_daemon;

// Re-export for use in command tests
#[cfg(test)]
pub(crate) use ipc::{framing, DaemonRequest, DaemonResponse, DaemonStatus};
#[cfg(test)]
pub(crate) use lifecycle::{get_pid_path, get_socket_path};

#[cfg(test)]
#[path = "ipc_tests.rs"]
mod ipc_tests;

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
