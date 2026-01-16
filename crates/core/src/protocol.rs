// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! WebSocket protocol messages for client-server communication.
//!
//! The protocol is simple:
//! - Client sends operations and sync requests
//! - Server broadcasts operations and responds to sync/snapshot requests

use serde::{Deserialize, Serialize};

use crate::hlc::Hlc;
use crate::issue::Issue;
use crate::op::Op;

/// Messages sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send an operation to the server.
    ///
    /// The server will apply it and broadcast to all clients.
    Op(Op),

    /// Request operations since a given HLC.
    ///
    /// Used for incremental sync on reconnect.
    Sync {
        /// Return all ops with ID > since.
        since: Hlc,
    },

    /// Request a full database snapshot.
    ///
    /// Used by new or recovering clients.
    Snapshot,

    /// Ping message for keepalive.
    Ping {
        /// Client-chosen ID echoed in Pong.
        id: u64,
    },
}

/// Messages sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// An operation broadcast to all connected clients.
    ///
    /// Sent when any client submits an operation.
    Op(Op),

    /// Response to a Sync request.
    ///
    /// Contains all operations since the requested HLC.
    SyncResponse {
        /// Operations since the requested HLC, sorted by ID.
        ops: Vec<Op>,
    },

    /// Response to a Snapshot request.
    ///
    /// Contains the current state of all issues.
    SnapshotResponse {
        /// All issues in the database.
        issues: Vec<Issue>,
        /// All tags as (issue_id, tag) pairs.
        tags: Vec<(String, String)>,
        /// The current HLC high-water mark.
        ///
        /// Client should subscribe to ops after this point.
        since: Hlc,
    },

    /// Pong response to client Ping.
    Pong {
        /// Echoed from the Ping message.
        id: u64,
    },

    /// Error message.
    Error {
        /// Human-readable error description.
        message: String,
    },
}

impl ClientMessage {
    /// Creates an Op message.
    pub fn op(op: Op) -> Self {
        ClientMessage::Op(op)
    }

    /// Creates a Sync message.
    pub fn sync(since: Hlc) -> Self {
        ClientMessage::Sync { since }
    }

    /// Creates a Snapshot message.
    pub fn snapshot() -> Self {
        ClientMessage::Snapshot
    }

    /// Creates a Ping message.
    pub fn ping(id: u64) -> Self {
        ClientMessage::Ping { id }
    }

    /// Serializes the message to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes the message from JSON.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl ServerMessage {
    /// Creates an Op message.
    pub fn op(op: Op) -> Self {
        ServerMessage::Op(op)
    }

    /// Creates a SyncResponse message.
    pub fn sync_response(ops: Vec<Op>) -> Self {
        ServerMessage::SyncResponse { ops }
    }

    /// Creates a SnapshotResponse message.
    pub fn snapshot_response(issues: Vec<Issue>, tags: Vec<(String, String)>, since: Hlc) -> Self {
        ServerMessage::SnapshotResponse {
            issues,
            tags,
            since,
        }
    }

    /// Creates a Pong message.
    pub fn pong(id: u64) -> Self {
        ServerMessage::Pong { id }
    }

    /// Creates an Error message.
    pub fn error(message: impl Into<String>) -> Self {
        ServerMessage::Error {
            message: message.into(),
        }
    }

    /// Serializes the message to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes the message from JSON.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
