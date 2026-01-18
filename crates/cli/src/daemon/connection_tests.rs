// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn shared_state_initial_values() {
    let state = SharedConnectionState::new();
    assert_eq!(state.get(), STATE_DISCONNECTED);
    assert_eq!(state.attempt(), 0);
    assert!(!state.is_connected());
    assert!(!state.is_connecting());
}

#[test]
fn shared_state_transitions() {
    let state = SharedConnectionState::new();

    // Transition to connecting
    state.set(STATE_CONNECTING);
    state.set_attempt(1);
    assert_eq!(state.get(), STATE_CONNECTING);
    assert_eq!(state.attempt(), 1);
    assert!(!state.is_connected());
    assert!(state.is_connecting());

    // Transition to connected
    state.set(STATE_CONNECTED);
    state.set_attempt(0);
    assert_eq!(state.get(), STATE_CONNECTED);
    assert!(state.is_connected());
    assert!(!state.is_connecting());

    // Transition back to disconnected
    state.set(STATE_DISCONNECTED);
    assert_eq!(state.get(), STATE_DISCONNECTED);
    assert!(!state.is_connected());
}

#[test]
fn shared_state_status_string() {
    let state = SharedConnectionState::new();

    // Disconnected
    assert_eq!(state.status_string(), "disconnected");

    // Connecting without attempt count
    state.set(STATE_CONNECTING);
    assert_eq!(state.status_string(), "connecting");

    // Connecting with attempt count
    state.set_attempt(3);
    assert_eq!(state.status_string(), "connecting (attempt 3)");

    // Connected
    state.set(STATE_CONNECTED);
    assert_eq!(state.status_string(), "connected");
}

#[test]
fn connection_config_default() {
    let config = ConnectionConfig::default();
    assert_eq!(config.url, "ws://localhost:7890");
    assert_eq!(config.max_retries, 10);
    assert_eq!(config.max_delay_secs, 30);
    assert_eq!(config.initial_delay_ms, 100);
}

#[tokio::test]
async fn connection_manager_creates_channel() {
    let shared_state = Arc::new(SharedConnectionState::new());
    let config = ConnectionConfig::default();

    let (manager, mut event_rx) = ConnectionManager::new(config, shared_state);

    // Channel should be open
    assert!(manager.event_tx.capacity() > 0);

    // Receiver should not have any pending messages
    assert!(event_rx.try_recv().is_err());
}

#[tokio::test]
async fn cancellation_token_works() {
    let shared_state = Arc::new(SharedConnectionState::new());
    let config = ConnectionConfig {
        url: "ws://invalid.example.com:9999".to_string(),
        max_retries: 100, // High retry count
        max_delay_secs: 1,
        initial_delay_ms: 10,
    };

    let (manager, mut event_rx) = ConnectionManager::new(config, Arc::clone(&shared_state));
    let cancel_token = manager.cancel_token();

    // Start connection task
    manager.spawn_connect_task();

    // Give it time to start connecting
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Should be in connecting state
    assert!(shared_state.is_connecting());

    // Cancel the task
    cancel_token.cancel();

    // Wait for cancellation to take effect
    tokio::time::sleep(Duration::from_millis(100)).await;

    // State should be disconnected
    assert_eq!(shared_state.get(), STATE_DISCONNECTED);

    // Channel should be empty (no events sent on cancellation)
    assert!(event_rx.try_recv().is_err());
}
