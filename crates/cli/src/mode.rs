// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Operating mode detection for wk CLI.
//!
//! Determines whether to run in private mode (direct SQLite) or user-level
//! mode (via daemon with shared database).

use crate::config::Config;

/// Operating mode for the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Will be used when routing commands through daemon
pub enum OperatingMode {
    /// Private mode: Direct SQLite access at .wok/issues.db, no daemon.
    Private,
    /// User-level mode: Daemon at ~/.local/state/wok/, IPC for all operations.
    UserLevel,
}

impl OperatingMode {
    /// Detect the operating mode from the given configuration.
    #[allow(dead_code)] // Will be used when routing commands through daemon
    pub fn detect(config: &Config) -> Self {
        if config.private {
            OperatingMode::Private
        } else {
            OperatingMode::UserLevel
        }
    }

    /// Returns true if this is private mode.
    #[cfg(test)]
    pub fn is_private(&self) -> bool {
        *self == OperatingMode::Private
    }

    /// Returns true if this is user-level mode.
    #[cfg(test)]
    pub fn is_user_level(&self) -> bool {
        *self == OperatingMode::UserLevel
    }
}

impl std::fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingMode::Private => write!(f, "private"),
            OperatingMode::UserLevel => write!(f, "user-level"),
        }
    }
}

#[cfg(test)]
#[path = "mode_tests.rs"]
mod tests;
