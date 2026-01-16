// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Sync client for communicating with wk-remote server.
//!
//! Provides a high-level interface for:
//! - Connecting to remote server
//! - Sending operations (with offline queue fallback)
//! - Receiving broadcasts
//! - Automatic reconnection with exponential backoff

use std::path::Path;
use std::time::Duration;

use wk_core::protocol::{ClientMessage, ServerMessage};
use wk_core::{Hlc, Op};

use super::queue::{OfflineQueue, QueueError};
use super::transport::{Transport, TransportError, WebSocketTransport};

/// Configuration for the sync client.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// URL of the remote server.
    pub url: String,
    /// Maximum reconnection attempts.
    pub max_retries: u32,
    /// Maximum delay between reconnection attempts (seconds).
    pub max_delay_secs: u64,
    /// Initial delay for exponential backoff (milliseconds).
    pub initial_delay_ms: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        SyncConfig {
            url: "ws://localhost:7890".to_string(),
            max_retries: 10,
            max_delay_secs: 30,
            initial_delay_ms: 100,
        }
    }
}

/// Error type for sync client operations.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// Transport error.
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),

    /// Queue error.
    #[error("queue error: {0}")]
    Queue(#[from] QueueError),

    /// Not connected.
    #[error("not connected to remote server")]
    NotConnected,

    /// Max retries exceeded.
    #[error("max reconnection retries exceeded")]
    MaxRetriesExceeded,
}

/// Result type for sync client operations.
pub type SyncResult<T> = Result<T, SyncError>;

/// State of the sync client connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected.
    Disconnected,
    /// Attempting to connect.
    Connecting,
    /// Connected to remote server.
    Connected,
    /// Reconnecting after disconnect.
    Reconnecting { attempt: u32 },
}

/// Sync client for remote operations.
pub struct SyncClient<T: Transport = WebSocketTransport> {
    /// Configuration.
    config: SyncConfig,
    /// Transport layer.
    transport: T,
    /// Offline queue.
    queue: OfflineQueue,
    /// Connection state.
    state: ConnectionState,
    /// Last known HLC (for sync queries).
    last_hlc: Option<Hlc>,
}

impl SyncClient<WebSocketTransport> {
    /// Create a new sync client with default WebSocket transport.
    pub fn new(config: SyncConfig, queue_path: &Path) -> SyncResult<Self> {
        let transport = WebSocketTransport::new();
        let queue = OfflineQueue::open(queue_path)?;

        Ok(SyncClient {
            config,
            transport,
            queue,
            state: ConnectionState::Disconnected,
            last_hlc: None,
        })
    }
}

impl<T: Transport> SyncClient<T> {
    /// Create a new sync client with custom transport (for testing).
    pub fn with_transport(config: SyncConfig, transport: T, queue_path: &Path) -> SyncResult<Self> {
        let queue = OfflineQueue::open(queue_path)?;

        Ok(SyncClient {
            config,
            transport,
            queue,
            state: ConnectionState::Disconnected,
            last_hlc: None,
        })
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected && self.transport.is_connected()
    }

    /// Get the number of pending operations in the offline queue.
    pub fn pending_ops_count(&self) -> SyncResult<usize> {
        Ok(self.queue.len()?)
    }

    /// Get the last known HLC.
    pub fn last_hlc(&self) -> Option<Hlc> {
        self.last_hlc
    }

    /// Update last HLC if the given HLC is greater.
    fn update_last_hlc(&mut self, hlc: Hlc) {
        if let Some(ref mut last) = self.last_hlc {
            if hlc > *last {
                *last = hlc;
            }
        } else {
            self.last_hlc = Some(hlc);
        }
    }

    /// Connect to the remote server.
    pub async fn connect(&mut self) -> SyncResult<()> {
        self.state = ConnectionState::Connecting;

        match self.transport.connect(&self.config.url).await {
            Ok(()) => {
                self.state = ConnectionState::Connected;
                Ok(())
            }
            Err(e) => {
                self.state = ConnectionState::Disconnected;
                Err(e.into())
            }
        }
    }

    /// Disconnect from the remote server.
    pub async fn disconnect(&mut self) -> SyncResult<()> {
        self.transport.disconnect().await?;
        self.state = ConnectionState::Disconnected;
        Ok(())
    }

