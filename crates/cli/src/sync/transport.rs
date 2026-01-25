// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Transport abstraction for WebSocket communication.
//!
//! Provides a trait-based transport layer that enables:
//! - Real WebSocket connections for production
//! - Mock transports for unit testing

use std::future::Future;
use std::pin::Pin;

use wk_core::protocol::{ClientMessage, ServerMessage};

/// Error type for transport operations.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// Connection failed.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Connection closed unexpectedly.
    #[error("connection closed")]
    ConnectionClosed,

    /// Send failed.
    #[error("send failed: {0}")]
    SendFailed(String),

    /// Receive failed.
    #[error("receive failed: {0}")]
    ReceiveFailed(String),

    /// Serialization/deserialization failed.
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// Result type for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Transport trait for WebSocket-like communication.
///
/// This trait abstracts over the actual transport mechanism, allowing
/// for easy testing with mock implementations.
pub trait Transport: Send + Sync {
    /// Connect to a remote server.
    fn connect(
        &mut self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = TransportResult<()>> + Send + '_>>;

    /// Disconnect from the server.
    fn disconnect(&mut self) -> Pin<Box<dyn Future<Output = TransportResult<()>> + Send + '_>>;

    /// Send a message to the server.
    fn send(
        &mut self,
        msg: ClientMessage,
    ) -> Pin<Box<dyn Future<Output = TransportResult<()>> + Send + '_>>;

    /// Receive a message from the server.
    ///
    /// Returns `None` if the connection is closed.
    fn recv(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = TransportResult<Option<ServerMessage>>> + Send + '_>>;

    /// Check if connected.
    fn is_connected(&self) -> bool;
}

/// WebSocket transport implementation using tokio-tungstenite.
pub struct WebSocketTransport {
    /// The WebSocket connection, if connected.
    ws: Option<WebSocketConnection>,
}

/// Internal WebSocket connection wrapper.
struct WebSocketConnection {
    sink: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::Message,
    >,
    stream: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport.
    pub fn new() -> Self {
        WebSocketTransport { ws: None }
    }
}

impl Default for WebSocketTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for WebSocketTransport {
    fn connect(
        &mut self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = TransportResult<()>> + Send + '_>> {
        let url = url.to_string();
        Box::pin(async move {
            use futures_util::StreamExt;

            let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
                .await
                .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

            let (sink, stream) = ws_stream.split();
            self.ws = Some(WebSocketConnection { sink, stream });
            Ok(())
        })
    }

    fn disconnect(&mut self) -> Pin<Box<dyn Future<Output = TransportResult<()>> + Send + '_>> {
        Box::pin(async move {
            if let Some(mut ws) = self.ws.take() {
                use futures_util::SinkExt;
                let _ = ws
                    .sink
                    .close()
                    .await
                    .map_err(|e| TransportError::SendFailed(e.to_string()));
            }
            Ok(())
        })
    }

    fn send(
        &mut self,
        msg: ClientMessage,
    ) -> Pin<Box<dyn Future<Output = TransportResult<()>> + Send + '_>> {
        Box::pin(async move {
            use futures_util::SinkExt;
            use tokio_tungstenite::tungstenite::Message;

            let ws = self.ws.as_mut().ok_or(TransportError::ConnectionClosed)?;

            let json = msg
                .to_json()
                .map_err(|e| TransportError::SerializationError(e.to_string()))?;

            if let Err(e) = ws.sink.send(Message::Text(json.into())).await {
                // Connection is broken, clear it
                self.ws = None;
                return Err(TransportError::SendFailed(e.to_string()));
            }

            // Flush to ensure the data is actually sent and we detect connection failures
            if let Err(e) = ws.sink.flush().await {
                self.ws = None;
                return Err(TransportError::SendFailed(e.to_string()));
            }

            Ok(())
        })
    }

    fn recv(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = TransportResult<Option<ServerMessage>>> + Send + '_>> {
        Box::pin(async move {
            use futures_util::StreamExt;
            use tokio_tungstenite::tungstenite::Message;

            let ws = self.ws.as_mut().ok_or(TransportError::ConnectionClosed)?;

            loop {
                match ws.stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let msg: ServerMessage = serde_json::from_str(&text)
                            .map_err(|e| TransportError::SerializationError(e.to_string()))?;
                        return Ok(Some(msg));
                    }
                    Some(Ok(Message::Close(_))) => {
                        // Connection closed, clear it
                        self.ws = None;
                        return Ok(None);
                    }
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                        // Ignore ping/pong, continue waiting
                        continue;
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types
                        continue;
                    }
                    Some(Err(e)) => {
                        // Connection is broken, clear it
                        self.ws = None;
                        return Err(TransportError::ReceiveFailed(e.to_string()));
                    }
                    None => {
                        // Stream ended, clear connection
                        self.ws = None;
                        return Ok(None);
                    }
                }
            }
        })
    }

    fn is_connected(&self) -> bool {
        self.ws.is_some()
    }
}
