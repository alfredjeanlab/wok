// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! IPC protocol for daemon-CLI communication.
//!
//! This module mirrors the protocol defined in the CLI crate's daemon/ipc.rs.
//! Messages are serialized as JSON with length-prefixed framing.

use serde::{Deserialize, Serialize};

/// Request sent from CLI to daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    /// Get daemon status.
    Status,
    /// Graceful shutdown.
    Shutdown,
    /// Ping to check if daemon is alive.
    Ping,
    /// Version handshake request.
    Hello { version: String },
}

/// Response sent from daemon to CLI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DaemonResponse {
    /// Status response.
    Status(DaemonStatus),
    /// Shutdown acknowledged.
    ShuttingDown,
    /// Pong response.
    Pong,
    /// Error response.
    Error { message: String },
    /// Version handshake response.
    Hello { version: String },
}

/// Daemon status information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonStatus {
    /// Current daemon PID.
    pub pid: u32,
    /// Uptime in seconds.
    pub uptime_secs: u64,
}

impl DaemonStatus {
    /// Create a new status with the given parameters.
    pub fn new(pid: u32, uptime_secs: u64) -> Self {
        Self { pid, uptime_secs }
    }
}

/// IPC message framing.
///
/// Messages are framed as:
/// - 4 bytes: message length (big-endian u32)
/// - N bytes: JSON-encoded message
pub mod framing {
    use std::io::{Read, Write};

    use super::*;

    /// Maximum message size (1MB) to prevent malformed responses from causing hangs.
    const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

    /// Read a request from the given reader.
    pub fn read_request<R: Read>(reader: &mut R) -> std::io::Result<DaemonRequest> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(std::io::Error::other(format!(
                "message too large: {} bytes (max {})",
                len, MAX_MESSAGE_SIZE
            )));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::other(format!("deserialize error: {}", e)))
    }

    /// Write a response to the given writer.
    pub fn write_response<W: Write>(
        writer: &mut W,
        response: &DaemonResponse,
    ) -> std::io::Result<()> {
        let json = serde_json::to_vec(response)
            .map_err(|e| std::io::Error::other(format!("serialize error: {}", e)))?;
        let len =
            u32::try_from(json.len()).map_err(|_| std::io::Error::other("message too large"))?;
        writer.write_all(&len.to_be_bytes())?;
        writer.write_all(&json)?;
        writer.flush()?;
        Ok(())
    }
}
