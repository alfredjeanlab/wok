// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Daemon runner: main loop and sync handling.
//!
//! The daemon:
//! 1. Acquires flock for single instance
//! 2. Creates Unix socket for IPC
//! 3. For WebSocket remotes: connects to remote server
//! 4. For Git remotes: manages oplog worktree
//! 5. Handles bidirectional sync

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use std::path::PathBuf;

use tokio::net::UnixListener;
use tokio::sync::mpsc;
use wk_core::{Database, Hlc, Merge, Op, OpPayload, Oplog};

use crate::commands::HlcPersistence;
use crate::config::{get_db_path, Config, RemoteType};
use crate::error::{Error, Result};
use crate::sync::{SyncClient, SyncConfig, Transport, WebSocketTransport};
use crate::wal::Wal;
use crate::worktree::{self, OplogWorktree};

/// Snapshot data: (issues, tags, since_hlc) received from server
type SnapshotData = (Vec<wk_core::Issue>, Vec<(String, String)>, Hlc);

use super::connection::{
    ConnectionConfig, ConnectionEvent, ConnectionManager, SharedConnectionState,
};
use super::ipc::{framing_async, DaemonRequest, DaemonResponse, DaemonStatus};
use super::lifecycle::{get_lock_path, get_pid_path, get_socket_path};

/// Backend-specific state for sync operations.
enum SyncBackend<T: Transport> {
    /// Git remote backend with oplog worktree.
    Git {
        worktree: OplogWorktree,
        /// Write-ahead log for pending ops.
        wal: Wal,
        /// Path to the SQLite cache database.
        db_path: PathBuf,
    },
    /// WebSocket remote backend.
    ///
    /// The client is optional because the connection is established asynchronously
    /// in the background. The daemon remains responsive to IPC while connecting.
    WebSocket {
        /// The sync client, present when connected.
        client: Option<SyncClient<T>>,
        /// Sync configuration for creating new clients.
        sync_config: SyncConfig,
        /// Path to the offline queue for pending operations.
        queue_path: PathBuf,
        /// Path to the SQLite cache database.
        db_path: PathBuf,
        /// Path to the client-side oplog for deduplication.
        oplog_path: PathBuf,
        /// ID of pending ping awaiting pong, if any.
        pending_ping_id: Option<u64>,
        /// When the pending ping was sent.
        last_ping_sent: Option<Instant>,
    },
}

/// State for the daemon that gets passed to IPC handlers.
struct DaemonState<'a, T: Transport> {
    shutdown: &'a Arc<AtomicBool>,
    backend: &'a mut SyncBackend<T>,
    remote_url: &'a str,
    last_sync: &'a mut Option<u64>,
    /// Shared connection state for lock-free status queries.
    connection_state: &'a Arc<SharedConnectionState>,
    pid: u32,
    start_time: Instant,
}

/// Run the daemon for the given daemon directory.
///
/// This function blocks until shutdown is requested.
///
/// # Arguments
/// * `daemon_dir` - Directory for daemon files (socket, pid, lock, sync_queue)
/// * `config` - Configuration loaded from work directory
pub fn run_daemon(daemon_dir: &Path, config: &Config) -> Result<()> {
    // Create tokio runtime
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| Error::Io(std::io::Error::other(format!("tokio: {}", e))))?;

    rt.block_on(run_daemon_async(daemon_dir, config))
}

