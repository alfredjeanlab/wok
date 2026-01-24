// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Cache operations for the daemon.
//!
//! This module handles applying operations to the local SQLite cache database
//! and managing the client-side oplog for deduplication.

use std::path::Path;

use wk_core::{Database, Hlc, Merge, Op, OpPayload, Oplog};

use crate::commands::HlcPersistence;
use crate::error::{Error, Result};

/// Update both server and last HLC markers atomically.
///
/// This is a helper to reduce duplication when updating HLC state
/// after receiving operations from the server.
pub fn update_hlc_markers(daemon_dir: &Path, hlc: Hlc) {
    let _ = HlcPersistence::server(daemon_dir).update(hlc);
    let _ = HlcPersistence::last(daemon_dir).update(hlc);
}

/// Handle a message received from the server.
pub fn handle_server_message(
    msg: &wk_core::protocol::ServerMessage,
    db_path: &Path,
    oplog_path: &Path,
) -> Result<()> {
    use wk_core::protocol::ServerMessage;

    let daemon_dir = oplog_path
        .parent()
        .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

    match msg {
        ServerMessage::Op(op) => {
            apply_op_to_cache(op, db_path, oplog_path)?;
            update_hlc_markers(daemon_dir, op.id);
        }
        ServerMessage::SyncResponse { ops } => {
            for op in ops {
                apply_op_to_cache(op, db_path, oplog_path)?;
            }
            // Persist max SERVER HLC from sync response
            if let Some(max_hlc) = ops.iter().map(|op| op.id).max() {
                update_hlc_markers(daemon_dir, max_hlc);
            }
        }
        ServerMessage::SnapshotResponse {
            issues,
            tags,
            since,
        } => {
            // Apply snapshot: convert issues to ops and apply through Merge trait
            apply_snapshot_to_cache(issues, tags, *since, db_path, oplog_path, daemon_dir)?;
        }
        // Ignore other message types (Pong, Error)
        _ => {}
    }

    Ok(())
}

/// Apply a received operation to the local cache database.
pub fn apply_op_to_cache(op: &Op, db_path: &Path, oplog_path: &Path) -> Result<()> {
    // Load oplog and check for duplicates
    let mut oplog =
        Oplog::open(oplog_path).map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;

    let is_new = oplog
        .append(op)
        .map_err(|e| Error::Sync(format!("failed to append to oplog: {}", e)))?;

    if !is_new {
        // Already seen this operation - skip
        return Ok(());
    }

    // Apply to database
    let mut db =
        Database::open(db_path).map_err(|e| Error::Sync(format!("failed to open db: {}", e)))?;
    db.apply(op)
        .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
    Ok(())
}

/// Apply a snapshot response to the local cache database.
pub fn apply_snapshot_to_cache(
    issues: &[wk_core::Issue],
    tags: &[(String, String)],
    since: Hlc,
    db_path: &Path,
    oplog_path: &Path,
    daemon_dir: &Path,
) -> Result<()> {
    let mut oplog =
        Oplog::open(oplog_path).map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;
    let mut db = Database::open(db_path)
        .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

    // Convert snapshot to ops
    let mut all_ops: Vec<Op> = Vec::new();
    for (index, issue) in issues.iter().enumerate() {
        // CORRECTNESS: Index bounded by issue count which fits in u32
        #[allow(clippy::cast_possible_truncation)]
        all_ops.extend(snapshot_issue_to_ops(issue, index as u32));
    }

    // Add label ops with unique synthetic HLCs
    // Start counter after issue ops to avoid collisions
    // CORRECTNESS: Issue count fits in u32
    #[allow(clippy::cast_possible_truncation)]
    let label_offset = issues.len() as u32;
    for (index, (issue_id, label)) in tags.iter().enumerate() {
        // CORRECTNESS: Index bounded by label count which fits in u32
        #[allow(clippy::cast_possible_truncation)]
        all_ops.push(Op::new(
            Hlc::new(0, label_offset + index as u32, 0),
            OpPayload::AddLabel {
                issue_id: issue_id.clone(),
                label: label.clone(),
            },
        ));
    }

    // Sort and apply through Merge trait with deduplication
    all_ops.sort();
    for op in &all_ops {
        // Only apply if new (not a duplicate)
        if oplog
            .append(op)
            .map_err(|e| Error::Sync(format!("failed to append to oplog: {}", e)))?
        {
            db.apply(op)
                .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
        }
    }

    // Persist the snapshot's high-water HLC for future sync requests
    update_hlc_markers(daemon_dir, since);

    Ok(())
}

/// Convert a snapshot issue to synthetic operations for HLC-aware merge.
///
/// Uses a unique index to create distinct HLCs for CreateIssue ops since
/// the oplog deduplicates by HLC, not by payload.
pub fn snapshot_issue_to_ops(issue: &wk_core::Issue, index: u32) -> Vec<Op> {
    let mut ops = Vec::new();

    // CreateIssue with unique synthetic HLC (using index to differentiate)
    // The wall_ms=0 ensures these are considered "oldest" and real ops win
    ops.push(Op::new(
        Hlc::new(0, index, 0),
        OpPayload::CreateIssue {
            id: issue.id.clone(),
            issue_type: issue.issue_type,
            title: issue.title.clone(),
        },
    ));

    // SetTitle with issue's title HLC
    if let Some(hlc) = issue.last_title_hlc {
        ops.push(Op::new(
            hlc,
            OpPayload::SetTitle {
                issue_id: issue.id.clone(),
                title: issue.title.clone(),
            },
        ));
    }

    // SetStatus with issue's status HLC
    if let Some(hlc) = issue.last_status_hlc {
        ops.push(Op::new(
            hlc,
            OpPayload::SetStatus {
                issue_id: issue.id.clone(),
                status: issue.status,
                reason: None,
            },
        ));
    }

    // SetType with issue's type HLC
    if let Some(hlc) = issue.last_type_hlc {
        ops.push(Op::new(
            hlc,
            OpPayload::SetType {
                issue_id: issue.id.clone(),
                issue_type: issue.issue_type,
            },
        ));
    }

    ops
}

/// Rebuild the SQLite cache by applying new operations.
///
/// This applies pulled operations to the existing database, using the
/// Merge trait to handle conflicts with HLC-based resolution.
pub fn rebuild_cache(db_path: &Path, new_ops: &[Op]) -> Result<()> {
    if new_ops.is_empty() {
        return Ok(());
    }

    // Open database and apply new ops
    let mut db = Database::open(db_path)
        .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

    // Sort ops by HLC for proper ordering
    let mut sorted_ops = new_ops.to_vec();
    sorted_ops.sort();

    // Apply all new ops using the Merge trait
    db.apply_all(&sorted_ops)
        .map_err(|e| Error::Sync(format!("failed to apply ops: {}", e)))?;

    Ok(())
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
