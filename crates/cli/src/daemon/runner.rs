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
use wk_core::{Database, Merge, Op};

use crate::config::{get_db_path, Config, RemoteType};
use crate::error::{Error, Result};
use crate::sync::{SyncClient, SyncConfig, Transport, WebSocketTransport};
use crate::wal::Wal;
use crate::worktree::{self, OplogWorktree};

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
    WebSocket {
        client: SyncClient<T>,
        /// Path to the SQLite cache database.
        db_path: PathBuf,
    },
}

/// State for the daemon that gets passed to IPC handlers.
struct DaemonState<'a, T: Transport> {
    shutdown: &'a Arc<AtomicBool>,
    backend: &'a mut SyncBackend<T>,
    remote_url: &'a str,
    last_sync: &'a mut Option<u64>,
    connected: bool,
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

    // Signal ready
    println!("READY");
    let _ = std::io::stdout().flush();

    // Get remote config (for URL display)
    let remote = config
        .remote
        .as_ref()
        .ok_or_else(|| Error::Config("no remote config".to_string()))?;

    // Initialize backend based on remote type
    let transport = WebSocketTransport::new();
    let mut backend = init_backend(daemon_dir, config, transport)?;

    // Daemon state
    let shutdown = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();
    let mut last_sync: Option<u64> = None;
    let mut connected = matches!(&backend, SyncBackend::Git { .. });

    // For WebSocket backend, attempt initial connection
    if let SyncBackend::WebSocket { ref mut client, .. } = backend {
        if client.connect_with_retry().await.is_ok() {
            connected = true;
        }
    }

    // Main async loop using tokio::select!
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match &mut backend {
            SyncBackend::WebSocket {
                ref mut client,
                ref db_path,
            } => {
                let is_connected = client.is_connected();
                tokio::select! {
                    // Accept IPC connections
                    result = listener.accept() => {
                        if let Ok((stream, _)) = result {
                            let mut state = DaemonState {
                                shutdown: &shutdown,
                                backend: &mut backend,
                                remote_url: &remote.url,
                                last_sync: &mut last_sync,
                                connected,
                                pid,
                                start_time,
                            };
                            if let Err(e) = handle_ipc_request_async(stream, &mut state).await {
                                eprintln!("IPC error: {}", e);
                            }
                            // Update connected status after IPC (sync may have connected)
                            if let SyncBackend::WebSocket { client, .. } = &state.backend {
                                connected = client.is_connected();
                            }
                        }
                    }

                    // Handle WebSocket events (when connected)
                    result = async { client.recv().await }, if is_connected => {
                        match result {
                            Ok(Some(msg)) => {
                                if let Err(e) = handle_server_message(&msg, db_path) {
                                    eprintln!("Error handling server message: {}", e);
                                }
                            }
                            Ok(None) => {
                                // Connection closed
                                connected = false;
                            }
                            Err(_) => {
                                // Connection error
                                connected = false;
                            }
                        }
                    }

                    // Reconnection timer (when disconnected)
                    _ = tokio::time::sleep(Duration::from_secs(5)), if !is_connected => {
                        if let SyncBackend::WebSocket { client, db_path } = &mut backend {
                            if client.connect_with_retry().await.is_ok() {
                                connected = true;
                                // Sync on connect
                                if let Err(e) = sync_on_reconnect(client, db_path).await {
                                    eprintln!("Sync on reconnect failed: {}", e);
                                }
                            }
                        }
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
                                connected,
                                pid,
                                start_time,
                            };
                            if let Err(e) = handle_ipc_request_async(stream, &mut state).await {
                                eprintln!("IPC error: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    drop(lock_file);
    let _ = fs::remove_file(&socket_path);
    let _ = fs::remove_file(&pid_path);

    Ok(())
}

/// Initialize the appropriate backend based on remote type.
fn init_backend<T: Transport>(
    daemon_dir: &Path,
    config: &Config,
    transport: T,
) -> Result<SyncBackend<T>> {
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
            };
            let client = SyncClient::with_transport(sync_config, transport, &queue_path)?;
            let db_path = get_db_path(daemon_dir, config);
            Ok(SyncBackend::WebSocket { client, db_path })
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
            DaemonResponse::Status(DaemonStatus::new(
                state.connected,
                state.remote_url.to_string(),
                pending_ops,
                *state.last_sync,
                state.pid,
                uptime_secs,
            ))
        }
        DaemonRequest::SyncNow => {
            // Perform sync based on backend type
            match perform_sync_async(state.backend).await {
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
fn handle_server_message(msg: &wk_core::protocol::ServerMessage, db_path: &Path) -> Result<()> {
    use wk_core::protocol::ServerMessage;

    match msg {
        ServerMessage::Op(op) => {
            apply_op_to_cache(op, db_path)?;
        }
        ServerMessage::SyncResponse { ops } => {
            for op in ops {
                apply_op_to_cache(op, db_path)?;
            }
        }
        // Ignore other message types (Pong, Error, SnapshotResponse, etc.)
        _ => {}
    }

    Ok(())
}

/// Apply a received operation to the local cache database.
fn apply_op_to_cache(op: &Op, db_path: &Path) -> Result<()> {
    let mut db =
        Database::open(db_path).map_err(|e| Error::Sync(format!("failed to open db: {}", e)))?;
    db.apply(op)
        .map_err(|e| Error::Sync(format!("failed to apply op: {}", e)))?;
    Ok(())
}

/// Perform sync on reconnect: flush queue and request catch-up.
async fn sync_on_reconnect<T: Transport>(client: &mut SyncClient<T>, db_path: &Path) -> Result<()> {
    // Flush any queued offline operations
    if let Ok(flushed) = client.flush_queue().await {
        if flushed > 0 {
            tracing::info!("Flushed {} queued operations", flushed);
        }
    }

    // Request sync to catch up on missed ops
    if let Some(since) = client.last_hlc() {
        let _ = client.request_sync(since).await;

        // Receive and apply sync response
        if let Ok(Some(msg)) = client.recv().await {
            handle_server_message(&msg, db_path)?;
        }
    }

    Ok(())
}

/// Get the count of pending operations from the backend.
fn get_pending_ops_count<T: Transport>(backend: &SyncBackend<T>) -> usize {
    match backend {
        SyncBackend::Git { wal, .. } => wal.count().unwrap_or(0),
        SyncBackend::WebSocket { client, .. } => client.pending_ops_count().unwrap_or(0),
    }
}

/// Perform a sync operation based on the backend type (async version).
async fn perform_sync_async<T: Transport>(backend: &mut SyncBackend<T>) -> Result<usize> {
    match backend {
        SyncBackend::Git {
            worktree,
            wal,
            db_path,
        } => sync_git(worktree, wal, db_path),
        SyncBackend::WebSocket { client, db_path } => sync_websocket(client, db_path).await,
    }
}

/// Perform WebSocket sync: connect if needed, flush queue, request sync.
async fn sync_websocket<T: Transport>(client: &mut SyncClient<T>, db_path: &Path) -> Result<usize> {
    use wk_core::protocol::ServerMessage;
    use wk_core::Merge;

    // Connect if not already connected
    if !client.is_connected() {
        client
            .connect_with_retry()
            .await
            .map_err(|e| Error::Sync(format!("failed to connect: {}", e)))?;
    }

    // Save last_hlc BEFORE flushing - flush_queue updates last_hlc with
    // each op sent, so if we check after, new clients would request sync
    // instead of snapshot and miss earlier ops from other clients.
    let sync_since = client.last_hlc();

    // Flush the offline queue to the server
    let flushed = client
        .flush_queue()
        .await
        .map_err(|e| Error::Sync(format!("failed to flush queue: {}", e)))?;

    // Request sync from server based on HLC BEFORE flush
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
    let mut snapshot_data: Option<(Vec<wk_core::Issue>, Vec<(String, String)>)> = None;

    // Set a timeout for receiving sync response
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            match client.recv().await {
                Ok(Some(ServerMessage::SyncResponse { ops })) => {
                    received_ops.extend(ops.clone());
                    ops_received += ops.len();
                    break;
                }
                Ok(Some(ServerMessage::SnapshotResponse { issues, tags, .. })) => {
                    // Snapshot contains full issues and tags
                    ops_received += issues.len();
                    snapshot_data = Some((issues, tags));
                    break;
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
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    // Ignore timeout errors - we'll sync what we got
    let _ = timeout.await;

    // Apply received ops to cache
    if !received_ops.is_empty() {
        let mut db = wk_core::Database::open(db_path)
            .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

        received_ops.sort();
        db.apply_all(&received_ops)
            .map_err(|e| Error::Sync(format!("failed to apply ops: {}", e)))?;
    }

    // Apply snapshot data if received
    // Use INSERT OR REPLACE to update existing records from snapshots
    if let Some((issues, tags)) = snapshot_data {
        use rusqlite::params;

        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| Error::Sync(format!("failed to open database: {}", e)))?;

        for issue in &issues {
            conn.execute(
                "INSERT OR REPLACE INTO issues (id, type, title, status, created_at, updated_at,
                 last_status_hlc, last_title_hlc, last_type_hlc)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    issue.id,
                    issue.issue_type.as_str(),
                    issue.title,
                    issue.status.as_str(),
                    issue.created_at.to_rfc3339(),
                    issue.updated_at.to_rfc3339(),
                    issue.last_status_hlc.map(|h| h.to_string()),
                    issue.last_title_hlc.map(|h| h.to_string()),
                    issue.last_type_hlc.map(|h| h.to_string()),
                ],
            )
            .map_err(|e| Error::Sync(format!("failed to create issue: {}", e)))?;
        }

        for (issue_id, label) in &tags {
            conn.execute(
                "INSERT OR IGNORE INTO labels (issue_id, label) VALUES (?1, ?2)",
                params![issue_id, label],
            )
            .map_err(|e| Error::Sync(format!("failed to add label: {}", e)))?;
        }
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