/// Async implementation of the daemon main loop.
async fn run_daemon_async(daemon_dir: &Path, config: &Config) -> Result<()> {
    // Ensure daemon directory exists (in case workspace path doesn't exist yet)
    std::fs::create_dir_all(daemon_dir)?;

    let lock_path = get_lock_path(daemon_dir);
    let socket_path = get_socket_path(daemon_dir);
    let pid_path = get_pid_path(daemon_dir);

    // Acquire lock file with flock
    let lock_file = acquire_lock(&lock_path)?;

    // Write PID file
    let pid = std::process::id();
    fs::write(&pid_path, pid.to_string())?;

    // Clean up any stale socket
    let _ = fs::remove_file(&socket_path);

    // Create Unix socket listener (tokio async)
    let listener = UnixListener::bind(&socket_path)?;

    // Signal ready - IMPORTANT: do this early so IPC is responsive immediately
    println!("READY");
    let _ = std::io::stdout().flush();

    // Get remote config (for URL display)
    let remote = config
        .remote
        .as_ref()
        .ok_or_else(|| Error::Config("no remote config".to_string()))?;

    // Create shared connection state for lock-free status queries
    let connection_state = Arc::new(SharedConnectionState::new());

    // Initialize backend based on remote type (no connection attempt yet)
    let mut backend = init_backend(daemon_dir, config)?;

    // Extract client-side oplog path from backend
    let client_oplog_path = match &backend {
        SyncBackend::Git { worktree, .. } => worktree.oplog_path.clone(),
        SyncBackend::WebSocket { oplog_path, .. } => oplog_path.clone(),
    };

    // Daemon state
    let shutdown = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();
    let mut last_sync: Option<u64> = None;

    // For Git backend, we're always "connected" (no live connection needed)
    if matches!(&backend, SyncBackend::Git { .. }) {
        connection_state.set(super::connection::STATE_CONNECTED);
    }

    // Set up connection manager for WebSocket backend
    let (mut connection_rx, connection_manager): (
        mpsc::Receiver<ConnectionEvent>,
        Option<ConnectionManager>,
    ) = if let SyncBackend::WebSocket { sync_config, .. } = &backend {
        let conn_config = ConnectionConfig {
            url: sync_config.url.clone(),
            max_retries: sync_config.max_retries,
            max_delay_secs: sync_config.max_delay_secs,
            initial_delay_ms: sync_config.initial_delay_ms,
        };
        let (manager, rx) = ConnectionManager::new(conn_config, Arc::clone(&connection_state));
        // Start initial connection attempt in background
        manager.spawn_connect_task();
        (rx, Some(manager))
    } else {
        // Git backend doesn't need connection manager
        let (_tx, rx) = mpsc::channel(1);
        (rx, None)
    };

    // Main async loop using tokio::select!
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match &mut backend {
            SyncBackend::WebSocket {
                ref mut client,
                ref db_path,
                ref sync_config,
                ref queue_path,
                ref mut pending_ping_id,
                ref mut last_ping_sent,
                ..
            } => {
                let is_connected = client.as_ref().is_some_and(|c| c.is_connected());
                let db_path_clone = db_path.clone();
                let oplog_path_clone = client_oplog_path.clone();
                let sync_config_clone = sync_config.clone();
                let queue_path_clone = queue_path.clone();

                // Heartbeat timing - extract state before async blocks
                let heartbeat_interval_ms = sync_config.heartbeat_interval_ms;
                let heartbeat_timeout_ms = sync_config.heartbeat_timeout_ms;
                let heartbeat_enabled = heartbeat_interval_ms > 0 && is_connected;
                let has_pending_ping = pending_ping_id.is_some();
                // Calculate remaining timeout based on when ping was sent
                let pong_timeout_remaining = if let Some(sent) = last_ping_sent {
                    let elapsed = sent.elapsed().as_millis() as u64;
                    if elapsed >= heartbeat_timeout_ms {
                        Duration::ZERO
                    } else {
                        Duration::from_millis(heartbeat_timeout_ms - elapsed)
                    }
                } else {
                    Duration::MAX
                };

                tokio::select! {
                    // Accept IPC connections - ALWAYS responsive
                    result = listener.accept() => {
                        if let Ok((stream, _)) = result {
                            let mut state = DaemonState {
                                shutdown: &shutdown,
                                backend: &mut backend,
                                remote_url: &remote.url,
                                last_sync: &mut last_sync,
                                connection_state: &connection_state,
                                pid,
                                start_time,
                            };
                            let _ = handle_ipc_request_async(stream, &mut state).await;
                        }
                    }

                    // Handle connection events from background task
                    Some(event) = connection_rx.recv() => {
                        match event {
                            ConnectionEvent::Connected(transport) => {
                                // Create a new SyncClient with the connected transport
                                match SyncClient::with_transport(
                                    sync_config_clone,
                                    transport,
                                    &queue_path_clone,
                                ) {
                                    Ok(mut new_client) => {
                                        // Mark the client as connected (transport is already connected)
                                        new_client.set_connected();

                                        // Initialize client's last_hlc from persisted SERVER state
                                        if let Some(server_hlc) = HlcPersistence::server(daemon_dir).read() {
                                            new_client.set_last_hlc(server_hlc);
                                        }

                                        // Update backend with new client and clear heartbeat state
                                        if let SyncBackend::WebSocket { client, pending_ping_id, last_ping_sent, .. } = &mut backend {
                                            *client = Some(new_client);
                                            *pending_ping_id = None;
                                            *last_ping_sent = None;
                                        }

                                        // Sync on connect
                                        if let SyncBackend::WebSocket { client: Some(c), db_path, .. } = &mut backend {
                                            // Note: Errors are silently ignored here since we'll retry on next sync
                                            let _ = sync_on_reconnect(c, db_path, &oplog_path_clone).await;
                                        }
                                    }
                                    Err(_) => {
                                        // Trigger reconnection
                                        if let Some(ref manager) = connection_manager {
                                            manager.spawn_connect_task();
                                        }
                                    }
                                }
                            }
                            ConnectionEvent::Failed { .. } => {
                                // Connection manager has given up, schedule retry after delay.
                                // Use spawn_delayed_connect to avoid blocking IPC handling.
                                if let Some(ref manager) = connection_manager {
                                    manager.spawn_delayed_connect(Duration::from_secs(5));
                                }
                            }
                        }
                    }

                    // Handle WebSocket events (only when connected)
                    result = async {
                        if let SyncBackend::WebSocket { client: Some(c), .. } = &mut backend {
                            c.recv().await
                        } else {
                            // Never ready if no client
                            std::future::pending().await
                        }
                    }, if is_connected => {
                        match result {
                            Ok(Some(msg)) => {
                                // Clear heartbeat state on any message (connection is alive)
                                if let SyncBackend::WebSocket {
                                    pending_ping_id: ping_id,
                                    last_ping_sent: ping_sent,
                                    ..
                                } = &mut backend {
                                    // Check if this is a Pong response to our ping
                                    if let wk_core::protocol::ServerMessage::Pong { id } = &msg {
                                        if *ping_id == Some(*id) {
                                            *ping_id = None;
                                            *ping_sent = None;
                                        }
                                    } else {
                                        // Any other message also means connection is alive
                                        *ping_id = None;
                                        *ping_sent = None;
                                    }
                                }
                                let _ = handle_server_message(&msg, &db_path_clone, &oplog_path_clone);
                            }
                            Ok(None) | Err(_) => {
                                // Connection closed or error - handle connection lost
                                handle_connection_lost(
                                    &mut backend,
                                    &connection_state,
                                    &connection_manager,
                                );
                            }
                        }
                    }

                    // Heartbeat interval timer - send ping when interval elapses
                    _ = tokio::time::sleep(Duration::from_millis(heartbeat_interval_ms)), if heartbeat_enabled && !has_pending_ping => {
                        // Generate a unique ping ID
                        let ping_id = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_nanos() as u64)
                            .unwrap_or(1);

                        // Send ping
                        let ping_sent = if let SyncBackend::WebSocket { client: Some(c), .. } = &mut backend {
                            c.ping(ping_id).await.is_ok()
                        } else {
                            false
                        };

                        if ping_sent {
                            if let SyncBackend::WebSocket {
                                pending_ping_id: pid,
                                last_ping_sent: lps,
                                ..
                            } = &mut backend {
                                *pid = Some(ping_id);
                                *lps = Some(Instant::now());
                            }
                        }
                    }

                    // Pong timeout timer - detect dead connection
                    _ = tokio::time::sleep(pong_timeout_remaining), if heartbeat_enabled && has_pending_ping => {
                        handle_connection_lost(
                            &mut backend,
                            &connection_state,
                            &connection_manager,
                        );
                    }
                }
            }
            SyncBackend::Git { .. } => {
                // Git backend: just handle IPC, no real-time sync
                tokio::select! {
                    result = listener.accept() => {
                        if let Ok((stream, _)) = result {
                            let mut state = DaemonState {
                                shutdown: &shutdown,
                                backend: &mut backend,
                                remote_url: &remote.url,
                                last_sync: &mut last_sync,
                                connection_state: &connection_state,
                                pid,
                                start_time,
                            };
                            let _ = handle_ipc_request_async(stream, &mut state).await;
                        }
                    }
                }
            }
        }
    }

    // Cancel any pending connection attempts
    if let Some(ref manager) = connection_manager {
        manager.cancel();
    }

    // Cleanup
    drop(lock_file);
    let _ = fs::remove_file(&socket_path);
    let _ = fs::remove_file(&pid_path);

    Ok(())
}

