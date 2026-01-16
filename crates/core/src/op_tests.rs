// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::hlc::Hlc;
use yare::parameterized;

fn hlc() -> Hlc {
    Hlc::new(1000, 0, 1)
}

#[parameterized(
    create_issue = { OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into()), "test-1" },
    set_status = { OpPayload::set_status("test-2".into(), Status::Done, None), "test-2" },
    set_title = { OpPayload::set_title("test-3".into(), "New Title".into()), "test-3" },
    set_type = { OpPayload::set_type("test-4".into(), IssueType::Bug), "test-4" },
    add_label = { OpPayload::add_label("test-5".into(), "urgent".into()), "test-5" },
    remove_label = { OpPayload::remove_label("test-6".into(), "done".into()), "test-6" },
    add_note = { OpPayload::add_note("test-7".into(), "Note".into(), Status::Todo), "test-7" },
    add_dep = { OpPayload::add_dep("test-8".into(), "test-9".into(), Relation::Blocks), "test-8" },
    remove_dep = { OpPayload::remove_dep("test-10".into(), "test-11".into(), Relation::TrackedBy), "test-10" },
    config_rename = { OpPayload::config_rename("old".into(), "new".into()), "" },
)]
fn op_issue_id_extraction(payload: OpPayload, expected_id: &str) {
    let op = Op::new(hlc(), payload);
    assert_eq!(op.issue_id(), expected_id);
}

#[test]
fn op_ordering() {
    let op1 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("a".into(), IssueType::Task, "A".into()),
    );
    let op2 = Op::new(
        Hlc::new(2000, 0, 1),
        OpPayload::create_issue("b".into(), IssueType::Task, "B".into()),
    );
    let op3 = Op::new(
        Hlc::new(1000, 1, 1),
        OpPayload::create_issue("c".into(), IssueType::Task, "C".into()),
    );

    assert!(op1 < op2);
    assert!(op1 < op3);
    assert!(op3 < op2);
}

#[parameterized(
    create_feature = { Op::new(Hlc::new(1000, 0, 1), OpPayload::create_issue("test-1".into(), IssueType::Feature, "Feature title".into())) },
    set_status_in_progress = { Op::new(Hlc::new(2000, 1, 2), OpPayload::set_status("test-1".into(), Status::InProgress, None)) },
    set_status_with_reason = { Op::new(Hlc::new(3000, 0, 1), OpPayload::set_status("test-1".into(), Status::Closed, Some("Won't fix".into()))) },
    add_label = { Op::new(Hlc::new(4000, 0, 1), OpPayload::add_label("test-1".into(), "urgent".into())) },
    add_note = { Op::new(Hlc::new(5000, 0, 1), OpPayload::add_note("test-1".into(), "A note".into(), Status::InProgress)) },
    add_dep = { Op::new(Hlc::new(6000, 0, 1), OpPayload::add_dep("test-1".into(), "test-2".into(), Relation::Blocks)) },
    config_rename = { Op::new(Hlc::new(7000, 0, 1), OpPayload::config_rename("old".into(), "new".into())) },
)]
fn op_serialization_roundtrip(op: Op) {
    let json = serde_json::to_string(&op).unwrap();
    let parsed: Op = serde_json::from_str(&json).unwrap();
    assert_eq!(op, parsed);
}

#[test]
fn op_payload_json_format() {
    let payload = OpPayload::create_issue("test-1".into(), IssueType::Task, "Title".into());
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"type\":\"create_issue\""));
    assert!(json.contains("\"id\":\"test-1\""));

    let payload = OpPayload::set_status("test-1".into(), Status::Done, Some("Complete".into()));
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"type\":\"set_status\""));
    assert!(json.contains("\"reason\":\"Complete\""));

    let payload = OpPayload::config_rename("wk".into(), "proj".into());
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"type\":\"config_rename\""));
    assert!(json.contains("\"old_prefix\":\"wk\""));
    assert!(json.contains("\"new_prefix\":\"proj\""));
}

#[test]
fn op_equality() {
    let op1 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("a".into(), IssueType::Task, "A".into()),
    );
    let op2 = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::create_issue("a".into(), IssueType::Task, "A".into()),
    );
    let op3 = Op::new(
        Hlc::new(1000, 0, 2),
        OpPayload::create_issue("a".into(), IssueType::Task, "A".into()),
    );

    assert_eq!(op1, op2);
    assert_ne!(op1, op3);
}
