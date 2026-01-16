// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! IPC protocol for CLI-daemon communication.
//!
//! The daemon listens on a Unix socket and accepts commands from CLI processes.
//! Messages are serialized as JSON with length-prefixed framing.

use serde::{Deserialize, Serialize};

/// Request sent from CLI to daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    /// Get daemon status.
    Status,
    /// Force immediate sync with remote.
    SyncNow,
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
    /// Sync completed.
    SyncComplete { ops_synced: usize },
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
    /// Whether connected to remote server.
    pub connected: bool,
    /// Remote server URL.
    pub remote_url: String,
    /// Number of pending operations in queue.
    pub pending_ops: usize,
    /// Unix timestamp of last successful sync (if any).
    pub last_sync: Option<u64>,
    /// Current daemon PID.
    pub pid: u32,
    /// Uptime in seconds.
    pub uptime_secs: u64,
}

impl DaemonStatus {
    /// Create a new status with the given parameters.
    pub fn new(
        connected: bool,
        remote_url: String,
        pending_ops: usize,
        last_sync: Option<u64>,
        pid: u32,
        uptime_secs: u64,
    ) -> Self {
        Self {
            connected,
            remote_url,
            pending_ops,
            last_sync,
            pid,
            uptime_secs,
        }
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
    use crate::error::{Error, Result};

    /// Maximum message size (1MB) to prevent malformed responses from causing hangs.
    const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

    /// Write a request to the given writer.
    pub fn write_request<W: Write>(writer: &mut W, request: &DaemonRequest) -> Result<()> {
        let json = serde_json::to_vec(request)
            .map_err(|e| Error::Io(std::io::Error::other(format!("serialize error: {}", e))))?;
        let len = u32::try_from(json.len())
            .map_err(|_| Error::Io(std::io::Error::other("message too large".to_string())))?;
        writer.write_all(&len.to_be_bytes())?;
        writer.write_all(&json)?;
        writer.flush()?;
        Ok(())
    }

    /// Read a request from the given reader (server-side, used by tests).
    #[cfg(test)]
    pub fn read_request<R: Read>(reader: &mut R) -> Result<DaemonRequest> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(Error::Io(std::io::Error::other(format!(
                "message too large: {} bytes (max {})",
                len, MAX_MESSAGE_SIZE
            ))));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        serde_json::from_slice(&buf)
            .map_err(|e| Error::Io(std::io::Error::other(format!("deserialize error: {}", e))))
    }

    /// Write a response to the given writer (server-side, used by tests).
    #[cfg(test)]
    pub fn write_response<W: Write>(writer: &mut W, response: &DaemonResponse) -> Result<()> {
        let json = serde_json::to_vec(response)
            .map_err(|e| Error::Io(std::io::Error::other(format!("serialize error: {}", e))))?;
        let len = u32::try_from(json.len())
            .map_err(|_| Error::Io(std::io::Error::other("message too large".to_string())))?;
        writer.write_all(&len.to_be_bytes())?;
        writer.write_all(&json)?;
        writer.flush()?;
        Ok(())
    }

    /// Read a response from the given reader.
    pub fn read_response<R: Read>(reader: &mut R) -> Result<DaemonResponse> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(Error::Io(std::io::Error::other(format!(
                "message too large: {} bytes (max {})",
                len, MAX_MESSAGE_SIZE
            ))));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        serde_json::from_slice(&buf)
            .map_err(|e| Error::Io(std::io::Error::other(format!("deserialize error: {}", e))))
    }
}

/// Async IPC message framing using tokio.
pub mod framing_async {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::error::{Error, Result};

    /// Maximum message size (1MB) to prevent malformed responses from causing hangs.
    const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

    /// Read a request from the given async reader.
    pub async fn read_request<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<DaemonRequest> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(Error::Io(std::io::Error::other(format!(
                "message too large: {} bytes (max {})",
                len, MAX_MESSAGE_SIZE
            ))));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;

        serde_json::from_slice(&buf)
            .map_err(|e| Error::Io(std::io::Error::other(format!("deserialize error: {}", e))))
    }

    /// Write a response to the given async writer.
    pub async fn write_response<W: AsyncWriteExt + Unpin>(
        writer: &mut W,
        response: &DaemonResponse,
    ) -> Result<()> {
        let json = serde_json::to_vec(response)
            .map_err(|e| Error::Io(std::io::Error::other(format!("serialize error: {}", e))))?;
        let len = u32::try_from(json.len())
            .map_err(|_| Error::Io(std::io::Error::other("message too large".to_string())))?;
        writer.write_all(&len.to_be_bytes()).await?;
        writer.write_all(&json).await?;
        writer.flush().await?;
        Ok(())
    }
}
