// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Generic HLC persistence abstraction for different HLC kinds.
//!
//! This module provides a unified interface for persisting HLC (Hybrid Logical Clock)
//! values to disk. It eliminates duplicate code for handling `last_hlc` (locally-generated
//! high-water mark) and `server_hlc` (server-confirmed high-water mark).

use std::path::{Path, PathBuf};

use wk_core::Hlc;

use crate::error::Result;

/// Generic HLC persistence abstraction for different HLC kinds
/// (e.g., local "last_hlc", server-side "server_hlc").
pub struct HlcPersistence {
    path: PathBuf,
}

impl HlcPersistence {
    /// Filename for locally-generated HLC high-water mark.
    pub const LAST_HLC: &'static str = "last_hlc.txt";

    /// Filename for server-confirmed HLC high-water mark.
    pub const SERVER_HLC: &'static str = "server_hlc.txt";

    /// Create persistence handler for a specific HLC file.
    pub fn new(daemon_dir: &Path, filename: &str) -> Self {
        HlcPersistence {
            path: daemon_dir.join(filename),
        }
    }

    /// Create persistence for local HLC (last_hlc.txt).
    pub fn last(daemon_dir: &Path) -> Self {
        Self::new(daemon_dir, Self::LAST_HLC)
    }

    /// Create persistence for server HLC (server_hlc.txt).
    pub fn server(daemon_dir: &Path) -> Self {
        Self::new(daemon_dir, Self::SERVER_HLC)
    }

    /// Read HLC from disk, returns None if file doesn't exist or is invalid.
    pub fn read(&self) -> Option<Hlc> {
        let content = std::fs::read_to_string(&self.path).ok()?;
        content.trim().parse().ok()
    }

    /// Write HLC to disk with fsync for durability.
    pub fn write(&self, hlc: Hlc) -> Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(&self.path)?;
        write!(file, "{}", hlc)?;
        file.sync_all()?;
        Ok(())
    }

    /// Update HLC only if the given value is greater (high-water mark pattern).
    pub fn update(&self, hlc: Hlc) -> Result<()> {
        if let Some(current) = self.read() {
            if hlc > current {
                self.write(hlc)?;
            }
        } else {
            self.write(hlc)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "hlc_persistence_tests.rs"]
mod tests;
