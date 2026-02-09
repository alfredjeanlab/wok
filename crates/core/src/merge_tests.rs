// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::hlc::Hlc;
use crate::issue::{IssueType, Relation, Status};
use crate::op::OpPayload;
use yare::parameterized;

fn test_db() -> Database {
    Database::open_in_memory().unwrap()
}

#[test]
fn merge_create_issue() {
    let mut db = test_db();

    let op = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );

    assert!(db.apply(&op).unwrap());
    assert!(db.issue_exists("test-1").unwrap());

    let issue = db.get_issue("test-1").unwrap();
    assert_eq!(issue.title, "Title");
    assert_eq!(issue.issue_type, IssueType::Task);
}

#[test]
fn merge_create_issue_duplicate() {
    let mut db = test_db();

    let op = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );

    assert!(db.apply(&op).unwrap()); // First apply succeeds
    assert!(!db.apply(&op).unwrap()); // Duplicate is a no-op
}

#[test]
fn merge_set_status() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );
    db.apply(&create).unwrap();

    let set_status = Op::new(
        Hlc::new(2000, 0, 1),
        OpPayload::set_status("test-1".into(), Status::InProgress, None),
    );
    assert!(db.apply(&set_status).unwrap());

    let issue = db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::InProgress);
    assert_eq!(issue.last_status_hlc, Some(Hlc::new(2000, 0, 1)));
}

#[test]
fn merge_set_status_last_wins() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );
    db.apply(&create).unwrap();

    // Apply status changes in order
    let first = Op::new(
        Hlc::new(2000, 0, 1),
        OpPayload::set_status("test-1".into(), Status::InProgress, None),
    );
    let second =
        Op::new(Hlc::new(3000, 0, 1), OpPayload::set_status("test-1".into(), Status::Done, None));

    db.apply(&first).unwrap();
    db.apply(&second).unwrap();

    let issue = db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Done);

    // Now apply an older status change - should be ignored
    let stale =
        Op::new(Hlc::new(2500, 0, 1), OpPayload::set_status("test-1".into(), Status::Closed, None));
    assert!(!db.apply(&stale).unwrap()); // No-op because HLC is older

    let issue = db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Done); // Still Done
}

#[test]
fn merge_set_title() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Old".into()),
    );
    db.apply(&create).unwrap();

    let set_title =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::set_title("test-1".into(), "New".into()));
    assert!(db.apply(&set_title).unwrap());

    let issue = db.get_issue("test-1").unwrap();
    assert_eq!(issue.title, "New");
    assert_eq!(issue.last_title_hlc, Some(Hlc::new(2000, 0, 1)));
}

#[test]
fn merge_set_type() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );
    db.apply(&create).unwrap();

    let set_type =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::set_type("test-1".into(), IssueType::Bug));
    assert!(db.apply(&set_type).unwrap());

    let issue = db.get_issue("test-1").unwrap();
    assert_eq!(issue.issue_type, IssueType::Bug);
}

#[test]
fn merge_add_label() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );
    db.apply(&create).unwrap();

    let add_label =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::add_label("test-1".into(), "urgent".into()));
    assert!(db.apply(&add_label).unwrap());

    let labels = db.get_labels("test-1").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
}

#[test]
fn merge_remove_label() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );
    db.apply(&create).unwrap();

    let add_label =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::add_label("test-1".into(), "urgent".into()));
    db.apply(&add_label).unwrap();

    let remove_label =
        Op::new(Hlc::new(3000, 0, 1), OpPayload::remove_label("test-1".into(), "urgent".into()));
    assert!(db.apply(&remove_label).unwrap());

    let labels = db.get_labels("test-1").unwrap();
    assert!(!labels.contains(&"urgent".to_string()));
}

#[test]
fn merge_add_note() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
    );
    db.apply(&create).unwrap();

    let add_note = Op::new(
        Hlc::new(2000, 0, 1),
        OpPayload::add_note("test-1".into(), "Note content".into(), Status::Todo),
    );
    assert!(db.apply(&add_note).unwrap());

    let notes = db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Note content");
}

