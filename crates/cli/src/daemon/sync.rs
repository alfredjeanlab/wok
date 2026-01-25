// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Sync operations for the daemon.
//!
//! This module handles WebSocket and Git sync operations,
//! including reconnection handling and bidirectional sync.

use std::path::Path;
use std::sync::Arc;

use wk_core::{Database, Hlc, Merge, Op, OpPayload, Oplog};

use super::cache::{handle_server_message, snapshot_issue_to_ops, update_hlc_markers};
use super::connection::SharedConnectionState;
use crate::commands::HlcPersistence;
use crate::error::{Error, Result};
use crate::sync::{SyncClient, Transport};
use crate::wal::Wal;
use crate::worktree::{self, OplogWorktree};

/// Snapshot data: (issues, tags, since_hlc) received from server
type SnapshotData = (Vec<wk_core::Issue>, Vec<(String, String)>, Hlc);

/// Perform a sync operation based on the backend type (async version).
pub async fn perform_sync_async<T: Transport>(
    backend: &mut super::runner::SyncBackend<T>,
    connection_state: &Arc<SharedConnectionState>,
) -> Result<usize> {
    use super::runner::SyncBackend;

    match backend {
        SyncBackend::Git {
            worktree,
            wal,
            db_path,
        } => sync_git(worktree, wal, db_path),
        SyncBackend::WebSocket {
            client,
            db_path,
            oplog_path,
            ..
        } => {
            if let Some(c) = client {
                sync_websocket(c, db_path, oplog_path, connection_state).await
            } else {
                // Not connected yet - report connection status
                if connection_state.is_connecting() {
                    Err(Error::Sync(format!(
                        "connecting to server (attempt {})",
                        connection_state.attempt()
                    )))
                } else {
                    Err(Error::Sync("not connected to server".to_string()))
                }
            }
        }
    }
}

/// Perform sync on reconnect: flush queue and request catch-up.
pub async fn sync_on_reconnect<T: Transport>(
    client: &mut SyncClient<T>,
    db_path: &Path,
    oplog_path: &Path,
) -> Result<()> {
    // Add pending ops to oplog BEFORE sending (matching sync_websocket behavior)
    // This ensures we recognize them as duplicates when server broadcasts them back
    let daemon_dir = oplog_path
        .parent()
        .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;
    let queue_path = daemon_dir.join("sync_queue.jsonl");

    if let Ok(queue) = crate::sync::OfflineQueue::open(&queue_path) {
        if let Ok(pending_ops) = queue.peek_all() {
            if !pending_ops.is_empty() {
                if let Ok(mut oplog) = Oplog::open(oplog_path) {
                    for op in &pending_ops {
                        // Ignore duplicate status - we just want them in the oplog
                        let _ = oplog.append(op);
                    }
                }
            }
        }
    }

    // Flush any queued offline operations
    if let Ok(flushed) = client.flush_queue().await {
        if flushed > 0 {
            tracing::info!("Flushed {} queued operations", flushed);
        }
    }

    // Request sync to catch up on missed ops, using persisted SERVER HLC (not client.last_hlc).
    // client.last_hlc() gets contaminated by local ops, which would cause new clients
    // to request sync(since=local_hlc) instead of snapshot, missing earlier ops.
    let sync_since = HlcPersistence::server(daemon_dir).read();

    if let Some(since) = sync_since {
        let _ = client.request_sync(since).await;
    } else {
        // No server HLC means this is a new client - request snapshot
        let _ = client.request_snapshot().await;
    }

    // Receive and apply sync/snapshot response
    // Note: The server broadcasts our own ops back to us before responding with the
    // sync/snapshot response, so we need to keep receiving until we get the response.
    let mut got_response = false;
    while let Ok(Some(msg)) = client.recv().await {
        use wk_core::protocol::ServerMessage;

        // Apply the message to local state
        handle_server_message(&msg, db_path, oplog_path)?;

        // Check if this is the sync/snapshot response (signals end of sync)
        match &msg {
            ServerMessage::SyncResponse { .. } | ServerMessage::SnapshotResponse { .. } => {
                got_response = true;
                break;
            }
            _ => {
                // Keep receiving (Op broadcasts, etc.)
                continue;
            }
        }
    }

    // Clear the queue only after successfully receiving sync response
    if got_response {
        let _ = client.clear_queue();
    }

    Ok(())
}

