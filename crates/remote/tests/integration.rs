// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Integration tests for wk-remote server binary.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Helper to spawn a server process and clean it up on drop.
struct ServerProcess {
    child: Child,
    port: u16,
    _temp_dir: tempfile::TempDir,
}

impl ServerProcess {
    fn spawn() -> Self {
        let temp_dir = tempfile::tempdir().expect("create temp dir");

        // Use a port range that's less likely to conflict
        // Starting from a high ephemeral port
        let port = 49152 + (std::process::id() % 1000) as u16;

        let child = Command::new(env!("CARGO_BIN_EXE_wk-remote"))
            .arg("--bind")
            .arg(format!("127.0.0.1:{}", port))
            .arg("--data")
            .arg(temp_dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn server process");

        ServerProcess {
            child,
            port,
            _temp_dir: temp_dir,
        }
    }

    fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        // Kill the server process
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tokio::test]
async fn test_server_lifecycle() {
    let server = ServerProcess::spawn();

    // Wait for server to start and retry connection a few times
    // CI runners can be slow, so we use generous timeouts
    let mut ws_stream = None;
    for _ in 0..20 {
        if let Ok(Ok((stream, _))) =
            tokio::time::timeout(Duration::from_millis(500), connect_async(&server.ws_url())).await
        {
            ws_stream = Some(stream);
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let ws_stream = ws_stream.expect("should connect to server within retries");

    // Connection successful
    let (mut sink, mut stream) = ws_stream.split();

    // Send a ping (format: {"type": "ping", "id": 12345})
    let ping_msg = serde_json::json!({"type": "ping", "id": 12345});
    sink.send(Message::Text(ping_msg.to_string().into()))
        .await
        .expect("send ping");

    // Wait for pong
    let response = tokio::time::timeout(Duration::from_secs(5), stream.next()).await;
    match response {
        Ok(Some(Ok(Message::Text(text)))) => {
            // Response format: {"type":"pong","id":12345}
            assert!(
                text.contains("12345"),
                "Expected pong with id 12345, got: {}",
                text
            );
        }
        other => panic!("Expected pong, got {:?}", other),
    }
    // Server process is automatically killed when dropped
}