    /// Connect with exponential backoff retry.
    pub async fn connect_with_retry(&mut self) -> SyncResult<()> {
        let mut attempt = 0;
        let mut delay_ms = self.config.initial_delay_ms;

        loop {
            attempt += 1;
            self.state = ConnectionState::Reconnecting { attempt };

            match self.transport.connect(&self.config.url).await {
                Ok(()) => {
                    self.state = ConnectionState::Connected;
                    return Ok(());
                }
                Err(_) if attempt >= self.config.max_retries => {
                    self.state = ConnectionState::Disconnected;
                    return Err(SyncError::MaxRetriesExceeded);
                }
                Err(_) => {
                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = std::cmp::min(delay_ms * 2, self.config.max_delay_secs * 1000);
                }
            }
        }
    }

    /// Send an operation to the remote server.
    ///
    /// If not connected, the operation is queued for later sending.
    pub async fn send_op(&mut self, op: Op) -> SyncResult<()> {
        self.update_last_hlc(op.id);

        if self.is_connected() {
            let msg = ClientMessage::op(op.clone());
            match self.transport.send(msg).await {
                Ok(()) => Ok(()),
                Err(_) => {
                    // Connection lost, queue the op
                    self.state = ConnectionState::Disconnected;
                    self.queue.enqueue(&op)?;
                    Ok(())
                }
            }
        } else {
            // Not connected, queue the op
            self.queue.enqueue(&op)?;
            Ok(())
        }
    }

    /// Receive a message from the remote server.
    ///
    /// Returns `None` if the connection is closed.
    pub async fn recv(&mut self) -> SyncResult<Option<ServerMessage>> {
        if !self.is_connected() {
            return Err(SyncError::NotConnected);
        }

        match self.transport.recv().await {
            Ok(Some(msg)) => {
                // Update last HLC from received ops
                if let ServerMessage::Op(ref op) = msg {
                    self.update_last_hlc(op.id);
                }
                Ok(Some(msg))
            }
            Ok(None) => {
                self.state = ConnectionState::Disconnected;
                Ok(None)
            }
            Err(e) => {
                self.state = ConnectionState::Disconnected;
                Err(e.into())
            }
        }
    }

    /// Request sync from the server.
    ///
    /// Sends a Sync message requesting all operations since the given HLC.
    pub async fn request_sync(&mut self, since: Hlc) -> SyncResult<()> {
        if !self.is_connected() {
            return Err(SyncError::NotConnected);
        }

        let msg = ClientMessage::sync(since);
        self.transport.send(msg).await?;
        Ok(())
    }

    /// Request a full snapshot from the server.
    pub async fn request_snapshot(&mut self) -> SyncResult<()> {
        if !self.is_connected() {
            return Err(SyncError::NotConnected);
        }

        let msg = ClientMessage::snapshot();
        self.transport.send(msg).await?;
        Ok(())
    }

    /// Send a ping to the server.
    pub async fn ping(&mut self, id: u64) -> SyncResult<()> {
        if !self.is_connected() {
            return Err(SyncError::NotConnected);
        }

        let msg = ClientMessage::ping(id);
        self.transport.send(msg).await?;
        Ok(())
    }

    /// Flush the offline queue to the server.
    ///
    /// Returns the number of operations successfully sent.
    pub async fn flush_queue(&mut self) -> SyncResult<usize> {
        if !self.is_connected() {
            return Err(SyncError::NotConnected);
        }

        let ops = self.queue.peek_all()?;
        let mut sent = 0;

        for op in ops {
            let msg = ClientMessage::op(op);
            match self.transport.send(msg).await {
                Ok(()) => {
                    sent += 1;
                }
                Err(e) => {
                    // Connection lost, remove already-sent ops
                    self.state = ConnectionState::Disconnected;
                    if sent > 0 {
                        self.queue.remove_first(sent)?;
                    }
                    return Err(e.into());
                }
            }
        }

        // All ops sent successfully, clear queue
        if sent > 0 {
            self.queue.clear()?;
        }

        Ok(sent)
    }

    /// Perform sync on connect.
    ///
    /// This should be called after connecting to sync local state with server.
    /// It:
    /// 1. Flushes any queued offline operations
    /// 2. Requests ops since last known HLC (or full snapshot if no HLC)
    pub async fn sync_on_connect(&mut self) -> SyncResult<()> {
        if !self.is_connected() {
            return Err(SyncError::NotConnected);
        }

        // Flush offline queue first
        let _ = self.flush_queue().await?;

        // Request sync or snapshot
        if let Some(since) = self.last_hlc {
            self.request_sync(since).await?;
        } else {
            self.request_snapshot().await?;
        }

        Ok(())
    }
}
