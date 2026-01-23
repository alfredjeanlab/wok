// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! JSONL (JSON Lines) file utilities.
//!
//! Provides durable append-only storage for JSON-serializable records.
//! Each record is stored as a single JSON line with fsync for durability.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::Result;

/// Appends a record to a JSONL file with fsync for durability.
pub fn append<T: Serialize>(path: &Path, record: &T) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    let json = serde_json::to_string(record)?;
    writeln!(file, "{json}")?;
    file.sync_all()?;

    Ok(())
}

/// Reads all records from a JSONL file.
///
/// Skips empty lines and returns an empty vec if the file doesn't exist.
pub fn read_all<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: T = serde_json::from_str(&line)?;
        records.push(record);
    }

    Ok(records)
}

/// Writes all records to a JSONL file, replacing existing content.
///
/// Used for rewriting files after partial consumption (e.g., queue drain).
pub fn write_all<T: Serialize>(path: &Path, records: &[T]) -> Result<()> {
    let mut file = File::create(path)?;

    for record in records {
        let json = serde_json::to_string(record)?;
        writeln!(file, "{json}")?;
    }
    file.sync_all()?;

    Ok(())
}

#[cfg(test)]
#[path = "jsonl_tests.rs"]
mod tests;
