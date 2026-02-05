// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for cycle detection in dependencies.
//! Converted from tests/specs/cli/edge_cases/cycles.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::common::{init_temp, wk, TempDir};
use predicates::prelude::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Self-referencing dependencies - Cannot create dependency from issue to itself
// =============================================================================

#[parameterized(
    blocks = { "blocks" },
    tracks = { "tracks" },
)]
fn cannot_create_self_referencing_dependency(relation: &str) {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Self-Ref Task");

    wk().arg("dep")
        .arg(&a)
        .arg(relation)
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Direct cycles - A blocks B, B blocks A is a cycle
// =============================================================================

#[test]
fn cannot_create_direct_cycle() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "DirectCycle Task A");
    let b = create_issue(&temp, "task", "DirectCycle Task B");

    // A blocks B succeeds
    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    // B blocks A fails (would create cycle)
    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Transitive cycles - A->B->C->A is a cycle
// =============================================================================

#[parameterized(
    three_nodes = { 3 },
    four_nodes = { 4 },
)]
fn cannot_create_transitive_cycle(node_count: usize) {
    let temp = init_temp();

    // Create the issues
    let issues: Vec<String> = (0..node_count)
        .map(|i| {
            create_issue(
                &temp,
                "task",
                &format!("TransCycle{} Task {}", node_count, i),
            )
        })
        .collect();

    // Create chain: 0 blocks 1, 1 blocks 2, etc.
    for i in 0..node_count - 1 {
        wk().arg("dep")
            .arg(&issues[i])
            .arg("blocks")
            .arg(&issues[i + 1])
            .current_dir(temp.path())
            .assert()
            .success();
    }

    // Try to close the cycle: last blocks first
    wk().arg("dep")
        .arg(&issues[node_count - 1])
        .arg("blocks")
        .arg(&issues[0])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// =============================================================================
// Valid DAG patterns - These should succeed
// =============================================================================

#[test]
fn valid_dag_with_shared_node_is_allowed() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "SharedNode Task A");
    let b = create_issue(&temp, "task", "SharedNode Task B");
    let c = create_issue(&temp, "task", "SharedNode Task C");

    // A and B both block C (diamond pattern, no cycle)
    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn valid_chain_is_allowed() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "Chain Task A");
    let b = create_issue(&temp, "task", "Chain Task B");
    let c = create_issue(&temp, "task", "Chain Task C");
    let d = create_issue(&temp, "task", "Chain Task D");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&c)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&c)
        .arg("blocks")
        .arg(&d)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn parallel_chains_are_allowed() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "ParallelChain Task A");
    let b = create_issue(&temp, "task", "ParallelChain Task B");
    let c = create_issue(&temp, "task", "ParallelChain Task C");
    let d = create_issue(&temp, "task", "ParallelChain Task D");

    // Two independent chains: A->B and C->D
    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("dep")
        .arg(&c)
        .arg("blocks")
        .arg(&d)
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Error messages - Cycle detection should provide helpful feedback
// =============================================================================

#[test]
fn cycle_detection_error_message_is_helpful() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "CycleErr Task A");
    let b = create_issue(&temp, "task", "CycleErr Task B");

    wk().arg("dep")
        .arg(&a)
        .arg("blocks")
        .arg(&b)
        .current_dir(temp.path())
        .assert()
        .success();

    // Should fail with a helpful message mentioning "cycle" or "circular"
    wk().arg("dep")
        .arg(&b)
        .arg("blocks")
        .arg(&a)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cycle").or(predicate::str::contains("circular")));
}
