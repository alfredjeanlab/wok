// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Operating mode detection for wk CLI.
//!
//! Determines whether to run in local mode (direct SQLite) or remote mode
//! (via daemon with WebSocket sync).

use crate::config::Config;

/// Operating mode for the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingMode {
    /// Local mode: Direct SQLite access, no sync, no daemon.
    Local,
    /// Remote mode: Requires daemon for sync with remote server.
    Remote,
}

impl OperatingMode {
    /// Detect the operating mode from the given configuration.
    pub fn detect(config: &Config) -> Self {
        if config.is_remote_mode() {
            OperatingMode::Remote
        } else {
            OperatingMode::Local
        }
    }

    /// Returns true if this is local mode.
    #[cfg(test)]
    pub fn is_local(&self) -> bool {
        *self == OperatingMode::Local
    }

    /// Returns true if this is remote mode.
    #[cfg(test)]
    pub fn is_remote(&self) -> bool {
        *self == OperatingMode::Remote
    }
}

impl std::fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingMode::Local => write!(f, "local"),
            OperatingMode::Remote => write!(f, "remote"),
        }
    }
}

#[cfg(test)]
#[path = "mode_tests.rs"]
mod tests;
