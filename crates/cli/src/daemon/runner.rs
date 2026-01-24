// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Daemon runner: main loop and IPC handling.
//!
//! The daemon:
//! 1. Acquires flock for single instance
//! 2. Creates Unix socket for IPC
//! 3. For WebSocket remotes: connects to remote server
//! 4. For Git remotes: manages oplog worktree
//! 5. Handles bidirectional sync

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::net::UnixListener;
use tokio::sync::mpsc;

use super::cache::handle_server_message;
use super::connection::{
    ConnectionConfig, ConnectionEvent, ConnectionManager, SharedConnectionState,
};
use super::ipc::{framing_async, DaemonRequest, DaemonResponse, DaemonStatus};
use super::lifecycle::{get_lock_path, get_pid_path, get_socket_path};
use super::sync::{perform_sync_async, sync_on_reconnect};
use crate::commands::HlcPersistence;
use crate::config::{get_db_path, Config, RemoteType};
use crate::error::{Error, Result};
use crate::sync::{SyncClient, SyncConfig, Transport, WebSocketTransport};
use crate::wal::Wal;
use crate::worktree::{self, OplogWorktree};

/// Backend-specific state for sync operations.
pub(super) enum SyncBackend<T: Transport> {
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