#[test]
fn merge_add_dep() {
    let mut db = test_db();

    let create1 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Issue 1".into()),
    );
    let create2 = Op::new(
        Hlc::new(1001, 0, 1),
        OpPayload::create_issue("test-2".into(), IssueType::Task, "Issue 2".into()),
    );
    db.apply(&create1).unwrap();
    db.apply(&create2).unwrap();

    let add_dep = Op::new(
        Hlc::new(2000, 0, 1),
        OpPayload::add_dep("test-1".into(), "test-2".into(), Relation::Blocks),
    );
    assert!(db.apply(&add_dep).unwrap());

    let blockers = db.get_blockers("test-2").unwrap();
    assert!(blockers.contains(&"test-1".to_string()));
}

#[test]
fn merge_remove_dep() {
    let mut db = test_db();

    let create1 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Issue 1".into()),
    );
    let create2 = Op::new(
        Hlc::new(1001, 0, 1),
        OpPayload::create_issue("test-2".into(), IssueType::Task, "Issue 2".into()),
    );
    db.apply(&create1).unwrap();
    db.apply(&create2).unwrap();

    let add_dep = Op::new(
        Hlc::new(2000, 0, 1),
        OpPayload::add_dep("test-1".into(), "test-2".into(), Relation::Blocks),
    );
    db.apply(&add_dep).unwrap();

    let remove_dep = Op::new(
        Hlc::new(3000, 0, 1),
        OpPayload::remove_dep("test-1".into(), "test-2".into(), Relation::Blocks),
    );
    assert!(db.apply(&remove_dep).unwrap());

    let blockers = db.get_blockers("test-2").unwrap();
    assert!(!blockers.contains(&"test-1".to_string()));
}

#[test]
fn merge_on_nonexistent_issue() {
    let mut db = test_db();

    let set_status = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::set_status("nonexistent".into(), Status::Done, None),
    );
    assert!(!db.apply(&set_status).unwrap()); // No-op, issue doesn't exist
}

#[test]
fn merge_apply_all() {
    let mut db = test_db();

    let ops = vec![
        Op::new(
            Hlc::new(1000, 0, 1),
            OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()),
        ),
        Op::new(
            Hlc::new(2000, 0, 1),
            OpPayload::set_status("test-1".into(), Status::InProgress, None),
        ),
        Op::new(Hlc::new(3000, 0, 1), OpPayload::add_label("test-1".into(), "urgent".into())),
    ];

    let applied = db.apply_all(&ops).unwrap();
    assert_eq!(applied, 3);

    let issue = db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::InProgress);

    let labels = db.get_labels("test-1").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
}

/// Tests merge idempotence: applying the same op twice equals applying once
#[parameterized(
    task = { IssueType::Task, "Task title" },
    bug = { IssueType::Bug, "Bug description" },
    feature = { IssueType::Feature, "Feature summary" },
)]
fn merge_is_idempotent(issue_type: IssueType, title: &str) {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let op = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), issue_type, title.into()),
    );

    // Apply once vs twice
    db1.apply(&op).unwrap();
    db2.apply(&op).unwrap();
    db2.apply(&op).unwrap();

    let issue1 = db1.get_issue("test-1").unwrap();
    let issue2 = db2.get_issue("test-1").unwrap();
    assert_eq!(issue1.id, issue2.id);
    assert_eq!(issue1.title, issue2.title);
    assert_eq!(issue1.issue_type, issue2.issue_type);
}

