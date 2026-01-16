// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::models::Status;
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

// Tests for parse_db error paths

#[test]
fn test_parse_db_invalid_status() {
    let result = parse_db::<Status>("INVALID_STATUS", "status");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, rusqlite::Error::FromSqlConversionFailure(..)));
}

#[test]
fn test_parse_db_valid_status() {
    let result = parse_db::<Status>("todo", "status");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Status::Todo);
}

// Tests for parse_timestamp error paths

#[test]
fn test_parse_timestamp_invalid() {
    let result = parse_timestamp("NOT-A-TIMESTAMP", "created_at");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, rusqlite::Error::FromSqlConversionFailure(..)));
}

#[test]
fn test_parse_timestamp_malformed() {
    let result = parse_timestamp("2024-13-45T99:99:99Z", "created_at");
    assert!(result.is_err());
}

#[test]
fn test_parse_timestamp_valid() {
    let result = parse_timestamp("2024-01-15T10:30:00Z", "created_at");
    assert!(result.is_ok());
    let dt = result.unwrap();
    assert_eq!(dt.year(), 2024);
}

use chrono::Datelike;

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