/// Initialize the appropriate backend based on remote type.
///
/// For WebSocket backends, the client is None initially. The connection
/// is established asynchronously by the ConnectionManager.
fn init_backend(daemon_dir: &Path, config: &Config) -> Result<SyncBackend<WebSocketTransport>> {
    let remote = config
        .remote
        .as_ref()
        .ok_or_else(|| Error::Config("no remote config".to_string()))?;

    match remote.remote_type() {
        RemoteType::Git => {
            // Get the work_dir (daemon_dir is the work_dir or workspace dir)
            let work_dir = daemon_dir;

            // Initialize the oplog worktree
            let worktree = worktree::init_oplog_worktree(work_dir, remote)?;

            // WAL for pending ops that haven't been committed to git yet
            let wal_path = daemon_dir.join("pending_ops.jsonl");
            let wal = Wal::open(&wal_path)?;

            // Get database path for cache rebuild
            let db_path = get_db_path(daemon_dir, config);

            Ok(SyncBackend::Git {
                worktree,
                wal,
                db_path,
            })
        }
        RemoteType::WebSocket => {
            let queue_path = daemon_dir.join("sync_queue.jsonl");
            let sync_config = SyncConfig {
                url: remote.url.clone(),
                max_retries: remote.reconnect_max_retries,
                max_delay_secs: remote.reconnect_max_delay_secs,
                initial_delay_ms: 100,
                heartbeat_interval_ms: remote.heartbeat_interval_ms,
                heartbeat_timeout_ms: remote.heartbeat_timeout_ms,
            };

            let db_path = get_db_path(daemon_dir, config);
            let oplog_path = daemon_dir.join("client_oplog.jsonl");

            // Client starts as None - connection established asynchronously
            Ok(SyncBackend::WebSocket {
                client: None,
                sync_config,
                queue_path,
                db_path,
                oplog_path,
                pending_ping_id: None,
                last_ping_sent: None,
            })
        }
    }
}