/// Tests SetStatus HLC ordering: later HLC wins regardless of application order
#[parameterized(
    todo_to_done = { Status::Todo, Status::Done },
    in_progress_to_closed = { Status::InProgress, Status::Closed },
    done_to_todo = { Status::Done, Status::Todo },
)]
fn set_status_later_hlc_wins(status1: Status, status2: Status) {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let create = Op::new(
        Hlc::new(500, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Test".into()),
    );
    db1.apply(&create).unwrap();
    db2.apply(&create).unwrap();

    let op1 = Op::new(Hlc::new(1000, 0, 1), OpPayload::set_status("test-1".into(), status1, None));
    let op2 = Op::new(Hlc::new(2000, 0, 1), OpPayload::set_status("test-1".into(), status2, None));

    // db1: forward order, db2: reverse order
    db1.apply(&op1).unwrap();
    db1.apply(&op2).unwrap();
    db2.apply(&op2).unwrap();
    db2.apply(&op1).unwrap();

    // Both converge to status2 (later HLC)
    assert_eq!(db1.get_issue("test-1").unwrap().status, status2);
    assert_eq!(db2.get_issue("test-1").unwrap().status, status2);
}

/// Tests SetTitle HLC ordering: later HLC wins regardless of application order
#[parameterized(
    short_titles = { "Alpha", "Beta" },
    long_titles = { "First long title", "Second long title" },
    special_chars = { "Title with spaces", "Title_with_underscores" },
)]
fn set_title_later_hlc_wins(title1: &str, title2: &str) {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let create = Op::new(
        Hlc::new(500, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Initial".into()),
    );
    db1.apply(&create).unwrap();
    db2.apply(&create).unwrap();

    let op1 = Op::new(Hlc::new(1000, 0, 1), OpPayload::set_title("test-1".into(), title1.into()));
    let op2 = Op::new(Hlc::new(2000, 0, 1), OpPayload::set_title("test-1".into(), title2.into()));

    // db1: forward order, db2: reverse order
    db1.apply(&op1).unwrap();
    db1.apply(&op2).unwrap();
    db2.apply(&op2).unwrap();
    db2.apply(&op1).unwrap();

    // Both converge to title2 (later HLC)
    assert_eq!(db1.get_issue("test-1").unwrap().title, title2);
    assert_eq!(db2.get_issue("test-1").unwrap().title, title2);
}

/// Tests AddLabel commutativity: order of label additions doesn't matter
#[parameterized(
    two_tags = { "urgent", "backend" },
    similar_tags = { "mod:cli", "mod:core" },
    overlapping = { "test", "testing" },
)]
fn add_label_commutative(tag1: &str, tag2: &str) {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let create = Op::new(
        Hlc::new(100, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Test".into()),
    );
    db1.apply(&create).unwrap();
    db2.apply(&create).unwrap();

    let op1 = Op::new(Hlc::new(1000, 0, 1), OpPayload::add_label("test-1".into(), tag1.into()));
    let op2 = Op::new(Hlc::new(2000, 0, 1), OpPayload::add_label("test-1".into(), tag2.into()));

    // db1: op1 then op2, db2: op2 then op1
    db1.apply(&op1).unwrap();
    db1.apply(&op2).unwrap();
    db2.apply(&op2).unwrap();
    db2.apply(&op1).unwrap();

    let mut tags1 = db1.get_labels("test-1").unwrap();
    let mut tags2 = db2.get_labels("test-1").unwrap();
    tags1.sort();
    tags2.sort();
    assert_eq!(tags1, tags2);
}

/// Tests CreateIssue first-write-wins: first received create wins
#[parameterized(
    task_vs_bug = { IssueType::Task, "First", IssueType::Bug, "Second" },
    feature_vs_task = { IssueType::Feature, "Feature title", IssueType::Task, "Task title" },
)]
fn create_issue_first_wins(type1: IssueType, title1: &str, type2: IssueType, title2: &str) {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let op1 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("test-1".into(), type1, title1.into()),
    );
    let op2 = Op::new(
        Hlc::new(2000, 0, 1),
        OpPayload::create_issue("test-1".into(), type2, title2.into()),
    );

    // db1 receives op1 first, db2 receives op2 first
    db1.apply(&op1).unwrap();
    db1.apply(&op2).unwrap();
    db2.apply(&op2).unwrap();
    db2.apply(&op1).unwrap();

    // First received wins for each database
    let issue1 = db1.get_issue("test-1").unwrap();
    assert_eq!(issue1.issue_type, type1);
    assert_eq!(issue1.title, title1);

    let issue2 = db2.get_issue("test-1").unwrap();
    assert_eq!(issue2.issue_type, type2);
    assert_eq!(issue2.title, title2);
}

