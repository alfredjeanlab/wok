// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::sync::Mutex;

// Mutex to serialize tests that modify environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_is_system_account() {
    assert!(is_system_account("root"));
    assert!(is_system_account("ROOT"));
    assert!(is_system_account("Root"));
    assert!(is_system_account("system"));
    assert!(is_system_account("administrator"));
    assert!(is_system_account("admin"));
    assert!(is_system_account("daemon"));
    assert!(is_system_account("nobody"));
    assert!(!is_system_account("alice"));
    assert!(!is_system_account("bob"));
    assert!(!is_system_account("kestred"));
}

#[test]
fn test_get_user_name_returns_non_empty() {
    let name = get_user_name();
    assert!(!name.is_empty());
}

#[test]
fn test_get_unix_username_respects_env() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Save original
    let original_user = std::env::var("USER").ok();

    // Test USER env var
    std::env::set_var("USER", "testuser");
    assert_eq!(get_unix_username(), Some("testuser".to_string()));

    // Restore
    match original_user {
        Some(v) => std::env::set_var("USER", v),
        None => std::env::remove_var("USER"),
    }
}

#[test]
fn test_get_unix_username_falls_through_to_logname() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let original_user = std::env::var("USER").ok();
    let original_logname = std::env::var("LOGNAME").ok();

    // When USER is not set, should fall back to LOGNAME
    std::env::remove_var("USER");
    std::env::set_var("LOGNAME", "testlogname");
    assert_eq!(get_unix_username(), Some("testlogname".to_string()));

    // Restore
    match original_user {
        Some(v) => std::env::set_var("USER", v),
        None => std::env::remove_var("USER"),
    }
    match original_logname {
        Some(v) => std::env::set_var("LOGNAME", v),
        None => std::env::remove_var("LOGNAME"),
    }
}

#[test]
fn test_get_unix_username_empty_returns_none() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let original_user = std::env::var("USER").ok();

    // Empty USER returns None (filter removes empty strings)
    std::env::set_var("USER", "");
    assert_eq!(get_unix_username(), None);

    // Restore
    match original_user {
        Some(v) => std::env::set_var("USER", v),
        None => std::env::remove_var("USER"),
    }
}