/// Handle an IPC request from a CLI process (async version).
async fn handle_ipc_request_async<T: Transport>(
    mut stream: tokio::net::UnixStream,
    state: &mut DaemonState<'_, T>,
) -> Result<()> {
    let request = framing_async::read_request(&mut stream).await?;

    let response = match request {
        DaemonRequest::Status => {
            let pending_ops = get_pending_ops_count(state.backend);
            let uptime_secs = state.start_time.elapsed().as_secs();
            // Use shared connection state for accurate status
            let connected = state.connection_state.is_connected();
            let connecting = state.connection_state.is_connecting();
            DaemonResponse::Status(DaemonStatus::new(
                connected,
                connecting,
                state.remote_url.to_string(),
                pending_ops,
                *state.last_sync,
                state.pid,
                uptime_secs,
            ))
        }
        DaemonRequest::SyncNow => {
            // Perform sync based on backend type
            match perform_sync_async(state.backend, state.connection_state).await {
                Ok(ops_synced) => {
                    *state.last_sync = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    );
                    DaemonResponse::SyncComplete { ops_synced }
                }
                Err(e) => DaemonResponse::Error {
                    message: e.to_string(),
                },
            }
        }
        DaemonRequest::Shutdown => {
            state.shutdown.store(true, Ordering::Relaxed);
            DaemonResponse::ShuttingDown
        }
        DaemonRequest::Ping => DaemonResponse::Pong,
        DaemonRequest::Hello { .. } => DaemonResponse::Hello {
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    framing_async::write_response(&mut stream, &response).await?;
    Ok(())
}

/// Handle a message received from the server.
fn handle_server_message(
    msg: &wk_core::protocol::ServerMessage,
    db_path: &Path,
    oplog_path: &Path,
) -> Result<()> {
    use wk_core::protocol::ServerMessage;

    let daemon_dir = oplog_path
        .parent()
        .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

    match msg {
        ServerMessage::Op(op) => {
            apply_op_to_cache(op, db_path, oplog_path)?;
            // Persist SERVER HLC for future sync requests
            let _ = HlcPersistence::server(daemon_dir).update(op.id);
            // Also update local HLC to incorporate received HLC
            let _ = HlcPersistence::last(daemon_dir).update(op.id);
        }
        ServerMessage::SyncResponse { ops } => {
            for op in ops {
                apply_op_to_cache(op, db_path, oplog_path)?;
            }
            // Persist max SERVER HLC from sync response
            if let Some(max_hlc) = ops.iter().map(|op| op.id).max() {
                let _ = HlcPersistence::server(daemon_dir).update(max_hlc);
                let _ = HlcPersistence::last(daemon_dir).update(max_hlc);
            }
        }
        ServerMessage::SnapshotResponse {
            issues,
            tags,
            since,
        } => {
            // Apply snapshot: convert issues to ops and apply through Merge trait
            apply_snapshot_to_cache(issues, tags, *since, db_path, oplog_path, daemon_dir)?;
        }
        // Ignore other message types (Pong, Error)
        _ => {}
    }

    Ok(())
}

/// Apply a received operation to the local cache database.
fn apply_op_to_cache(op: &Op, db_path: &Path, oplog_path: &Path) -> Result<()> {
    // Load oplog and check for duplicates
    let mut oplog =
        Oplog::open(oplog_path).map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;

    let is_new = oplog
        .append(op)
        .map_err(|e| Error::Sync(format!("failed to append to oplog: {}", e)))?;

    if !is_new {
        // Already seen this operation - skip
        return Ok(());
    }

    // Apply to database
    let mut db =
        Database::open(db_path).map_err(|e| Error::Sync(format!("failed to open db: {}", e)))?;
    db.apply(op)
        .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
    Ok(())
}

/// Apply a snapshot response to the local cache database.
fn apply_snapshot_to_cache(
    issues: &[wk_core::Issue],
    tags: &[(String, String)],
    since: Hlc,
    db_path: &Path,
    oplog_path: &Path,
    daemon_dir: &Path,
) -> Result<()> {
    use wk_core::Merge;

    let mut oplog =
        Oplog::open(oplog_path).map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;
    let mut db = wk_core::Database::open(db_path)
        .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

    // Convert snapshot to ops
    let mut all_ops: Vec<Op> = Vec::new();
    for (index, issue) in issues.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        all_ops.extend(snapshot_issue_to_ops(issue, index as u32));
    }

    // Add label ops with unique synthetic HLCs
    // Start counter after issue ops to avoid collisions
    #[allow(clippy::cast_possible_truncation)]
    let label_offset = issues.len() as u32;
    for (index, (issue_id, label)) in tags.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        all_ops.push(Op::new(
            Hlc::new(0, label_offset + index as u32, 0),
            OpPayload::AddLabel {
                issue_id: issue_id.clone(),
                label: label.clone(),
            },
        ));
    }

    // Sort and apply through Merge trait with deduplication
    all_ops.sort();
    for op in &all_ops {
        // Only apply if new (not a duplicate)
        if oplog
            .append(op)
            .map_err(|e| Error::Sync(format!("failed to append to oplog: {}", e)))?
        {
            db.apply(op)
                .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
        }
    }

    // Persist the snapshot's high-water HLC for future sync requests
    let _ = HlcPersistence::server(daemon_dir).update(since);
    let _ = HlcPersistence::last(daemon_dir).update(since);

    Ok(())
}