/// Tests HLC tiebreaker: when wall_ms is equal, counter breaks the tie
#[parameterized(
    counter_1_vs_2 = { 1, 2 },
    counter_0_vs_1 = { 0, 1 },
    counter_5_vs_10 = { 5, 10 },
)]
fn hlc_counter_tiebreaker(counter1: u32, counter2: u32) {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let create = Op::new(
        Hlc::new(500, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Test".into()),
    );
    db1.apply(&create).unwrap();
    db2.apply(&create).unwrap();

    // Same wall_ms, different counter
    let op1 = Op::new(
        Hlc::new(1000, counter1, 1),
        OpPayload::set_status("test-1".into(), Status::InProgress, None),
    );
    let op2 = Op::new(
        Hlc::new(1000, counter2, 1),
        OpPayload::set_status("test-1".into(), Status::Done, None),
    );

    // Apply in different orders
    db1.apply(&op1).unwrap();
    db1.apply(&op2).unwrap();
    db2.apply(&op2).unwrap();
    db2.apply(&op1).unwrap();

    // Higher counter wins (op2 has counter2 > counter1)
    assert_eq!(db1.get_issue("test-1").unwrap().status, Status::Done);
    assert_eq!(db2.get_issue("test-1").unwrap().status, Status::Done);
}

/// Tests HLC tiebreaker: when wall_ms and counter are equal, node_id breaks the tie
#[parameterized(
    node_1_vs_2 = { 1, 2 },
    node_1_vs_99 = { 1, 99 },
)]
fn hlc_node_id_tiebreaker(node1: u32, node2: u32) {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let create = Op::new(
        Hlc::new(500, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Test".into()),
    );
    db1.apply(&create).unwrap();
    db2.apply(&create).unwrap();

    // Same wall_ms and counter, different node_id
    let op1 = Op::new(
        Hlc::new(1000, 0, node1),
        OpPayload::set_title("test-1".into(), "Title from node 1".into()),
    );
    let op2 = Op::new(
        Hlc::new(1000, 0, node2),
        OpPayload::set_title("test-1".into(), "Title from node 2".into()),
    );

    // Apply in different orders
    db1.apply(&op1).unwrap();
    db1.apply(&op2).unwrap();
    db2.apply(&op2).unwrap();
    db2.apply(&op1).unwrap();

    // Higher node_id wins
    assert_eq!(db1.get_issue("test-1").unwrap().title, "Title from node 2");
    assert_eq!(db2.get_issue("test-1").unwrap().title, "Title from node 2");
}

/// Tests boundary conditions for titles
#[parameterized(
    empty_to_nonempty = { "", "Non-empty" },
    nonempty_to_empty = { "Something", "" },
    unicode = { "Hello", "こんにちは" },
    long_title = { "Short", "This is a very long title that contains many words and should still work correctly in the system" },
)]
fn title_boundary_conditions(title1: &str, title2: &str) {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(500, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Initial".into()),
    );
    db.apply(&create).unwrap();

    let op1 = Op::new(Hlc::new(1000, 0, 1), OpPayload::set_title("test-1".into(), title1.into()));
    let op2 = Op::new(Hlc::new(2000, 0, 1), OpPayload::set_title("test-1".into(), title2.into()));

    db.apply(&op1).unwrap();
    db.apply(&op2).unwrap();

    assert_eq!(db.get_issue("test-1").unwrap().title, title2);
}

/// Tests HLC boundary values
#[parameterized(
    zero_wall = { 0u64, 1u64 },
    large_wall = { u64::MAX - 1, u64::MAX },
)]
fn hlc_boundary_values(val1: u64, val2: u64) {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(100, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Test".into()),
    );
    db.apply(&create).unwrap();

    let op1 = Op::new(
        Hlc::new(val1, 0, 1),
        OpPayload::set_status("test-1".into(), Status::InProgress, None),
    );
    let op2 =
        Op::new(Hlc::new(val2, 0, 1), OpPayload::set_status("test-1".into(), Status::Done, None));

    db.apply(&op1).unwrap();
    db.apply(&op2).unwrap();

    // Later HLC wins
    assert_eq!(db.get_issue("test-1").unwrap().status, Status::Done);
}

