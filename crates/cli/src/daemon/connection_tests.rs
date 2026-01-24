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