/// Perform sync on reconnect: flush queue and request catch-up.
async fn sync_on_reconnect<T: Transport>(
    client: &mut SyncClient<T>,
    db_path: &Path,
    oplog_path: &Path,
) -> Result<()> {
    // Add pending ops to oplog BEFORE sending (matching sync_websocket behavior)
    // This ensures we recognize them as duplicates when server broadcasts them back
    let daemon_dir = oplog_path
        .parent()
        .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;
    let queue_path = daemon_dir.join("sync_queue.jsonl");

    if let Ok(queue) = crate::sync::OfflineQueue::open(&queue_path) {
        if let Ok(pending_ops) = queue.peek_all() {
            if !pending_ops.is_empty() {
                if let Ok(mut oplog) = Oplog::open(oplog_path) {
                    for op in &pending_ops {
                        // Ignore duplicate status - we just want them in the oplog
                        let _ = oplog.append(op);
                    }
                }
            }
        }
    }

    // Flush any queued offline operations
    if let Ok(flushed) = client.flush_queue().await {
        if flushed > 0 {
            tracing::info!("Flushed {} queued operations", flushed);
        }
    }

    // Request sync to catch up on missed ops, using persisted SERVER HLC (not client.last_hlc).
    // client.last_hlc() gets contaminated by local ops, which would cause new clients
    // to request sync(since=local_hlc) instead of snapshot, missing earlier ops.
    let sync_since = HlcPersistence::server(daemon_dir).read();

    if let Some(since) = sync_since {
        let _ = client.request_sync(since).await;
    } else {
        // No server HLC means this is a new client - request snapshot
        let _ = client.request_snapshot().await;
    }

    // Receive and apply sync/snapshot response
    // Note: The server broadcasts our own ops back to us before responding with the
    // sync/snapshot response, so we need to keep receiving until we get the response.
    let mut got_response = false;
    while let Ok(Some(msg)) = client.recv().await {
        use wk_core::protocol::ServerMessage;

        // Apply the message to local state
        handle_server_message(&msg, db_path, oplog_path)?;

        // Check if this is the sync/snapshot response (signals end of sync)
        match &msg {
            ServerMessage::SyncResponse { .. } | ServerMessage::SnapshotResponse { .. } => {
                got_response = true;
                break;
            }
            _ => {
                // Keep receiving (Op broadcasts, etc.)
                continue;
            }
        }
    }

    // Clear the queue only after successfully receiving sync response
    if got_response {
        let _ = client.clear_queue();
    }

    Ok(())
}

/// Get the count of pending operations from the backend.
fn get_pending_ops_count<T: Transport>(backend: &SyncBackend<T>) -> usize {
    match backend {
        SyncBackend::Git { wal, .. } => wal.count().unwrap_or(0),
        SyncBackend::WebSocket {
            client, queue_path, ..
        } => {
            // If we have a client, use its count; otherwise read from queue directly
            if let Some(c) = client {
                c.pending_ops_count().unwrap_or(0)
            } else {
                // Read directly from offline queue when no client available
                crate::sync::OfflineQueue::open(queue_path)
                    .ok()
                    .and_then(|q| q.len().ok())
                    .unwrap_or(0)
            }
        }
    }
}

/// Perform a sync operation based on the backend type (async version).
async fn perform_sync_async<T: Transport>(
    backend: &mut SyncBackend<T>,
    connection_state: &Arc<SharedConnectionState>,
) -> Result<usize> {
    match backend {
        SyncBackend::Git {
            worktree,
            wal,
            db_path,
        } => sync_git(worktree, wal, db_path),
        SyncBackend::WebSocket {
            client,
            db_path,
            oplog_path,
            ..
        } => {
            if let Some(c) = client {
                sync_websocket(c, db_path, oplog_path, connection_state).await
            } else {
                // Not connected yet - report connection status
                if connection_state.is_connecting() {
                    Err(Error::Sync(format!(
                        "connecting to server (attempt {})",
                        connection_state.attempt()
                    )))
                } else {
                    Err(Error::Sync("not connected to server".to_string()))
                }
            }
        }
    }
}

