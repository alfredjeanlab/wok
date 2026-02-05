// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;
use crate::display::{format_tree_child, format_tree_root, RelationType};
use crate::error::Result;

use super::open_db;

pub fn run(ids: &[String]) -> Result<()> {
    let (db, _, _) = open_db()?;
    run_impl(&db, ids)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn run_impl(db: &Database, ids: &[String]) -> Result<()> {
    // Resolve all IDs first (fail fast if any is invalid)
    let resolved_ids: Vec<String> = ids
        .iter()
        .map(|id| db.resolve_id(id))
        .collect::<Result<Vec<_>>>()?;

    for (i, resolved_id) in resolved_ids.iter().enumerate() {
        if i > 0 {
            println!("---");
        }
        output_single_tree(db, resolved_id)?;
    }

    Ok(())
}

fn output_single_tree(db: &Database, resolved_id: &str) -> Result<()> {
    let issue = db.get_issue(resolved_id)?;

    // Get blockers for root issue
    let blockers = db.get_transitive_blockers(resolved_id)?;
    let blocked_by = if blockers.is_empty() {
        None
    } else {
        Some(blockers.as_slice())
    };

    // Print root issue
    println!("{}", format_tree_root(&issue, blocked_by));

    // Get tracked and blocking issues
    let tracked = db.get_tracked(resolved_id)?;
    let blocking = db.get_blocking(resolved_id)?;

    // Determine if we need relation labels (only if both types exist)
    let show_labels = !tracked.is_empty() && !blocking.is_empty();

    // Print tracked children first
    let tracked_is_last_group = blocking.is_empty();
    print_children(
        db,
        &tracked,
        "",
        RelationType::Tracks,
        show_labels,
        tracked_is_last_group,
    )?;

    // Print blocking children (issues this one blocks)
    print_children(db, &blocking, "", RelationType::Blocks, show_labels, true)?;

    Ok(())
}

fn print_children(
    db: &crate::db::Database,
    children: &[String],
    prefix: &str,
    relation: RelationType,
    show_labels: bool,
    is_last_group: bool,
) -> Result<()> {
    for (i, child_id) in children.iter().enumerate() {
        let is_last_in_group = i == children.len() - 1;
        let is_last = is_last_in_group && is_last_group;
        let issue = db.get_issue(child_id)?;

        // Get transitive blockers for this issue (already filtered for open status)
        let blockers = db.get_transitive_blockers(child_id)?;

        let blocked_by = if blockers.is_empty() {
            None
        } else {
            Some(blockers.as_slice())
        };

        let label = if show_labels { Some(relation) } else { None };
        for line in format_tree_child(&issue, prefix, is_last, blocked_by, label) {
            println!("{}", line);
        }

        // Recursively print grandchildren (only for tracked relations)
        if relation == RelationType::Tracks {
            let grandchildren = db.get_tracked(child_id)?;
            let grandblocking = db.get_blocking(child_id)?;
            if !grandchildren.is_empty() || !grandblocking.is_empty() {
                let child_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };
                let show_grandlabels = !grandchildren.is_empty() && !grandblocking.is_empty();
                let tracked_is_last = grandblocking.is_empty();
                print_children(
                    db,
                    &grandchildren,
                    &child_prefix,
                    RelationType::Tracks,
                    show_grandlabels,
                    tracked_is_last,
                )?;
                print_children(
                    db,
                    &grandblocking,
                    &child_prefix,
                    RelationType::Blocks,
                    show_grandlabels,
                    true,
                )?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tests;
