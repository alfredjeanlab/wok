// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use tempfile::tempdir;

#[test]
fn test_open_in_memory() {
    let db = Database::open_in_memory().unwrap();

    // Verify tables exist
    let mut stmt = db
        .conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap();

    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(tables.contains(&"issues".to_string()));
    assert!(tables.contains(&"deps".to_string()));
    assert!(tables.contains(&"labels".to_string()));
    assert!(tables.contains(&"notes".to_string()));
    assert!(tables.contains(&"events".to_string()));
}

#[test]
fn test_wal_mode_enabled_for_file_db() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).unwrap();

    // Check WAL mode is enabled
    let journal_mode: String = db
        .conn
        .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");

    // Check busy_timeout is set
    let busy_timeout: i32 = db
        .conn
        .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(busy_timeout, 5000);
}

#[test]
fn test_busy_timeout_for_in_memory_db() {
    let db = Database::open_in_memory().unwrap();

    // Check busy_timeout is set (WAL not supported in memory)
    let busy_timeout: i32 = db
        .conn
        .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(busy_timeout, 5000);
}
