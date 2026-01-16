// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for the offline queue module.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::test_helpers::make_test_op;
use super::*;
use tempfile::tempdir;

#[test]
fn test_queue_empty_file() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("empty.jsonl");

    // Create empty file
    std::fs::write(&queue_path, "").unwrap();

    let queue = OfflineQueue::open(&queue_path).unwrap();
    assert!(queue.is_empty().unwrap());
    assert_eq!(queue.len().unwrap(), 0);
}

#[test]
fn test_queue_file_with_blank_lines() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("blanks.jsonl");

    // Create file with blank lines
    let mut queue = OfflineQueue::open(&queue_path).unwrap();
    queue.enqueue(&make_test_op(1000)).unwrap();

    // Manually add blank lines
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&queue_path)
        .unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "   ").unwrap();

    queue.enqueue(&make_test_op(2000)).unwrap();

    // Should skip blank lines
    let ops = queue.peek_all().unwrap();
    assert_eq!(ops.len(), 2);
}

#[test]
fn test_queue_remove_all() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("queue.jsonl");
    let mut queue = OfflineQueue::open(&queue_path).unwrap();

    queue.enqueue(&make_test_op(1000)).unwrap();
    queue.enqueue(&make_test_op(2000)).unwrap();
    queue.enqueue(&make_test_op(3000)).unwrap();

    // Remove all by removing count >= len
    queue.remove_first(10).unwrap();

    assert!(queue.is_empty().unwrap());
}

#[test]
fn test_enqueue_and_peek() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("queue.jsonl");
    let mut queue = OfflineQueue::open(&queue_path).unwrap();

    assert!(queue.is_empty().unwrap());

    let op1 = make_test_op(1000);
    let op2 = make_test_op(2000);

    queue.enqueue(&op1).unwrap();
    queue.enqueue(&op2).unwrap();

    assert_eq!(queue.len().unwrap(), 2);

    let ops = queue.peek_all().unwrap();
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].id.wall_ms, 1000);
    assert_eq!(ops[1].id.wall_ms, 2000);
}

#[test]
fn test_clear() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("queue.jsonl");
    let mut queue = OfflineQueue::open(&queue_path).unwrap();

    queue.enqueue(&make_test_op(1000)).unwrap();
    queue.enqueue(&make_test_op(2000)).unwrap();

    assert_eq!(queue.len().unwrap(), 2);

    queue.clear().unwrap();

    assert!(queue.is_empty().unwrap());
}

#[test]
fn test_remove_first() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("queue.jsonl");
    let mut queue = OfflineQueue::open(&queue_path).unwrap();

    queue.enqueue(&make_test_op(1000)).unwrap();
    queue.enqueue(&make_test_op(2000)).unwrap();
    queue.enqueue(&make_test_op(3000)).unwrap();

    queue.remove_first(1).unwrap();

    let ops = queue.peek_all().unwrap();
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].id.wall_ms, 2000);
    assert_eq!(ops[1].id.wall_ms, 3000);
}

#[test]
fn test_persistence() {
    let dir = tempdir().unwrap();
    let queue_path = dir.path().join("queue.jsonl");

    // Write ops with one queue instance
    {
        let mut queue = OfflineQueue::open(&queue_path).unwrap();
        queue.enqueue(&make_test_op(1000)).unwrap();
        queue.enqueue(&make_test_op(2000)).unwrap();
    }

    // Read with new instance
    {
        let queue = OfflineQueue::open(&queue_path).unwrap();
        let ops = queue.peek_all().unwrap();
        assert_eq!(ops.len(), 2);
    }
}
