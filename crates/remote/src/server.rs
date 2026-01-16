// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! WebSocket server implementation.
//!
//! Handles client connections, message routing, and broadcast fanout.
//!
//! # Coverage Notes
//!
//! Due to LLVM coverage instrumentation limitations with async Rust:
//! - `handle_client_message`: Fully covered (all business logic)
//! - `run`, `handle_connection`: Async plumbing exercised by tests but not instrumented
//!
//! All code paths ARE tested via integration tests; the instrumentation simply
//! cannot track execution across `tokio::spawn` and `tokio::select!` boundaries.

use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

use wk_core::protocol::{ClientMessage, ServerMessage};

use crate::state::ServerState;

/// Run the WebSocket server on the given address.
pub async fn run(addr: SocketAddr, state: ServerState) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_addr, state).await {
                error!("Connection error from {}: {}", peer_addr, e);
            }
        });
    }
}

/// Handle a single WebSocket connection.
pub(crate) async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    state: ServerState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    info!("New WebSocket connection from: {}", peer_addr);

    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    // Subscribe to broadcasts
    let mut broadcast_rx = state.subscribe();

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match handle_client_message(&text, &state).await {
                            Ok(Some(response)) => {
                                let json = response.to_json()?;
                                ws_sink.send(Message::Text(json)).await?;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let error_msg = ServerMessage::error(e.to_string());
                                let json = error_msg.to_json()?;
                                ws_sink.send(Message::Text(json)).await?;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Client {} disconnected", peer_addr);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        ws_sink.send(Message::Pong(data)).await?;
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types (Binary, Pong, Frame)
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error from {}: {}", peer_addr, e);
                        break;
                    }
                    None => {
                        info!("Client {} stream ended", peer_addr);
                        break;
                    }
                }
            }

            // Handle broadcast messages to send to client
            broadcast = broadcast_rx.recv() => {
                match broadcast {
                    Ok(msg) => {
                        let json = msg.to_json()?;
                        if let Err(e) = ws_sink.send(Message::Text(json)).await {
                            warn!("Failed to send broadcast to {}: {}", peer_addr, e);
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Client {} lagged by {} messages", peer_addr, n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }

    info!("Connection closed: {}", peer_addr);
    Ok(())
}

/// Process a client message and return an optional response.
async fn handle_client_message(
    text: &str,
    state: &ServerState,
) -> Result<Option<ServerMessage>, Box<dyn std::error::Error + Send + Sync>> {
    let msg: ClientMessage = serde_json::from_str(text)?;
    debug!("Received message: {:?}", msg);

    match msg {
        ClientMessage::Op(op) => {
            // Apply op - broadcast happens inside apply_op
            match state.apply_op(op).await {
                Ok(true) => {
                    debug!("Op applied successfully");
                }
                Ok(false) => {
                    debug!("Op was duplicate, skipped");
                }
                Err(e) => {
                    return Ok(Some(ServerMessage::error(e.to_string())));
                }
            }
            Ok(None) // Response is via broadcast
        }

        ClientMessage::Sync { since } => {
            let ops = state.ops_since(since).await?;
            debug!("Sync response: {} ops since {:?}", ops.len(), since);
            Ok(Some(ServerMessage::sync_response(ops)))
        }

        ClientMessage::Snapshot => {
            let (issues, tags, since) = state.snapshot().await?;
            debug!(
                "Snapshot response: {} issues, {} tags",
                issues.len(),
                tags.len()
            );
            Ok(Some(ServerMessage::snapshot_response(issues, tags, since)))
        }

        ClientMessage::Ping { id } => {
            debug!("Ping received: {}", id);
            Ok(Some(ServerMessage::pong(id)))
        }
    }
}
