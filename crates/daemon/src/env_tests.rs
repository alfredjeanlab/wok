// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::names;
use super::*;
use std::path::PathBuf;

#[test]
fn constants_match_env_var_names() {
    assert_eq!(names::WOK_STATE_DIR, "WOK_STATE_DIR");
    assert_eq!(names::XDG_STATE_HOME, "XDG_STATE_HOME");
    assert_eq!(names::RUST_LOG, "RUST_LOG");
}

#[test]
fn state_dir_returns_path_when_set() {
    let _guard = EnvGuard::set(names::WOK_STATE_DIR, "/custom/state");
    assert_eq!(state_dir(), Some(PathBuf::from("/custom/state")));
}

#[test]
fn state_dir_returns_none_when_unset() {
    let _guard = EnvGuard::remove(names::WOK_STATE_DIR);
    assert_eq!(state_dir(), None);
}

#[test]
fn xdg_state_home_returns_path_when_set() {
    let _guard = EnvGuard::set(names::XDG_STATE_HOME, "/custom/xdg");
    assert_eq!(xdg_state_home(), Some(PathBuf::from("/custom/xdg")));
}

#[test]
fn xdg_state_home_returns_none_when_unset() {
    let _guard = EnvGuard::remove(names::XDG_STATE_HOME);
    assert_eq!(xdg_state_home(), None);
}

/// RAII guard that sets/removes an env var and restores it on drop.
struct EnvGuard {
    key: &'static str,
    original: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let original = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, original }
    }

    fn remove(key: &'static str) -> Self {
        let original = std::env::var(key).ok();
        std::env::remove_var(key);
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(val) => std::env::set_var(self.key, val),
            None => std::env::remove_var(self.key),
        }
    }
}
