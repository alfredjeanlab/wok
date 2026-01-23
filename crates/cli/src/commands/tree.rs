// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;
use crate::display::{format_tree_child, format_tree_root};
use crate::error::Result;

use super::open_db;

pub fn run(id: &str) -> Result<()> {
    let (db, _, _) = open_db()?;
    run_impl(&db, id)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, id: &str) -> Result<()> {
    let issue = db.get_issue(id)?;

    // Get blockers for root issue
    let blockers = db.get_transitive_blockers(id)?;
    let blocked_by = if blockers.is_empty() {
        None
    } else {
        Some(blockers.as_slice())
    };

    // Print root issue
    println!("{}", format_tree_root(&issue, blocked_by));

    // Get and print children
    let children = db.get_tracked(id)?;
    print_children(db, &children, "")?;

    Ok(())
}

fn print_children(db: &crate::db::Database, children: &[String], prefix: &str) -> Result<()> {
    for (i, child_id) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        let issue = db.get_issue(child_id)?;

        // Get transitive blockers for this issue (already filtered for open status)
        let blockers = db.get_transitive_blockers(child_id)?;

        let blocked_by = if blockers.is_empty() {
            None
        } else {
            Some(blockers.as_slice())
        };

        for line in format_tree_child(&issue, prefix, is_last, blocked_by) {
            println!("{}", line);
        }

        // Recursively print grandchildren
        let grandchildren = db.get_tracked(child_id)?;
        if !grandchildren.is_empty() {
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            print_children(db, &grandchildren, &child_prefix)?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tests;
