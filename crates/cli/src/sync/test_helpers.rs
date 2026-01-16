// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Shared test helpers for sync module tests.

use wk_core::issue::IssueType;
use wk_core::{Hlc, Op, OpPayload};

/// Create a test operation with the given wall clock timestamp.
pub fn make_test_op(wall_ms: u64) -> Op {
    make_test_op_with_node(wall_ms, 1)
}

/// Create a test operation with the given wall clock timestamp and node ID.
pub fn make_test_op_with_node(wall_ms: u64, node_id: u32) -> Op {
    Op::new(
        Hlc::new(wall_ms, 0, node_id),
        OpPayload::create_issue(
            format!("test-{}-{}", wall_ms, node_id),
            IssueType::Task,
            format!("Test {} from node {}", wall_ms, node_id),
        ),
    )
}
