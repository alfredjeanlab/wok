// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! IPC client for communicating with the wokd daemon.
//!
//! Provides a connection to the daemon and methods for sending requests.

use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::error::{Error, Result};

use super::ipc::{
    framing, DaemonRequest, DaemonResponse, MutateOp, MutateResult, QueryOp, QueryResult,
};

/// Connection timeout for daemon communication.
const TIMEOUT_SECS: u64 = 5;

/// A client connection to the daemon.
pub struct DaemonClient {
    stream: UnixStream,
}

impl DaemonClient {
    /// Connect to the daemon at the given socket path.
    pub fn connect(socket_path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(socket_path)
            .map_err(|e| Error::Daemon(format!("failed to connect to daemon: {}", e)))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
            .map_err(|e| Error::Daemon(format!("failed to set read timeout: {}", e)))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
            .map_err(|e| Error::Daemon(format!("failed to set write timeout: {}", e)))?;

        Ok(DaemonClient { stream })
    }

    /// Send a request and receive a response.
    fn request(&mut self, request: DaemonRequest) -> Result<DaemonResponse> {
        framing::write_request(&mut self.stream, &request)?;
        framing::read_response(&mut self.stream)
    }

    /// Execute a query operation.
    pub fn query(&mut self, op: QueryOp) -> Result<QueryResult> {
        match self.request(DaemonRequest::Query(op))? {
            DaemonResponse::QueryResult(result) => Ok(result),
            DaemonResponse::Error { message } => Err(Error::Daemon(message)),
            other => Err(Error::Daemon(format!("unexpected response: {:?}", other))),
        }
    }

    /// Execute a mutation operation.
    pub fn mutate(&mut self, op: MutateOp) -> Result<MutateResult> {
        match self.request(DaemonRequest::Mutate(op))? {
            DaemonResponse::MutateResult(result) => Ok(result),
            DaemonResponse::Error { message } => Err(Error::Daemon(message)),
            other => Err(Error::Daemon(format!("unexpected response: {:?}", other))),
        }
    }
}
