// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::config::RemoteConfig;

#[test]
fn test_detect_local_mode() {
    let config = Config::new("proj".to_string()).unwrap();
    let mode = OperatingMode::detect(&config);
    assert_eq!(mode, OperatingMode::Local);
    assert!(mode.is_local());
    assert!(!mode.is_remote());
    assert_eq!(mode.to_string(), "local");
}

#[test]
fn test_detect_remote_mode() {
    let config = Config {
        prefix: "proj".to_string(),
        workspace: None,
        remote: Some(RemoteConfig {
            url: "ws://remote:7890".to_string(),
            branch: "wk/oplog".to_string(),
            worktree: None,
            reconnect_max_retries: 10,
            reconnect_max_delay_secs: 30,
        }),
    };
    let mode = OperatingMode::detect(&config);
    assert_eq!(mode, OperatingMode::Remote);
    assert!(!mode.is_local());
    assert!(mode.is_remote());
    assert_eq!(mode.to_string(), "remote");
}
