// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Centralized environment variable access.
//!
//! All runtime environment variables used by the CLI are defined here
//! with typed accessor functions. The variable name constants are generated
//! by `build.rs` and live in the [`vars`] submodule.

use std::path::PathBuf;

/// Generated environment variable name constants.
pub mod vars {
    include!(concat!(env!("OUT_DIR"), "/env_vars.rs"));
}

/// Returns `true` if `WK_TIMINGS` is set (any value).
pub fn wk_timings() -> bool {
    std::env::var(vars::WK_TIMINGS).is_ok()
}

/// Returns `true` if `NO_COLOR=1`.
pub fn no_color() -> bool {
    std::env::var(vars::NO_COLOR).is_ok_and(|v| v == "1")
}

/// Returns `true` if `COLOR=1`.
pub fn force_color() -> bool {
    std::env::var(vars::COLOR).is_ok_and(|v| v == "1")
}

/// Returns the value of `WOK_STATE_DIR` if set.
pub fn state_dir() -> Option<PathBuf> {
    std::env::var(vars::WOK_STATE_DIR).ok().map(PathBuf::from)
}

/// Returns the value of `XDG_STATE_HOME` if set.
pub fn xdg_state_home() -> Option<PathBuf> {
    std::env::var(vars::XDG_STATE_HOME).ok().map(PathBuf::from)
}

/// Returns the value of `WOK_DAEMON_BINARY` if set.
pub fn daemon_binary() -> Option<PathBuf> {
    std::env::var(vars::WOK_DAEMON_BINARY)
        .ok()
        .map(PathBuf::from)
}

#[cfg(test)]
#[path = "env_tests.rs"]
mod tests;