/// Tests that 3 label additions are commutative (all orderings converge)
#[test]
fn three_labels_commutative() {
    let labels = ["alpha", "beta", "gamma"];
    let orderings: [[usize; 3]; 6] =
        [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];

    let mut results: Vec<Vec<String>> = Vec::new();

    for order in &orderings {
        let mut db = test_db();
        let create = Op::new(
            Hlc::new(100, 0, 1),
            OpPayload::create_issue("test-1".into(), IssueType::Task, "Test".into()),
        );
        db.apply(&create).unwrap();

        for &i in order {
            let op = Op::new(
                Hlc::new(1000 + i as u64, 0, 1),
                OpPayload::add_label("test-1".into(), labels[i].into()),
            );
            db.apply(&op).unwrap();
        }

        let mut result = db.get_labels("test-1").unwrap();
        result.sort();
        results.push(result);
    }

    // All orderings should produce the same result
    for result in &results[1..] {
        assert_eq!(&results[0], result);
    }
}

/// Tests that mixed operations (status + title + label) converge regardless of order
#[test]
fn mixed_ops_convergence() {
    let mut db1 = test_db();
    let mut db2 = test_db();

    let create = Op::new(
        Hlc::new(100, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Initial".into()),
    );
    db1.apply(&create).unwrap();
    db2.apply(&create).unwrap();

    let status_op = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::set_status("test-1".into(), Status::InProgress, None),
    );
    let title_op =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::set_title("test-1".into(), "Updated".into()));
    let label_op =
        Op::new(Hlc::new(3000, 0, 1), OpPayload::add_label("test-1".into(), "important".into()));

    // db1: status, title, label
    db1.apply(&status_op).unwrap();
    db1.apply(&title_op).unwrap();
    db1.apply(&label_op).unwrap();

    // db2: label, status, title
    db2.apply(&label_op).unwrap();
    db2.apply(&status_op).unwrap();
    db2.apply(&title_op).unwrap();

    // Both should converge to the same state
    let issue1 = db1.get_issue("test-1").unwrap();
    let issue2 = db2.get_issue("test-1").unwrap();

    assert_eq!(issue1.status, issue2.status);
    assert_eq!(issue1.title, issue2.title);
    assert_eq!(db1.get_labels("test-1").unwrap(), db2.get_labels("test-1").unwrap());
}

/// Tests concurrent updates to the same field from different nodes
#[test]
fn concurrent_updates_same_field() {
    let mut db1 = test_db();
    let mut db2 = test_db();
    let mut db3 = test_db();

    let create = Op::new(
        Hlc::new(100, 0, 1),
        OpPayload::create_issue("test-1".into(), IssueType::Task, "Initial".into()),
    );
    db1.apply(&create).unwrap();
    db2.apply(&create).unwrap();
    db3.apply(&create).unwrap();

    // Three nodes update title at the "same" wall time, different node IDs
    let op_node1 =
        Op::new(Hlc::new(1000, 0, 1), OpPayload::set_title("test-1".into(), "From node 1".into()));
    let op_node2 =
        Op::new(Hlc::new(1000, 0, 2), OpPayload::set_title("test-1".into(), "From node 2".into()));
    let op_node3 =
        Op::new(Hlc::new(1000, 0, 3), OpPayload::set_title("test-1".into(), "From node 3".into()));

    // Apply in different orders to each database
    db1.apply(&op_node1).unwrap();
    db1.apply(&op_node2).unwrap();
    db1.apply(&op_node3).unwrap();

    db2.apply(&op_node3).unwrap();
    db2.apply(&op_node1).unwrap();
    db2.apply(&op_node2).unwrap();

    db3.apply(&op_node2).unwrap();
    db3.apply(&op_node3).unwrap();
    db3.apply(&op_node1).unwrap();

    // All should converge to node 3's update (highest node_id)
    assert_eq!(db1.get_issue("test-1").unwrap().title, "From node 3");
    assert_eq!(db2.get_issue("test-1").unwrap().title, "From node 3");
    assert_eq!(db3.get_issue("test-1").unwrap().title, "From node 3");
}