/// Perform WebSocket sync: flush queue, request sync.
///
/// The client must already be connected. Connection is established asynchronously
/// by the ConnectionManager, not by this function.
pub async fn sync_websocket<T: Transport>(
    client: &mut SyncClient<T>,
    db_path: &Path,
    oplog_path: &Path,
    _connection_state: &Arc<SharedConnectionState>,
) -> Result<usize> {
    use wk_core::protocol::ServerMessage;

    // Verify we're connected (caller should ensure this)
    if !client.is_connected() {
        return Err(Error::Sync("not connected to server".to_string()));
    }

    // Derive daemon_dir from oplog path (both are in the same daemon directory)
    let daemon_dir = oplog_path
        .parent()
        .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

    // Use persisted SERVER HLC for sync baseline, not client.last_hlc().
    // client.last_hlc() gets contaminated by local ops (via send_op), which
    // would cause new clients to request sync(since=local_hlc) instead of
    // snapshot, missing earlier ops from other clients.
    let sync_since = HlcPersistence::server(daemon_dir).read();
    let queue_path = daemon_dir.join("sync_queue.jsonl");

    // Read operations from queue without dequeuing
    let queue = crate::sync::OfflineQueue::open(&queue_path)
        .map_err(|e| Error::Sync(format!("failed to open queue: {}", e)))?;
    let pending_ops = queue
        .peek_all()
        .map_err(|e| Error::Sync(format!("failed to read queue: {}", e)))?;

    // Add pending operations to client oplog before sending to server
    // This ensures that when the server broadcasts them back, we recognize them as duplicates
    if !pending_ops.is_empty() {
        let mut oplog = Oplog::open(oplog_path)
            .map_err(|e| Error::Sync(format!("failed to open oplog for pending ops: {}", e)))?;
        for op in &pending_ops {
            // Ignore duplicate status - we just want them in the oplog
            let _ = oplog
                .append(op)
                .map_err(|e| Error::Sync(format!("failed to append pending op to oplog: {}", e)))?;
        }
    }

    // Flush the offline queue to the server
    let flushed = client
        .flush_queue()
        .await
        .map_err(|e| Error::Sync(format!("failed to flush queue: {}", e)))?;

    // Request sync from server based on persisted server HLC (or snapshot if none)
    if let Some(hlc) = sync_since {
        client
            .request_sync(hlc)
            .await
            .map_err(|e| Error::Sync(format!("failed to request sync: {}", e)))?;
    } else {
        client
            .request_snapshot()
            .await
            .map_err(|e| Error::Sync(format!("failed to request snapshot: {}", e)))?;
    }

    // Receive and apply operations from server
    let mut ops_received = 0;
    let mut received_ops = Vec::new();
    let mut snapshot_data: Option<SnapshotData> = None;

    // Set a timeout for receiving sync response
    // Returns true if we successfully received a sync/snapshot response
    let timeout_result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            match client.recv().await {
                Ok(Some(ServerMessage::SyncResponse { ops })) => {
                    received_ops.extend(ops.clone());
                    ops_received += ops.len();
                    return true; // Successfully received sync response
                }
                Ok(Some(ServerMessage::SnapshotResponse {
                    issues,
                    tags,
                    since,
                })) => {
                    // Snapshot contains full issues, tags, and high-water HLC
                    ops_received += issues.len();
                    snapshot_data = Some((issues, tags, since));
                    return true; // Successfully received snapshot response
                }
                Ok(Some(ServerMessage::Op(op))) => {
                    received_ops.push(op);
                    ops_received += 1;
                }
                Ok(Some(_)) => {
                    // Ignore other messages (Pong, Error)
                    continue;
                }
                Ok(None) => {
                    // Connection closed
                    return false;
                }
                Err(_) => {
                    return false;
                }
            }
        }
    });

    // Check if we successfully received a sync response (not timed out, not connection error)
    let sync_response_received = matches!(timeout_result.await, Ok(true));

    // Clear the send queue only if we successfully received a sync response.
    // This ensures ops are not lost if the connection drops before the server
    // acknowledges receipt. Duplicate ops on resend are handled by the server.
    if sync_response_received {
        let _ = client.clear_queue();
    }

    // Apply received ops to cache
    if !received_ops.is_empty() {
        let daemon_dir = oplog_path
            .parent()
            .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

        let mut oplog = Oplog::open(oplog_path)
            .map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;
        let mut db = Database::open(db_path)
            .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

        received_ops.sort();
        for op in &received_ops {
            // Only apply if new (not a duplicate)
            if oplog
                .append(op)
                .map_err(|e| Error::Sync(format!("failed to append to oplog: {}", e)))?
            {
                db.apply(op)
                    .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
            }
        }

        // Persist the max HLC from received ops for future sync requests
        if let Some(max_hlc) = received_ops.iter().map(|op| op.id).max() {
            update_hlc_markers(daemon_dir, max_hlc);
            client.set_last_hlc(max_hlc);
        }
    }

    // Apply snapshot data if received
    // Convert snapshot to ops and apply through Merge trait for HLC resolution
    if let Some((issues, tags, since)) = snapshot_data {
        let daemon_dir = oplog_path
            .parent()
            .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

        let mut oplog = Oplog::open(oplog_path)
            .map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;
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
        // This ensures new operations will have timestamps higher than what the server has seen
        update_hlc_markers(daemon_dir, since);

        // Update client's last_hlc so subsequent sync requests use the correct baseline
        client.set_last_hlc(since);
    }

    Ok(flushed + ops_received)
}

