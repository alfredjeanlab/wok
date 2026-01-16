// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::models::{Issue, IssueType};

fn create_test_issue(db: &Database, id: &str) {
    let issue = Issue::new(id.to_string(), IssueType::Task, format!("Issue {}", id));
    db.create_issue(&issue).unwrap();
}

#[test]
fn test_add_and_get_dependency() {
    let db = Database::open_in_memory().unwrap();
    create_test_issue(&db, "a");
    create_test_issue(&db, "b");

    db.add_dependency("a", "b", Relation::Blocks).unwrap();

    let deps = db.get_deps_from("a").unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].to_id, "b");
    assert_eq!(deps[0].relation, Relation::Blocks);
}

#[test]
fn test_self_dependency() {
    let db = Database::open_in_memory().unwrap();
    create_test_issue(&db, "a");

    let result = db.add_dependency("a", "a", Relation::Blocks);
    assert!(matches!(result, Err(Error::SelfDependency)));
}

#[test]
fn test_cycle_detection() {
    let db = Database::open_in_memory().unwrap();
    create_test_issue(&db, "a");
    create_test_issue(&db, "b");
    create_test_issue(&db, "c");

    db.add_dependency("a", "b", Relation::Blocks).unwrap();
    db.add_dependency("b", "c", Relation::Blocks).unwrap();

    // This should fail - would create a cycle a -> b -> c -> a
    let result = db.add_dependency("c", "a", Relation::Blocks);
    assert!(matches!(result, Err(Error::CycleDetected)));
}

#[test]
fn test_remove_dependency() {
    let db = Database::open_in_memory().unwrap();
    create_test_issue(&db, "a");
    create_test_issue(&db, "b");

    db.add_dependency("a", "b", Relation::Blocks).unwrap();
    db.remove_dependency("a", "b", Relation::Blocks).unwrap();

    let deps = db.get_deps_from("a").unwrap();
    assert!(deps.is_empty());
}