#[test]
fn merge_config_rename_updates_all_tables() {
    let mut db = test_db();

    // Create an issue with labels, notes, and dependencies
    let create1 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("old-1".into(), IssueType::Task, "Issue 1".into()),
    );
    let create2 = Op::new(
        Hlc::new(1001, 0, 1),
        OpPayload::create_issue("old-2".into(), IssueType::Bug, "Issue 2".into()),
    );
    db.apply(&create1).unwrap();
    db.apply(&create2).unwrap();

    // Add label
    let add_label =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::add_label("old-1".into(), "urgent".into()));
    db.apply(&add_label).unwrap();

    // Add note
    let add_note = Op::new(
        Hlc::new(3000, 0, 1),
        OpPayload::add_note("old-1".into(), "A note".into(), Status::Todo),
    );
    db.apply(&add_note).unwrap();

    // Add dependency
    let add_dep = Op::new(
        Hlc::new(4000, 0, 1),
        OpPayload::add_dep("old-1".into(), "old-2".into(), Relation::Blocks),
    );
    db.apply(&add_dep).unwrap();

    // Apply config rename
    let rename =
        Op::new(Hlc::new(5000, 0, 1), OpPayload::config_rename("old".into(), "new".into()));
    assert!(db.apply(&rename).unwrap());

    // Verify issues were renamed
    assert!(db.issue_exists("new-1").unwrap());
    assert!(db.issue_exists("new-2").unwrap());
    assert!(!db.issue_exists("old-1").unwrap());
    assert!(!db.issue_exists("old-2").unwrap());

    // Verify labels were updated
    let labels = db.get_labels("new-1").unwrap();
    assert!(labels.contains(&"urgent".to_string()));

    // Verify notes were updated
    let notes = db.get_notes("new-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "A note");

    // Verify dependencies were updated
    let blockers = db.get_blockers("new-2").unwrap();
    assert!(blockers.contains(&"new-1".to_string()));
}

#[test]
fn merge_config_rename_is_idempotent() {
    let mut db = test_db();

    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("old-1".into(), IssueType::Task, "Issue".into()),
    );
    db.apply(&create).unwrap();

    let rename =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::config_rename("old".into(), "new".into()));

    // Apply once
    db.apply(&rename).unwrap();
    assert!(db.issue_exists("new-1").unwrap());

    // Apply again (idempotent)
    db.apply(&rename).unwrap();
    assert!(db.issue_exists("new-1").unwrap());
    assert!(!db.issue_exists("old-1").unwrap());

    // Verify only one issue exists
    let issues = db.list_issues(None, None, None).unwrap();
    assert_eq!(issues.len(), 1);
}

#[test]
fn merge_config_rename_partial_match() {
    let mut db = test_db();

    // Create issues with similar prefixes
    let create1 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("old-1".into(), IssueType::Task, "Should rename".into()),
    );
    let create2 = Op::new(
        Hlc::new(1001, 0, 1),
        OpPayload::create_issue("older-1".into(), IssueType::Task, "Should NOT rename".into()),
    );
    let create3 = Op::new(
        Hlc::new(1002, 0, 1),
        OpPayload::create_issue("oldest-1".into(), IssueType::Task, "Should NOT rename".into()),
    );
    db.apply(&create1).unwrap();
    db.apply(&create2).unwrap();
    db.apply(&create3).unwrap();

    // Rename only "old" prefix
    let rename =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::config_rename("old".into(), "new".into()));
    db.apply(&rename).unwrap();

    // Verify correct renames
    assert!(db.issue_exists("new-1").unwrap()); // old-1 -> new-1
    assert!(db.issue_exists("older-1").unwrap()); // unchanged
    assert!(db.issue_exists("oldest-1").unwrap()); // unchanged
    assert!(!db.issue_exists("old-1").unwrap()); // renamed
}

#[test]
fn merge_config_rename_no_matching_issues() {
    let mut db = test_db();

    // Create issues with different prefix
    let create = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("proj-1".into(), IssueType::Task, "Issue".into()),
    );
    db.apply(&create).unwrap();

    // Rename a prefix that doesn't exist
    let rename =
        Op::new(Hlc::new(2000, 0, 1), OpPayload::config_rename("nonexistent".into(), "new".into()));
    assert!(db.apply(&rename).unwrap()); // Still returns true (operation applied)

    // Verify issue unchanged
    assert!(db.issue_exists("proj-1").unwrap());
}
