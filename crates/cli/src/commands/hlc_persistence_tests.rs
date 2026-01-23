// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use tempfile::TempDir;
use wk_core::HlcClock;

#[test]
fn test_read_write_roundtrip() {
    let dir = TempDir::new().unwrap();
    let persistence = HlcPersistence::new(dir.path(), "test_hlc.txt");
    let clock = HlcClock::new(123);
    let hlc = clock.now();

    persistence.write(hlc).unwrap();
    let read_back = persistence.read().unwrap();

    assert_eq!(hlc, read_back);
}

#[test]
fn test_read_nonexistent_returns_none() {
    let dir = TempDir::new().unwrap();
    let persistence = HlcPersistence::new(dir.path(), "nonexistent.txt");

    assert!(persistence.read().is_none());
}

#[test]
fn test_update_only_advances() {
    let dir = TempDir::new().unwrap();
    let persistence = HlcPersistence::new(dir.path(), "test_hlc.txt");
    let clock = HlcClock::new(123);

    let hlc1 = clock.now();
    let hlc2 = clock.now(); // hlc2 > hlc1

    persistence.update(hlc2).unwrap();
    persistence.update(hlc1).unwrap(); // Should not update (hlc1 < hlc2)

    assert_eq!(persistence.read().unwrap(), hlc2);
}

#[test]
fn test_last_convenience_constructor() {
    let dir = TempDir::new().unwrap();
    let clock = HlcClock::new(123);
    let hlc = clock.now();

    // Verify last() creates a persistence that writes to the correct file
    HlcPersistence::last(dir.path()).write(hlc).unwrap();
    assert!(dir.path().join(HlcPersistence::LAST_HLC).exists());
}

#[test]
fn test_server_convenience_constructor() {
    let dir = TempDir::new().unwrap();
    let clock = HlcClock::new(123);
    let hlc = clock.now();

    // Verify server() creates a persistence that writes to the correct file
    HlcPersistence::server(dir.path()).write(hlc).unwrap();
    assert!(dir.path().join(HlcPersistence::SERVER_HLC).exists());
}

#[test]
fn test_update_writes_on_first_call() {
    let dir = TempDir::new().unwrap();
    let persistence = HlcPersistence::new(dir.path(), "test_hlc.txt");
    let clock = HlcClock::new(123);
    let hlc = clock.now();

    // File doesn't exist yet
    assert!(persistence.read().is_none());

    // update should write even when file doesn't exist
    persistence.update(hlc).unwrap();

    assert_eq!(persistence.read().unwrap(), hlc);
}
