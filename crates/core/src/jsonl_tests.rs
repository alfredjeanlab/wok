// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestRecord {
    id: u32,
    name: String,
}

#[test]
fn append_creates_file_if_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    let record = TestRecord {
        id: 1,
        name: "first".into(),
    };
    append(&path, &record).unwrap();

    assert!(path.exists());
}

#[test]
fn read_all_returns_empty_for_missing_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("missing.jsonl");

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert!(records.is_empty());
}

#[test]
fn append_and_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    let r1 = TestRecord {
        id: 1,
        name: "first".into(),
    };
    let r2 = TestRecord {
        id: 2,
        name: "second".into(),
    };

    append(&path, &r1).unwrap();
    append(&path, &r2).unwrap();

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert_eq!(records, vec![r1, r2]);
}

#[test]
fn read_all_skips_empty_lines() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    // Write content with empty lines manually
    std::fs::write(
        &path,
        "{\"id\":1,\"name\":\"a\"}\n\n{\"id\":2,\"name\":\"b\"}\n",
    )
    .unwrap();

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert_eq!(records.len(), 2);
}

#[test]
fn write_all_replaces_content() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    let r1 = TestRecord {
        id: 1,
        name: "first".into(),
    };
    append(&path, &r1).unwrap();

    let r2 = TestRecord {
        id: 2,
        name: "replaced".into(),
    };
    write_all(&path, &[r2.clone()]).unwrap();

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert_eq!(records, vec![r2]);
}
