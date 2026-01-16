// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for the transport module.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::transport::{Transport, TransportError, TransportResult};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use wk_core::protocol::{ClientMessage, ServerMessage};

/// Mock transport for testing without real sockets.
pub struct MockTransport {
    connected: bool,
    /// Messages that will be returned by recv().
    incoming: Arc<Mutex<VecDeque<ServerMessage>>>,
    /// Messages that were sent via send().
    outgoing: Arc<Mutex<Vec<ClientMessage>>>,
    /// Whether the next connect should fail.
    connect_should_fail: bool,
}

impl MockTransport {
    pub fn new() -> Self {
        MockTransport {
            connected: false,
            incoming: Arc::new(Mutex::new(VecDeque::new())),
            outgoing: Arc::new(Mutex::new(Vec::new())),
            connect_should_fail: false,
        }
    }

    /// Add a message that will be returned by recv().
    pub fn queue_incoming(&self, msg: ServerMessage) {
        self.incoming.lock().unwrap().push_back(msg);
    }

    /// Get all messages that were sent.
    pub fn get_outgoing(&self) -> Vec<ClientMessage> {
        self.outgoing.lock().unwrap().clone()
    }

    /// Set whether connect should fail.
    pub fn set_connect_fail(&mut self, fail: bool) {
        self.connect_should_fail = fail;
    }
}

impl Transport for MockTransport {
    fn connect(
        &mut self,
        _url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TransportResult<()>> + Send + '_>> {
        Box::pin(async move {
            if self.connect_should_fail {
                Err(TransportError::ConnectionFailed("mock failure".into()))
            } else {
                self.connected = true;
                Ok(())
            }
        })
    }

    fn disconnect(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TransportResult<()>> + Send + '_>> {
        Box::pin(async move {
            self.connected = false;
            Ok(())
        })
    }

    fn send(
        &mut self,
        msg: ClientMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TransportResult<()>> + Send + '_>> {
        let outgoing = Arc::clone(&self.outgoing);
        Box::pin(async move {
            outgoing.lock().unwrap().push(msg);
            Ok(())
        })
    }

    fn recv(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = TransportResult<Option<ServerMessage>>> + Send + '_>,
    > {
        let incoming = Arc::clone(&self.incoming);
        Box::pin(async move {
            let msg = incoming.lock().unwrap().pop_front();
            Ok(msg)
        })
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[tokio::test]
async fn test_mock_transport_connect() {
    let mut transport = MockTransport::new();
    assert!(!transport.is_connected());

    transport.connect("ws://localhost:1234").await.unwrap();
    assert!(transport.is_connected());

    transport.disconnect().await.unwrap();
    assert!(!transport.is_connected());
}

#[tokio::test]
async fn test_mock_transport_send_recv() {
    let mut transport = MockTransport::new();
    transport.connect("ws://localhost:1234").await.unwrap();

    // Send a message
    let msg = ClientMessage::ping(42);
    transport.send(msg).await.unwrap();

    // Check it was recorded
    let outgoing = transport.get_outgoing();
    assert_eq!(outgoing.len(), 1);
    assert!(matches!(outgoing[0], ClientMessage::Ping { id: 42 }));

    // Queue an incoming message
    transport.queue_incoming(ServerMessage::pong(42));

    // Receive it
    let received = transport.recv().await.unwrap();
    assert!(matches!(received, Some(ServerMessage::Pong { id: 42 })));

    // No more messages
    let received = transport.recv().await.unwrap();
    assert!(received.is_none());
}

#[tokio::test]
async fn test_mock_transport_connect_fail() {
    let mut transport = MockTransport::new();
    transport.set_connect_fail(true);

    let result = transport.connect("ws://localhost:1234").await;
    assert!(result.is_err());
    assert!(!transport.is_connected());
}