/// Convert a snapshot issue to synthetic operations for HLC-aware merge.
///
/// Uses a unique index to create distinct HLCs for CreateIssue ops since
/// the oplog deduplicates by HLC, not by payload.
fn snapshot_issue_to_ops(issue: &wk_core::Issue, index: u32) -> Vec<Op> {
    let mut ops = Vec::new();

    // CreateIssue with unique synthetic HLC (using index to differentiate)
    // The wall_ms=0 ensures these are considered "oldest" and real ops win
    ops.push(Op::new(
        Hlc::new(0, index, 0),
        OpPayload::CreateIssue {
            id: issue.id.clone(),
            issue_type: issue.issue_type,
            title: issue.title.clone(),
        },
    ));

    // SetTitle with issue's title HLC
    if let Some(hlc) = issue.last_title_hlc {
        ops.push(Op::new(
            hlc,
            OpPayload::SetTitle {
                issue_id: issue.id.clone(),
                title: issue.title.clone(),
            },
        ));
    }

    // SetStatus with issue's status HLC
    if let Some(hlc) = issue.last_status_hlc {
        ops.push(Op::new(
            hlc,
            OpPayload::SetStatus {
                issue_id: issue.id.clone(),
                status: issue.status,
                reason: None,
            },
        ));
    }

    // SetType with issue's type HLC
    if let Some(hlc) = issue.last_type_hlc {
        ops.push(Op::new(
            hlc,
            OpPayload::SetType {
                issue_id: issue.id.clone(),
                issue_type: issue.issue_type,
            },
        ));
    }

    ops
}

