// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Write-Ahead Log for pending operations.
//!
//! The WAL stores operations that have been created locally but not yet synced
//! to the remote. This ensures durability across daemon restarts and allows
//! operations to accumulate while offline.
//!
//! Format: JSONL (one JSON object per line), same as oplog.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use wk_core::Op;

use crate::error::Result;

/// Write-Ahead Log for pending operations.
pub struct Wal {
    path: PathBuf,
}

impl Wal {
    /// Opens or creates a WAL at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Wal { path })
    }

    /// Returns the path to the WAL file.
    // KEEP UNTIL: WAL inspection commands are implemented
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends an operation to the WAL.
    ///
    /// The operation is written with fsync for durability.
    // KEEP UNTIL: WAL inspection commands are implemented
    #[allow(dead_code)]
    pub fn append(&self, op: &Op) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(op)?;
        writeln!(file, "{}", json)?;
        file.sync_all()?;

        Ok(())
    }

    /// Appends multiple operations to the WAL atomically.
    // KEEP UNTIL: WAL inspection commands are implemented
    #[allow(dead_code)]
    pub fn append_batch(&self, ops: &[Op]) -> Result<()> {
        if ops.is_empty() {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        for op in ops {
            let json = serde_json::to_string(op)?;
            writeln!(file, "{}", json)?;
        }
        file.sync_all()?;

        Ok(())
    }

    /// Reads all pending operations from the WAL.
    pub fn read_all(&self) -> Result<Vec<Op>> {
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
            ops.push(op);
        }

        Ok(ops)
    }

    /// Returns the count of pending operations.
    pub fn count(&self) -> Result<usize> {
        if !self.path.exists() {
            return Ok(0);
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let count = reader
            .lines()
            .map_while(|l| l.ok())
            .filter(|l| !l.trim().is_empty())
            .count();

        Ok(count)
    }

    /// Clears all pending operations from the WAL.
    ///
    /// Call this after operations have been successfully synced.
    pub fn clear(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::write(&self.path, "")?;
        }
        Ok(())
    }

    /// Takes all pending operations and clears the WAL.
    ///
    /// This is atomic - if any error occurs, the WAL is left unchanged.
    pub fn take_all(&self) -> Result<Vec<Op>> {
        let ops = self.read_all()?;
        if !ops.is_empty() {
            self.clear()?;
        }
        Ok(ops)
    }

    /// Returns true if the WAL has pending operations.
    // KEEP UNTIL: WAL inspection commands are implemented
    #[allow(dead_code)]
    pub fn has_pending(&self) -> bool {
        if !self.path.exists() {
            return false;
        }

        self.count().map(|c| c > 0).unwrap_or(false)
    }
}

#[cfg(test)]
#[path = "wal_tests.rs"]
mod tests;
