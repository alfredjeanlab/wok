// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Background connection management for the daemon.
//!
//! This module provides infrastructure for managing WebSocket connections
//! in a background task, allowing the main loop to remain responsive to IPC
//! during connection attempts.

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::sync::{Transport, WebSocketTransport};

/// Connection state values for atomic state field.
pub const STATE_DISCONNECTED: u8 = 0;
pub const STATE_CONNECTING: u8 = 1;
pub const STATE_CONNECTED: u8 = 2;

/// Connection state visible to both background task and main loop.
///
/// Uses atomic fields for lock-free reads from IPC handlers.
pub struct SharedConnectionState {
    /// Current state (atomic for lock-free reads).
    state: AtomicU8,
    /// Connection attempt count (for status reporting).
    attempt: AtomicU32,
}

impl SharedConnectionState {
    /// Create a new shared state initialized to disconnected.
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(STATE_DISCONNECTED),
            attempt: AtomicU32::new(0),
        }
    }

    /// Get the current state.
    pub fn get(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    /// Set the state.
    pub fn set(&self, state: u8) {
        self.state.store(state, Ordering::Release);
    }

    /// Get the current attempt count.
    pub fn attempt(&self) -> u32 {
        self.attempt.load(Ordering::Acquire)
    }

    /// Set the attempt count.
    pub fn set_attempt(&self, attempt: u32) {
        self.attempt.store(attempt, Ordering::Release);
    }

    /// Check if currently connected.
    pub fn is_connected(&self) -> bool {
        self.get() == STATE_CONNECTED
    }

    /// Check if currently connecting.
    pub fn is_connecting(&self) -> bool {
        self.get() == STATE_CONNECTING
    }

    /// Get a human-readable status string.
    ///
    /// Used for enhanced status reporting in IPC responses.
    // KEEP UNTIL: Enhanced IPC status reporting
    #[allow(dead_code)]
    pub fn status_string(&self) -> String {
        match self.get() {
            STATE_DISCONNECTED => "disconnected".to_string(),
            STATE_CONNECTING => {
                let attempt = self.attempt();
                if attempt > 0 {
                    format!("connecting (attempt {})", attempt)
                } else {
                    "connecting".to_string()
                }
            }
            STATE_CONNECTED => "connected".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

impl Default for SharedConnectionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Events sent from the connection task to the main loop.
pub enum ConnectionEvent {
    /// Successfully connected. Contains the connected transport.
    Connected(WebSocketTransport),
    /// Connection attempt failed.
    Failed {
        /// Number of attempts made.
        attempts: u32,
        /// Error message.
        error: String,
    },
}

impl std::fmt::Debug for ConnectionEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected(_) => f.debug_tuple("Connected").field(&"<transport>").finish(),
            Self::Failed { attempts, error } => f
                .debug_struct("Failed")
                .field("attempts", attempts)
                .field("error", error)
                .finish(),
        }
    }
}

/// Configuration for the connection manager.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// URL to connect to.
    pub url: String,
    /// Maximum reconnection attempts (0 = unlimited).
    pub max_retries: u32,
    /// Maximum delay between reconnection attempts (seconds).
    pub max_delay_secs: u64,
    /// Initial delay for exponential backoff (milliseconds).
    pub initial_delay_ms: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:7890".to_string(),
            max_retries: 10,
            max_delay_secs: 30,
            initial_delay_ms: 100,
        }
    }
}

/// Manages the background connection task.
///
/// The ConnectionManager spawns a background task that handles WebSocket
/// connection establishment with exponential backoff. The main loop receives
/// connection events through a channel and can remain responsive to IPC
/// during connection attempts.
pub struct ConnectionManager {
    /// Configuration for connections.
    config: ConnectionConfig,
    /// Shared state for status reporting.
    shared_state: Arc<SharedConnectionState>,
    /// Sender for connection events.
    event_tx: mpsc::Sender<ConnectionEvent>,
    /// Cancellation token for graceful shutdown.
    cancel_token: CancellationToken,
}