/// Perform WebSocket sync: flush queue, request sync.
///
/// The client must already be connected. Connection is established asynchronously
/// by the ConnectionManager, not by this function.
async fn sync_websocket<T: Transport>(
    client: &mut SyncClient<T>,
    db_path: &Path,
    oplog_path: &Path,
    _connection_state: &Arc<SharedConnectionState>,
) -> Result<usize> {
    use wk_core::protocol::ServerMessage;
    use wk_core::Merge;

    // Verify we're connected (caller should ensure this)
    if !client.is_connected() {
        return Err(Error::Sync("not connected to server".to_string()));
    }

    // Derive daemon_dir from oplog path (both are in the same daemon directory)
    let daemon_dir = oplog_path
        .parent()
        .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

    // Use persisted SERVER HLC for sync baseline, not client.last_hlc().
    // client.last_hlc() gets contaminated by local ops (via send_op), which
    // would cause new clients to request sync(since=local_hlc) instead of
    // snapshot, missing earlier ops from other clients.
    let sync_since = HlcPersistence::server(daemon_dir).read();
    let queue_path = daemon_dir.join("sync_queue.jsonl");

    // Read operations from queue without dequeuing
    let queue = crate::sync::OfflineQueue::open(&queue_path)
        .map_err(|e| Error::Sync(format!("failed to open queue: {}", e)))?;
    let pending_ops = queue
        .peek_all()
        .map_err(|e| Error::Sync(format!("failed to read queue: {}", e)))?;

    // Add pending operations to client oplog before sending to server
    // This ensures that when the server broadcasts them back, we recognize them as duplicates
    if !pending_ops.is_empty() {
        let mut oplog = Oplog::open(oplog_path)
            .map_err(|e| Error::Sync(format!("failed to open oplog for pending ops: {}", e)))?;
        for op in &pending_ops {
            // Ignore duplicate status - we just want them in the oplog
            let _ = oplog
                .append(op)
                .map_err(|e| Error::Sync(format!("failed to append pending op to oplog: {}", e)))?;
        }
    }

    // Flush the offline queue to the server
    let flushed = client
        .flush_queue()
        .await
        .map_err(|e| Error::Sync(format!("failed to flush queue: {}", e)))?;

    // Request sync from server based on persisted server HLC (or snapshot if none)
    if let Some(hlc) = sync_since {
        client
            .request_sync(hlc)
            .await
            .map_err(|e| Error::Sync(format!("failed to request sync: {}", e)))?;
    } else {
        client
            .request_snapshot()
            .await
            .map_err(|e| Error::Sync(format!("failed to request snapshot: {}", e)))?;
    }

    // Receive and apply operations from server
    let mut ops_received = 0;
    let mut received_ops = Vec::new();
    let mut snapshot_data: Option<SnapshotData> = None;

    // Set a timeout for receiving sync response
    // Returns true if we successfully received a sync/snapshot response
    let timeout_result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            match client.recv().await {
                Ok(Some(ServerMessage::SyncResponse { ops })) => {
                    received_ops.extend(ops.clone());
                    ops_received += ops.len();
                    return true; // Successfully received sync response
                }
                Ok(Some(ServerMessage::SnapshotResponse {
                    issues,
                    tags,
                    since,
                })) => {
                    // Snapshot contains full issues, tags, and high-water HLC
                    ops_received += issues.len();
                    snapshot_data = Some((issues, tags, since));
                    return true; // Successfully received snapshot response
                }
                Ok(Some(ServerMessage::Op(op))) => {
                    received_ops.push(op);
                    ops_received += 1;
                }
                Ok(Some(_)) => {
                    // Ignore other messages (Pong, Error)
                    continue;
                }
                Ok(None) => {
                    // Connection closed
                    return false;
                }
                Err(_) => {
                    return false;
                }
            }
        }
    });

    // Check if we successfully received a sync response (not timed out, not connection error)
    let sync_response_received = matches!(timeout_result.await, Ok(true));

    // Clear the send queue only if we successfully received a sync response.
    // This ensures ops are not lost if the connection drops before the server
    // acknowledges receipt. Duplicate ops on resend are handled by the server.
    if sync_response_received {
        let _ = client.clear_queue();
    }

    // Apply received ops to cache
    if !received_ops.is_empty() {
        let daemon_dir = oplog_path
            .parent()
            .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

        let mut oplog = Oplog::open(oplog_path)
            .map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;
        let mut db = wk_core::Database::open(db_path)
            .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

        received_ops.sort();
        for op in &received_ops {
            // Only apply if new (not a duplicate)
            if oplog
                .append(op)
                .map_err(|e| Error::Sync(format!("failed to append to oplog: {}", e)))?
            {
                db.apply(op)
                    .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
            }
        }

        // Persist the max HLC from received ops for future sync requests
        if let Some(max_hlc) = received_ops.iter().map(|op| op.id).max() {
            let _ = HlcPersistence::server(daemon_dir).update(max_hlc);
            let _ = HlcPersistence::last(daemon_dir).update(max_hlc);
            client.set_last_hlc(max_hlc);
        }
    }

    // Apply snapshot data if received
    // Convert snapshot to ops and apply through Merge trait for HLC resolution
    if let Some((issues, tags, since)) = snapshot_data {
        let daemon_dir = oplog_path
            .parent()
            .ok_or_else(|| Error::Sync("invalid oplog path - no parent directory".to_string()))?;

        let mut oplog = Oplog::open(oplog_path)
            .map_err(|e| Error::Sync(format!("failed to open oplog: {}", e)))?;
        let mut db = wk_core::Database::open(db_path)
            .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

        // Convert snapshot to ops
        let mut all_ops: Vec<Op> = Vec::new();
        for (index, issue) in issues.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            all_ops.extend(snapshot_issue_to_ops(issue, index as u32));
        }

        // Add label ops with unique synthetic HLCs
        // Start counter after issue ops to avoid collisions
        let label_offset = issues.len() as u32;
        for (index, (issue_id, label)) in tags.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            all_ops.push(Op::new(
                Hlc::new(0, label_offset + index as u32, 0),
                OpPayload::AddLabel {
                    issue_id: issue_id.clone(),
                    label: label.clone(),
                },
            ));
        }

        // Sort and apply through Merge trait with deduplication
        all_ops.sort();
        for op in &all_ops {
            // Only apply if new (not a duplicate)
            if oplog
                .append(op)
                .map_err(|e| Error::Sync(format!("failed to append to oplog: {}", e)))?
            {
                db.apply(op)
                    .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
            }
        }

        // Persist the snapshot's high-water HLC for future sync requests
        // This ensures new operations will have timestamps higher than what the server has seen
        let _ = HlcPersistence::server(daemon_dir).update(since);
        let _ = HlcPersistence::last(daemon_dir).update(since);

        // Update client's last_hlc so subsequent sync requests use the correct baseline
        client.set_last_hlc(since);
    }

    Ok(flushed + ops_received)
}

