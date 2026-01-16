// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Append-only operation log for sync protocol.
//!
//! The oplog stores all operations as JSONL (one JSON object per line) for:
//! - Incremental sync between clients
//! - Audit trail
//! - Recovery/rebuild if needed
//!
//! Each operation is appended with fsync for durability.

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::hlc::Hlc;
use crate::op::Op;

/// Append-only operation log stored as JSONL.
pub struct Oplog {
    path: PathBuf,
    /// Set of operation IDs we've seen (for deduplication).
    seen_ids: HashSet<Hlc>,
}

impl Oplog {
    /// Opens or creates an oplog at the given path.
    ///
    /// Loads existing operation IDs into memory for deduplication.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut seen_ids = HashSet::new();

        if path.exists() {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let op: Op = serde_json::from_str(&line)?;
                seen_ids.insert(op.id);
            }
        }

        Ok(Oplog { path, seen_ids })
    }

    /// Creates a new in-memory oplog (for testing).
    #[cfg(test)]
    pub fn in_memory() -> Self {
        Oplog {
            path: PathBuf::new(),
            seen_ids: HashSet::new(),
        }
    }

    /// Appends an operation to the log.
    ///
    /// Returns Ok(true) if the operation was appended, Ok(false) if it was
    /// a duplicate (already in the log).
    pub fn append(&mut self, op: &Op) -> Result<bool> {
        if self.seen_ids.contains(&op.id) {
            return Ok(false);
        }

        // Append to file if we have a real path
        if !self.path.as_os_str().is_empty() {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;

            let json = serde_json::to_string(op)?;
            writeln!(file, "{json}")?;
            file.sync_all()?;
        }

        self.seen_ids.insert(op.id);
        Ok(true)
    }

    /// Returns all operations with ID greater than the given HLC.
    ///
    /// Used for incremental sync: "give me all ops since X".
    pub fn ops_since(&self, since: Hlc) -> Result<Vec<Op>> {
        if self.path.as_os_str().is_empty() {
            return Ok(Vec::new());
        }

        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        let mut ops = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let op: Op = serde_json::from_str(&line)?;
            if op.id > since {
                ops.push(op);
            }
        }

        ops.sort();
        Ok(ops)
    }

    /// Returns all operations in the log.
    pub fn all_ops(&self) -> Result<Vec<Op>> {
        self.ops_since(Hlc::min())
    }

    /// Returns the number of operations in the log.
    pub fn len(&self) -> usize {
        self.seen_ids.len()
    }

    /// Returns true if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.seen_ids.is_empty()
    }

    /// Returns true if the operation with this ID has been seen.
    pub fn contains(&self, id: &Hlc) -> bool {
        self.seen_ids.contains(id)
    }

    /// Returns the path to the oplog file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
#[path = "oplog_tests.rs"]
mod tests;
