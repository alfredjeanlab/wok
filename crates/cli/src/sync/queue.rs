// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Offline queue for persisting operations when disconnected.
//!
//! Uses JSONL format for durability - each operation is written as a single line
//! and fsynced immediately. On reconnect, queued operations are flushed to the
//! server in order.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use wk_core::Op;

/// Error type for queue operations.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for queue operations.
pub type QueueResult<T> = Result<T, QueueError>;

/// Offline queue for persisting operations.
///
/// Operations are stored in a JSONL file, one operation per line.
/// Each write is fsynced to ensure durability.
pub struct OfflineQueue {
    /// Path to the queue file.
    path: PathBuf,
}

impl OfflineQueue {
    /// Create or open an offline queue at the given path.
    pub fn open(path: &Path) -> QueueResult<Self> {
        // Ensure the file exists (create if not)
        OpenOptions::new().create(true).append(true).open(path)?;

        Ok(OfflineQueue {
            path: path.to_path_buf(),
        })
    }

    /// Enqueue an operation for later sending.
    ///
    /// The operation is immediately persisted to disk.
    pub fn enqueue(&mut self, op: &Op) -> QueueResult<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(op)?;
        writeln!(file, "{}", json)?;
        file.sync_all()?;

        Ok(())
    }

    /// Read all queued operations without removing them.
    pub fn peek_all(&self) -> QueueResult<Vec<Op>> {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(e) => return Err(e.into()),
        };

        let reader = BufReader::new(file);
        let mut ops = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let op: Op = serde_json::from_str(&line)?;
            ops.push(op);
        }

        Ok(ops)
    }

    /// Clear all queued operations.
    ///
    /// Call this after successfully flushing the queue to the server.
    pub fn clear(&mut self) -> QueueResult<()> {
        // Truncate the file
        File::create(&self.path)?;
        Ok(())
    }

    /// Get the number of queued operations.
    pub fn len(&self) -> QueueResult<usize> {
        Ok(self.peek_all()?.len())
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> QueueResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Remove the first N operations from the queue.
    ///
    /// This is used when flushing operations incrementally.
    pub fn remove_first(&mut self, count: usize) -> QueueResult<()> {
        let ops = self.peek_all()?;
        if count >= ops.len() {
            return self.clear();
        }

        // Rewrite the file without the first N ops
        let remaining = &ops[count..];
        let mut file = File::create(&self.path)?;
        for op in remaining {
            let json = serde_json::to_string(op)?;
            writeln!(file, "{}", json)?;
        }
        file.sync_all()?;

        Ok(())
    }
}