/// Perform git sync: fetch, merge oplogs, flush WAL, commit, and push.
///
/// Sync algorithm:
/// 1. Fetch from remote
/// 2. Read local oplog
/// 3. Check if remote has changes (compare HEADs)
/// 4. If remote has changes:
///    a. Read remote oplog
///    b. Merge local and remote oplogs (HLC sort + dedup)
///    c. Write merged oplog
/// 5. Append pending ops from WAL to oplog
/// 6. Commit and push
/// 7. Rebuild SQLite cache if new ops pulled
/// 8. Return count of ops pushed
fn sync_git(worktree: &OplogWorktree, wal: &Wal, db_path: &Path) -> Result<usize> {
    use std::collections::HashSet;
    use std::process::Command;

    let worktree_path = &worktree.path;

    // 1. Fetch from remote (ignore errors - remote may not exist yet)
    let _ = Command::new("git")
        .current_dir(worktree_path)
        .args(["fetch", "origin", &worktree.branch])
        .output();

    // 2. Read local oplog
    let mut local_ops = worktree::read_oplog(&worktree.oplog_path)?;

    // 3. Check if remote has changes
    let local_head = git_rev_parse(worktree_path, "HEAD").ok();
    let remote_head = git_rev_parse(worktree_path, &format!("origin/{}", worktree.branch)).ok();

    let (pulled_ops, pulled_count) = if local_head != remote_head && remote_head.is_some() {
        // 4. Remote has changes - merge oplogs
        // For now, use git merge to bring in remote changes, then re-read
        let _ = Command::new("git")
            .current_dir(worktree_path)
            .args([
                "merge",
                "--strategy-option=theirs",
                &format!("origin/{}", worktree.branch),
            ])
            .output();

        // Re-read the merged oplog
        let merged_ops = worktree::read_oplog(&worktree.oplog_path)?;

        // Find new ops that we pulled
        let local_ids: HashSet<_> = local_ops.iter().map(|op| &op.id).collect();
        let new_ops: Vec<Op> = merged_ops
            .iter()
            .filter(|op| !local_ids.contains(&op.id))
            .cloned()
            .collect();
        let new_count = new_ops.len();

        local_ops = merged_ops;
        (new_ops, new_count)
    } else {
        (Vec::new(), 0)
    };

    // 5. Take pending ops from WAL and append to oplog
    let pending_ops = wal.take_all()?;
    let pushed_count = pending_ops.len();

    if !pending_ops.is_empty() {
        // Append pending ops to oplog (deduplicating)
        let existing_ids: HashSet<_> = local_ops.iter().map(|op| &op.id).collect();
        let new_ops: Vec<_> = pending_ops
            .into_iter()
            .filter(|op| !existing_ids.contains(&op.id))
            .collect();

        if !new_ops.is_empty() {
            worktree::append_oplog(&worktree.oplog_path, &new_ops)?;

            // 6. Git add, commit, push
            let _ = Command::new("git")
                .current_dir(worktree_path)
                .args(["add", worktree::OPLOG_FILE])
                .output();

            let _ = Command::new("git")
                .current_dir(worktree_path)
                .args(["commit", "-m", "wk sync"])
                .output();

            let _ = Command::new("git")
                .current_dir(worktree_path)
                .args(["push", "origin", &worktree.branch])
                .output();
        }
    }

    // 7. Rebuild SQLite cache if new ops were pulled
    if pulled_count > 0 {
        rebuild_cache(db_path, &pulled_ops)?;
    }

    Ok(pushed_count)
}

/// Rebuild the SQLite cache by applying new operations.
///
/// This applies pulled operations to the existing database, using the
/// Merge trait to handle conflicts with HLC-based resolution.
fn rebuild_cache(db_path: &Path, new_ops: &[Op]) -> Result<()> {
    if new_ops.is_empty() {
        return Ok(());
    }

    // Open database and apply new ops
    let mut db = Database::open(db_path)
        .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

    // Sort ops by HLC for proper ordering
    let mut sorted_ops = new_ops.to_vec();
    sorted_ops.sort();

    // Apply all new ops using the Merge trait
    db.apply_all(&sorted_ops)
        .map_err(|e| Error::Sync(format!("failed to apply ops: {}", e)))?;

    Ok(())
}

/// Run git rev-parse to get a commit hash.
fn git_rev_parse(worktree_path: &std::path::Path, refspec: &str) -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .current_dir(worktree_path)
        .args(["rev-parse", refspec])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(Error::Config(format!("git rev-parse {} failed", refspec)))
    }
}

/// Handle connection lost: clear client, clear heartbeat state, set disconnected, spawn reconnect.
fn handle_connection_lost<T: Transport>(
    backend: &mut SyncBackend<T>,
    connection_state: &Arc<SharedConnectionState>,
    connection_manager: &Option<ConnectionManager>,
) {
    if let SyncBackend::WebSocket {
        client,
        pending_ping_id,
        last_ping_sent,
        ..
    } = backend
    {
        *client = None;
        *pending_ping_id = None;
        *last_ping_sent = None;
    }
    connection_state.set(super::connection::STATE_DISCONNECTED);
    if let Some(ref manager) = connection_manager {
        manager.spawn_connect_task();
    }
}

/// Acquire an exclusive lock on the lock file.
fn acquire_lock(lock_path: &Path) -> Result<File> {
    use fs2::FileExt;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;

    // Try to acquire exclusive lock (non-blocking)
    file.try_lock_exclusive()
        .map_err(|e| Error::Io(std::io::Error::other(format!("lock already held: {}", e))))?;

    Ok(file)
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