impl ConnectionManager {
    /// Create a new connection manager.
    ///
    /// Returns the manager and a receiver for connection events.
    pub fn new(
        config: ConnectionConfig,
        shared_state: Arc<SharedConnectionState>,
    ) -> (Self, mpsc::Receiver<ConnectionEvent>) {
        let (event_tx, event_rx) = mpsc::channel(16);
        let cancel_token = CancellationToken::new();

        let manager = Self {
            config,
            shared_state,
            event_tx,
            cancel_token,
        };

        (manager, event_rx)
    }

    /// Get a cancellation token for this manager.
    ///
    /// Allows external code to monitor or trigger cancellation.
    // KEEP UNTIL: Enhanced IPC status reporting
    #[allow(dead_code)]
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Request a connection attempt.
    ///
    /// This starts the connection process if not already connecting.
    /// The result will be sent through the event channel.
    pub fn spawn_connect_task(&self) {
        let config = self.config.clone();
        let shared_state = Arc::clone(&self.shared_state);
        let event_tx = self.event_tx.clone();
        let cancel_token = self.cancel_token.clone();

        tokio::spawn(async move {
            connect_with_retry(config, shared_state, event_tx, cancel_token).await;
        });
    }

    /// Cancel any pending connection attempts.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Schedule a connection attempt after a delay.
    ///
    /// This spawns a background task that waits for the delay, then starts
    /// the connection attempt. This allows the caller to remain responsive
    /// while waiting for the retry.
    pub fn spawn_delayed_connect(&self, delay: Duration) {
        let config = self.config.clone();
        let shared_state = Arc::clone(&self.shared_state);
        let event_tx = self.event_tx.clone();
        let cancel_token = self.cancel_token.clone();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            if !cancel_token.is_cancelled() {
                connect_with_retry(config, shared_state, event_tx, cancel_token).await;
            }
        });
    }
}

/// Background connection task with exponential backoff.
async fn connect_with_retry(
    config: ConnectionConfig,
    shared_state: Arc<SharedConnectionState>,
    event_tx: mpsc::Sender<ConnectionEvent>,
    cancel_token: CancellationToken,
) {
    let mut attempt = 0u32;
    let mut delay_ms = config.initial_delay_ms;

    loop {
        // Check for cancellation before each attempt
        if cancel_token.is_cancelled() {
            shared_state.set(STATE_DISCONNECTED);
            return;
        }

        attempt = attempt.saturating_add(1);
        shared_state.set(STATE_CONNECTING);
        shared_state.set_attempt(attempt);

        // Create a new transport for each attempt
        let mut transport = WebSocketTransport::new();

        // Try to connect with cancellation support
        let connect_result = tokio::select! {
            _ = cancel_token.cancelled() => {
                shared_state.set(STATE_DISCONNECTED);
                return;
            }
            result = transport.connect(&config.url) => result,
        };

        match connect_result {
            Ok(()) => {
                // Connection successful
                shared_state.set(STATE_CONNECTED);
                shared_state.set_attempt(0);

                // Send the connected transport to the main loop
                let _ = event_tx.send(ConnectionEvent::Connected(transport)).await;
                return;
            }
            Err(e) => {
                // Connection failed
                let error = e.to_string();

                // Check if we've exceeded max retries (0 = unlimited)
                if config.max_retries > 0 && attempt >= config.max_retries {
                    shared_state.set(STATE_DISCONNECTED);
                    let _ = event_tx
                        .send(ConnectionEvent::Failed {
                            attempts: attempt,
                            error,
                        })
                        .await;
                    return;
                }

                // Wait with exponential backoff, checking for cancellation
                let delay = Duration::from_millis(delay_ms);
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        shared_state.set(STATE_DISCONNECTED);
                        return;
                    }
                    _ = tokio::time::sleep(delay) => {}
                }

                // Increase delay for next attempt (exponential backoff with cap)
                delay_ms = std::cmp::min(delay_ms.saturating_mul(2), config.max_delay_secs * 1000);
            }
        }
    }
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
