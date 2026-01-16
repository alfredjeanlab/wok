// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::issue::IssueType;
use crate::op::OpPayload;
use tempfile::TempDir;

fn test_op(wall_ms: u64, id: &str) -> Op {
    Op::new(
        Hlc::new(wall_ms, 0, 1),
        OpPayload::create_issue(id.to_string(), IssueType::Task, "Title".to_string()),
    )
}

#[test]
fn oplog_append_and_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("oplog.jsonl");

    let mut oplog = Oplog::open(&path).unwrap();
    assert!(oplog.is_empty());

    let op1 = test_op(1000, "test-1");
    let op2 = test_op(2000, "test-2");
    let op3 = test_op(3000, "test-3");

    assert!(oplog.append(&op1).unwrap());
    assert!(oplog.append(&op2).unwrap());
    assert!(oplog.append(&op3).unwrap());

    assert_eq!(oplog.len(), 3);

    let all = oplog.all_ops().unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].id, op1.id);
    assert_eq!(all[1].id, op2.id);
    assert_eq!(all[2].id, op3.id);
}

#[test]
fn oplog_deduplication() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("oplog.jsonl");

    let mut oplog = Oplog::open(&path).unwrap();

    let op = test_op(1000, "test-1");

    assert!(oplog.append(&op).unwrap()); // First append succeeds
    assert!(!oplog.append(&op).unwrap()); // Second append is deduplicated
    assert!(!oplog.append(&op).unwrap()); // Third append is also deduplicated

    assert_eq!(oplog.len(), 1);
}

#[test]
fn oplog_ops_since() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("oplog.jsonl");

    let mut oplog = Oplog::open(&path).unwrap();

    let op1 = test_op(1000, "test-1");
    let op2 = test_op(2000, "test-2");
    let op3 = test_op(3000, "test-3");

    oplog.append(&op1).unwrap();
    oplog.append(&op2).unwrap();
    oplog.append(&op3).unwrap();

    // Since before all ops
    let since_0 = oplog.ops_since(Hlc::min()).unwrap();
    assert_eq!(since_0.len(), 3);

    // Since after op1
    let since_1500 = oplog.ops_since(Hlc::new(1500, 0, 0)).unwrap();
    assert_eq!(since_1500.len(), 2);
    assert_eq!(since_1500[0].id, op2.id);
    assert_eq!(since_1500[1].id, op3.id);

    // Since after op2
    let since_2500 = oplog.ops_since(Hlc::new(2500, 0, 0)).unwrap();
    assert_eq!(since_2500.len(), 1);
    assert_eq!(since_2500[0].id, op3.id);

    // Since after all ops
    let since_4000 = oplog.ops_since(Hlc::new(4000, 0, 0)).unwrap();
    assert!(since_4000.is_empty());
}

#[test]
fn oplog_persistence() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("oplog.jsonl");

    // Create and write ops
    {
        let mut oplog = Oplog::open(&path).unwrap();
        oplog.append(&test_op(1000, "test-1")).unwrap();
        oplog.append(&test_op(2000, "test-2")).unwrap();
    }

    // Reopen and verify
    {
        let oplog = Oplog::open(&path).unwrap();
        assert_eq!(oplog.len(), 2);
        assert!(oplog.contains(&Hlc::new(1000, 0, 1)));
        assert!(oplog.contains(&Hlc::new(2000, 0, 1)));
    }

    // Append more and verify dedup still works after reload
    {
        let mut oplog = Oplog::open(&path).unwrap();
        assert!(!oplog.append(&test_op(1000, "test-1")).unwrap()); // Duplicate
        assert!(oplog.append(&test_op(3000, "test-3")).unwrap()); // New
        assert_eq!(oplog.len(), 3);
    }
}

#[test]
fn oplog_nonexistent_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.jsonl");

    let oplog = Oplog::open(&path).unwrap();
    assert!(oplog.is_empty());

    let ops = oplog.ops_since(Hlc::min()).unwrap();
    assert!(ops.is_empty());
}

#[test]
fn oplog_contains() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("oplog.jsonl");

    let mut oplog = Oplog::open(&path).unwrap();

    let op = test_op(1000, "test-1");
    assert!(!oplog.contains(&op.id));

    oplog.append(&op).unwrap();
    assert!(oplog.contains(&op.id));
}

#[test]
fn oplog_sorted_ops() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("oplog.jsonl");

    let mut oplog = Oplog::open(&path).unwrap();

    // Append in non-chronological order
    oplog.append(&test_op(3000, "test-3")).unwrap();
    oplog.append(&test_op(1000, "test-1")).unwrap();
    oplog.append(&test_op(2000, "test-2")).unwrap();

    // ops_since should return sorted
    let ops = oplog.all_ops().unwrap();
    assert_eq!(ops[0].id.wall_ms, 1000);
    assert_eq!(ops[1].id.wall_ms, 2000);
    assert_eq!(ops[2].id.wall_ms, 3000);
}

#[test]
fn oplog_in_memory() {
    let mut oplog = Oplog::in_memory();

    let op = test_op(1000, "test-1");
    assert!(oplog.append(&op).unwrap());
    assert!(!oplog.append(&op).unwrap()); // Dedup still works

    assert_eq!(oplog.len(), 1);
    assert!(oplog.contains(&op.id));

    // ops_since returns empty for in-memory (no file to read)
    let ops = oplog.all_ops().unwrap();
    assert!(ops.is_empty());
}
