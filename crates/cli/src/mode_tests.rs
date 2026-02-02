// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn test_detect_user_level_mode() {
    let config = Config::new("proj".to_string()).unwrap();
    let mode = OperatingMode::detect(&config);
    assert_eq!(mode, OperatingMode::UserLevel);
    assert!(mode.is_user_level());
    assert!(!mode.is_private());
    assert_eq!(mode.to_string(), "user-level");
}

#[test]
fn test_detect_private_mode() {
    let config = Config::new_private("proj".to_string()).unwrap();
    let mode = OperatingMode::detect(&config);
    assert_eq!(mode, OperatingMode::Private);
    assert!(mode.is_private());
    assert!(!mode.is_user_level());
    assert_eq!(mode.to_string(), "private");
}
