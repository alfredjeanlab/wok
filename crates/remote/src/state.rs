// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Server state management.
//!
//! Wraps the canonical database and oplog for thread-safe access.

use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use wk_core::protocol::ServerMessage;
use wk_core::{Database, Hlc, HlcClock, Merge, Op, Oplog, Result, SystemClock};

use crate::git_backing::GitBacking;

/// Shared server state containing the canonical database and oplog.
#[derive(Clone)]
pub struct ServerState {
    inner: Arc<ServerStateInner>,
}

struct ServerStateInner {
    /// The canonical database (protected by mutex for writes).
    db: Mutex<Database>,
    /// The operation log (protected by mutex for writes).
    oplog: Mutex<Oplog>,
    /// HLC clock for generating timestamps.
    clock: HlcClock<SystemClock>,
    /// Broadcast channel for notifying clients of new operations.
    broadcast_tx: tokio::sync::broadcast::Sender<ServerMessage>,
    /// High-water mark HLC for sync queries.
    last_hlc: RwLock<Hlc>,
    /// Optional git backing for durability.
    git_backing: Option<Arc<GitBacking>>,
}

impl ServerState {
    /// Creates a new server state with database and oplog in the given directory.
    pub fn new(data_dir: &Path, git_backing: Option<Arc<GitBacking>>) -> Result<Self> {
        let db_path = data_dir.join("issues.db");
        let oplog_path = data_dir.join("oplog.jsonl");

        let db = Database::open(&db_path)?;
        let oplog = Oplog::open(&oplog_path)?;

        // Generate a node ID from the data directory path hash
        let node_id = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            data_dir.hash(&mut hasher);
            (hasher.finish() & 0xFFFF_FFFF) as u32
        };

        let clock = HlcClock::new(node_id);

        // Create broadcast channel with reasonable buffer
        let (broadcast_tx, _) = tokio::sync::broadcast::channel(1024);

        Ok(ServerState {
            inner: Arc::new(ServerStateInner {
                db: Mutex::new(db),
                oplog: Mutex::new(oplog),
                clock,
                broadcast_tx,
                last_hlc: RwLock::new(Hlc::min()),
                git_backing,
            }),
        })
    }

    /// Applies an operation to the database and oplog, then broadcasts it.
    ///
    /// Returns Ok(true) if the operation was applied, Ok(false) if it was a duplicate.
    pub async fn apply_op(&self, op: Op) -> Result<bool> {
        // Update the clock based on the received op
        let _ = self.inner.clock.receive(&op.id);

        // Append to oplog (dedup check)
        let appended = {
            let mut oplog = self.inner.oplog.lock().await;
            oplog.append(&op)?
        };

        if !appended {
            return Ok(false); // Duplicate
        }

        // Apply to database
        {
            let mut db = self.inner.db.lock().await;
            db.apply(&op)?;
        }

        // Update high-water mark
        {
            let mut last_hlc = self.inner.last_hlc.write().await;
            if op.id > *last_hlc {
                *last_hlc = op.id;
            }
        }

        // Broadcast to all clients
        let _ = self.inner.broadcast_tx.send(ServerMessage::op(op));

        // Mark git backing as dirty
        if let Some(ref git) = self.inner.git_backing {
            git.mark_dirty().await;
        }

        Ok(true)
    }

    /// Returns operations since the given HLC for sync.
    pub async fn ops_since(&self, since: Hlc) -> Result<Vec<Op>> {
        let oplog = self.inner.oplog.lock().await;
        oplog.ops_since(since)
    }

    /// Returns a snapshot of all issues and tags.
    pub async fn snapshot(&self) -> Result<(Vec<wk_core::Issue>, Vec<(String, String)>, Hlc)> {
        let db = self.inner.db.lock().await;
        let issues = db.get_all_issues()?;
        let tags = db.get_all_labels()?;
        let since = *self.inner.last_hlc.read().await;
        Ok((issues, tags, since))
    }

    /// Subscribe to broadcast messages.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ServerMessage> {
        self.inner.broadcast_tx.subscribe()
    }

    /// Generate a new HLC timestamp.
    // KEEP UNTIL: Server-initiated timestamp generation
    #[allow(dead_code)]
    pub fn now(&self) -> Hlc {
        self.inner.clock.now()
    }
}