/// Perform git sync: fetch, merge oplogs, flush WAL, commit, and push.
///
/// Sync algorithm:
/// 1. Fetch from remote
/// 2. Read local oplog
/// 3. Check if remote has changes (compare HEADs)
/// 4. If remote has changes:
///    a. Read remote oplog
///    b. Merge local and remote oplogs (HLC sort + dedup)
///    c. Write merged oplog
/// 5. Append pending ops from WAL to oplog
/// 6. Commit and push
/// 7. Rebuild SQLite cache if new ops pulled
/// 8. Return count of ops pushed
pub fn sync_git(worktree: &OplogWorktree, wal: &Wal, db_path: &Path) -> Result<usize> {
    use std::collections::HashSet;
    use std::process::Command;

    use super::cache::rebuild_cache;

    let worktree_path = &worktree.path;

    // 1. Fetch from remote (ignore errors - remote may not exist yet)
    let _ = Command::new("git")
        .current_dir(worktree_path)
        .args(["fetch", "origin", &worktree.branch])
        .output();

    // 2. Read local oplog
    let mut local_ops = worktree::read_oplog(&worktree.oplog_path)?;

    // 3. Check if remote has changes
    let local_head = git_rev_parse(worktree_path, "HEAD").ok();
    let remote_head = git_rev_parse(worktree_path, &format!("origin/{}", worktree.branch)).ok();

    let (pulled_ops, pulled_count) = if local_head != remote_head && remote_head.is_some() {
        // 4. Remote has changes - merge oplogs
        // For now, use git merge to bring in remote changes, then re-read
        let _ = Command::new("git")
            .current_dir(worktree_path)
            .args([
                "merge",
                "--strategy-option=theirs",
                &format!("origin/{}", worktree.branch),
            ])
            .output();

        // Re-read the merged oplog
        let merged_ops = worktree::read_oplog(&worktree.oplog_path)?;

        // Find new ops that we pulled
        let local_ids: HashSet<_> = local_ops.iter().map(|op| &op.id).collect();
        let new_ops: Vec<Op> = merged_ops
            .iter()
            .filter(|op| !local_ids.contains(&op.id))
            .cloned()
            .collect();
        let new_count = new_ops.len();

        local_ops = merged_ops;
        (new_ops, new_count)
    } else {
        (Vec::new(), 0)
    };

    // 5. Take pending ops from WAL and append to oplog
    let pending_ops = wal.take_all()?;
    let pushed_count = pending_ops.len();

    if !pending_ops.is_empty() {
        // Append pending ops to oplog (deduplicating)
        let existing_ids: HashSet<_> = local_ops.iter().map(|op| &op.id).collect();
        let new_ops: Vec<_> = pending_ops
            .into_iter()
            .filter(|op| !existing_ids.contains(&op.id))
            .collect();

        if !new_ops.is_empty() {
            worktree::append_oplog(&worktree.oplog_path, &new_ops)?;

            // 6. Git add, commit, push
            let _ = Command::new("git")
                .current_dir(worktree_path)
                .args(["add", worktree::OPLOG_FILE])
                .output();

            let _ = Command::new("git")
                .current_dir(worktree_path)
                .args(["commit", "-m", "wok sync"])
                .output();

            let _ = Command::new("git")
                .current_dir(worktree_path)
                .args(["push", "origin", &worktree.branch])
                .output();
        }
    }

    // 7. Rebuild SQLite cache if new ops were pulled
    if pulled_count > 0 {
        rebuild_cache(db_path, &pulled_ops)?;
    }

    Ok(pushed_count)
}

/// Run git rev-parse to get a commit hash.
pub fn git_rev_parse(worktree_path: &std::path::Path, refspec: &str) -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .current_dir(worktree_path)
        .args(["rev-parse", refspec])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(Error::Config(format!("git rev-parse {} failed", refspec)))
    }
}

#[cfg(test)]
#[path = "sync_tests.rs"]
mod tests;
