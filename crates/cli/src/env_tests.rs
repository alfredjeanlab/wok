// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use std::path::PathBuf;

#[test]
fn test_vars_constants() {
    assert_eq!(vars::WK_TIMINGS, "WK_TIMINGS");
    assert_eq!(vars::NO_COLOR, "NO_COLOR");
    assert_eq!(vars::COLOR, "COLOR");
    assert_eq!(vars::WOK_STATE_DIR, "WOK_STATE_DIR");
    assert_eq!(vars::XDG_STATE_HOME, "XDG_STATE_HOME");
    assert_eq!(vars::WOK_DAEMON_BINARY, "WOK_DAEMON_BINARY");
}

#[test]
fn test_wk_timings_unset() {
    std::env::remove_var("WK_TIMINGS");
    assert!(!wk_timings());
}

#[test]
fn test_wk_timings_set() {
    std::env::set_var("WK_TIMINGS", "1");
    assert!(wk_timings());
    std::env::remove_var("WK_TIMINGS");
}

#[test]
fn test_wk_timings_set_any_value() {
    std::env::set_var("WK_TIMINGS", "yes");
    assert!(wk_timings());
    std::env::remove_var("WK_TIMINGS");
}

#[test]
fn test_no_color_unset() {
    std::env::remove_var("NO_COLOR");
    assert!(!no_color());
}

#[test]
fn test_no_color_set_to_one() {
    std::env::set_var("NO_COLOR", "1");
    assert!(no_color());
    std::env::remove_var("NO_COLOR");
}

#[test]
fn test_no_color_set_to_other() {
    std::env::set_var("NO_COLOR", "true");
    assert!(!no_color());
    std::env::remove_var("NO_COLOR");
}

#[test]
fn test_force_color_unset() {
    std::env::remove_var("COLOR");
    assert!(!force_color());
}

#[test]
fn test_force_color_set_to_one() {
    std::env::set_var("COLOR", "1");
    assert!(force_color());
    std::env::remove_var("COLOR");
}

#[test]
fn test_force_color_set_to_other() {
    std::env::set_var("COLOR", "yes");
    assert!(!force_color());
    std::env::remove_var("COLOR");
}

#[test]
fn test_state_dir_unset() {
    std::env::remove_var("WOK_STATE_DIR");
    assert_eq!(state_dir(), None);
}

#[test]
fn test_state_dir_set() {
    std::env::set_var("WOK_STATE_DIR", "/tmp/wok-test");
    assert_eq!(state_dir(), Some(PathBuf::from("/tmp/wok-test")));
    std::env::remove_var("WOK_STATE_DIR");
}

#[test]
fn test_xdg_state_home_unset() {
    std::env::remove_var("XDG_STATE_HOME");
    assert_eq!(xdg_state_home(), None);
}

#[test]
fn test_xdg_state_home_set() {
    std::env::set_var("XDG_STATE_HOME", "/tmp/xdg-test");
    assert_eq!(xdg_state_home(), Some(PathBuf::from("/tmp/xdg-test")));
    std::env::remove_var("XDG_STATE_HOME");
}

#[test]
fn test_daemon_binary_unset() {
    std::env::remove_var("WOK_DAEMON_BINARY");
    assert_eq!(daemon_binary(), None);
}

#[test]
fn test_daemon_binary_set() {
    std::env::set_var("WOK_DAEMON_BINARY", "/usr/local/bin/wokd");
    assert_eq!(daemon_binary(), Some(PathBuf::from("/usr/local/bin/wokd")));
    std::env::remove_var("WOK_DAEMON_BINARY");
}
