// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Remote sync module for distributed issue tracking.
//!
//! Provides WebSocket client functionality for syncing with wk-remote server.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │   Client    │────►│  Transport  │────►│   Remote    │
//! │ (SyncClient)│◄────│   (trait)   │◄────│   Server    │
//! └─────────────┘     └─────────────┘     └─────────────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │   Queue     │  (offline operations)
//! │ (OffQueue)  │
//! └─────────────┘
//! ```
//!
//! # Features
//!
//! - WebSocket connection to wk-remote server
//! - Offline queue with persisted JSONL storage
//! - Automatic reconnect with exponential backoff
//! - Sync on connect (request ops since last HLC)
//! - Injectable transport trait for testing

mod client;
mod queue;
mod transport;

pub use client::{SyncClient, SyncConfig, SyncError};
pub use queue::OfflineQueue;
pub use transport::{Transport, WebSocketTransport};

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
mod client_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod queue_tests;

#[cfg(test)]
mod transport_tests;
