// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::fs;
use tempfile::TempDir;
use wk_core::{Hlc, IssueType, Op, OpPayload};

fn make_op(wall_ms: u64, counter: u32, node: u32) -> Op {
    Op {
        id: Hlc::new(wall_ms, counter, node),
        payload: OpPayload::CreateIssue {
            id: format!("TEST-{}", counter),
            issue_type: IssueType::Task,
            title: format!("Test issue {}", counter),
        },
    }
}

#[test]
fn test_wal_append_and_read() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("pending.jsonl");

    let wal = Wal::open(&wal_path).unwrap();

    let op1 = make_op(1000, 1, 1);
    let op2 = make_op(2000, 2, 1);

    wal.append(&op1).unwrap();
    wal.append(&op2).unwrap();

    let ops = wal.read_all().unwrap();
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].id, op1.id);
    assert_eq!(ops[1].id, op2.id);
}

#[test]
fn test_wal_append_batch() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("pending.jsonl");

    let wal = Wal::open(&wal_path).unwrap();

    let ops = vec![
        make_op(1000, 1, 1),
        make_op(2000, 2, 1),
        make_op(3000, 3, 1),
    ];

    wal.append_batch(&ops).unwrap();

    let read_ops = wal.read_all().unwrap();
    assert_eq!(read_ops.len(), 3);
}

#[test]
fn test_wal_count() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("pending.jsonl");

    let wal = Wal::open(&wal_path).unwrap();

    assert_eq!(wal.count().unwrap(), 0);

    wal.append(&make_op(1000, 1, 1)).unwrap();
    assert_eq!(wal.count().unwrap(), 1);

    wal.append(&make_op(2000, 2, 1)).unwrap();
    assert_eq!(wal.count().unwrap(), 2);
}

#[test]
fn test_wal_clear() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("pending.jsonl");

    let wal = Wal::open(&wal_path).unwrap();

    wal.append(&make_op(1000, 1, 1)).unwrap();
    wal.append(&make_op(2000, 2, 1)).unwrap();

    assert_eq!(wal.count().unwrap(), 2);

    wal.clear().unwrap();

    assert_eq!(wal.count().unwrap(), 0);
    assert_eq!(wal.read_all().unwrap().len(), 0);
}

#[test]
fn test_wal_take_all() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("pending.jsonl");

    let wal = Wal::open(&wal_path).unwrap();

    wal.append(&make_op(1000, 1, 1)).unwrap();
    wal.append(&make_op(2000, 2, 1)).unwrap();

    let ops = wal.take_all().unwrap();
    assert_eq!(ops.len(), 2);

    // WAL should be empty after take
    assert_eq!(wal.count().unwrap(), 0);
    assert!(!wal.has_pending());
}

#[test]
fn test_wal_has_pending() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("pending.jsonl");

    let wal = Wal::open(&wal_path).unwrap();

    assert!(!wal.has_pending());

    wal.append(&make_op(1000, 1, 1)).unwrap();
    assert!(wal.has_pending());

    wal.clear().unwrap();
    assert!(!wal.has_pending());
}

#[test]
fn test_wal_empty_file() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("pending.jsonl");

    // Create empty file
    fs::write(&wal_path, "").unwrap();

    let wal = Wal::open(&wal_path).unwrap();

    assert_eq!(wal.count().unwrap(), 0);
    assert_eq!(wal.read_all().unwrap().len(), 0);
    assert!(!wal.has_pending());
}

#[test]
fn test_wal_nonexistent_file() {
    let temp = TempDir::new().unwrap();
    let wal_path = temp.path().join("nonexistent.jsonl");

    let wal = Wal::open(&wal_path).unwrap();

    assert_eq!(wal.count().unwrap(), 0);
    assert_eq!(wal.read_all().unwrap().len(), 0);
    assert!(!wal.has_pending());
}
