# Background Connection Fix

**Root Feature:** `wok-a5c1`

## Problem Summary

The daemon blocks IPC handling during WebSocket connection attempts:
1. Initial connection (up to 51 seconds with exponential backoff)
2. Reconnection attempts after disconnect
3. sync_websocket fallback when not connected

This causes:
- detect_daemon() timeout and socket file deletion
- Unresponsive status queries
- Blocked shutdown requests
- ~4 second test slowness (timeout + polling)

## Solution: Background Connection Task

Move all connection logic to a dedicated background task. Main loop starts immediately and handles IPC regardless of connection state.

## Design

### Shared State

```rust
/// Connection state visible to both background task and main loop.
struct SharedConnectionState {
    /// Current state (atomic for lock-free reads).
    state: AtomicU8,  // 0=Disconnected, 1=Connecting, 2=Connected
    /// Connection attempt count (for status reporting).
    attempt: AtomicU32,
}

// Constants for state values
const STATE_DISCONNECTED: u8 = 0;
const STATE_CONNECTING: u8 = 1;
const STATE_CONNECTED: u8 = 2;
```

### Channel-Based Transport Handoff

Background task establishes connection, sends it to main loop via oneshot channel:

```rust
enum ConnectionEvent {
    Connected(WebSocketConnection),
    Failed { attempts: u32, error: String },
    Disconnected,
}
```

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Daemon Process                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────────┐       ┌──────────────────────┐    │
│  │ Connection Task  │──────►│   Main Loop          │    │
│  │                  │ chan  │                      │    │
│  │ - connect_with_  │       │ - IPC handling       │    │
│  │   retry()        │       │ - WebSocket recv     │    │
│  │ - exponential    │       │ - State queries      │    │
│  │   backoff        │       │                      │    │
│  └──────────────────┘       └──────────────────────┘    │
│           │                          │                   │
│           ▼                          ▼                   │
│  ┌──────────────────────────────────────────────────┐   │
│  │         SharedConnectionState (Arc)               │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Shared State Infrastructure

**File: `crates/cli/src/daemon/connection.rs`** (new)

1. Define `SharedConnectionState` with atomic fields
2. Define `ConnectionEvent` enum
3. Implement `ConnectionManager` struct:
   - Holds config, shared state, event sender
   - `async fn run(&self)` - main connection loop
   - `fn request_connect(&self)` - trigger connection attempt
   - `fn request_disconnect(&self)` - trigger graceful disconnect

### Phase 2: Refactor Runner

**File: `crates/cli/src/daemon/runner.rs`**

1. Create shared state before main loop
2. Spawn connection task with `tokio::spawn`
3. Add connection event channel to `select!`:
   ```rust
   tokio::select! {
       // IPC - always responsive
       result = listener.accept() => { ... }

       // Connection events from background task
       Some(event) = connection_rx.recv() => {
           match event {
               ConnectionEvent::Connected(conn) => {
                   // Store connection, update state
               }
               ConnectionEvent::Disconnected => {
                   // Clear connection, trigger reconnect
               }
           }
       }

       // WebSocket recv - only when connected
       result = recv_from_ws(), if connected => { ... }
   }
   ```
4. Remove synchronous `connect_with_retry()` call at startup
5. Remove reconnection timer branch (handled by connection task)

### Phase 3: Fix sync_websocket Race

**File: `crates/cli/src/daemon/runner.rs`** (sync_websocket function)

Current code:
```rust
if !client.is_connected() {
    client.connect_with_retry().await?;  // BLOCKS, RACES
}
```

New approach:
```rust
// Check shared state
match shared_state.get() {
    STATE_CONNECTED => {
        // Proceed with sync
    }
    STATE_CONNECTING => {
        // Wait for connection or timeout
        // Don't start another connection attempt
    }
    STATE_DISCONNECTED => {
        // Request connection, wait for result
        connection_manager.request_connect();
        // Wait with timeout for Connected event
    }
}
```

### Phase 4: Update IPC Handlers

**File: `crates/cli/src/daemon/runner.rs`** (handle_ipc_request_async)

1. Status handler reads from `SharedConnectionState`:
   - Report "connecting (attempt 3/10)" during connection
   - Report "connected" / "disconnected" accurately

2. SyncNow handler:
   - If disconnected: trigger connection, wait with timeout
   - If connecting: wait for connection with timeout
   - If connected: proceed with sync

3. Shutdown handler:
   - Signal connection task to stop
   - Wait for clean termination

### Phase 5: Cancellation Support

1. Add `CancellationToken` from tokio-util
2. Pass to connection task
3. Check token in retry loop:
   ```rust
   loop {
       tokio::select! {
           _ = token.cancelled() => return,
           result = transport.connect(&url) => { ... }
       }

       tokio::select! {
           _ = token.cancelled() => return,
           _ = tokio::time::sleep(delay) => {}
       }
   }
   ```
4. Cancel token on shutdown request

## Issues Resolved

| Issue | Resolution |
|-------|------------|
| IPC blocked during initial connection | Main loop starts immediately |
| IPC blocked during reconnection | Connection runs in background |
| detect_daemon timeout | Daemon responds to Ping immediately |
| Socket file deletion | No timeout, no deletion |
| Race: multiple connect attempts | Single connection task, state check |
| Transport state corruption | Only connection task touches transport |
| Shutdown during connection | Cancellation token |
| Status during connection | Accurate state reporting |

## Issues NOT Addressed (Acceptable)

| Issue | Rationale |
|-------|-----------|
| Connection drops between state check and use | Normal network error handling suffices |
| Ordering of ops during reconnect | Existing queue handles this |

## Testing

1. **Unit tests** for `SharedConnectionState` transitions
2. **Integration test**: Verify IPC works during connection attempts
3. **Spec test**: Reduce timeout, verify test runs faster
4. **Stress test**: Rapid connect/disconnect cycles

## Migration

- No config changes
- No CLI changes
- No protocol changes
- Backward compatible (daemon behavior improves, doesn't change semantics)

## Files Changed

| File | Change |
|------|--------|
| `crates/cli/src/daemon/mod.rs` | Add `mod connection` |
| `crates/cli/src/daemon/connection.rs` | New: connection manager |
| `crates/cli/src/daemon/runner.rs` | Refactor main loop |
| `crates/cli/Cargo.toml` | Add `tokio-util` for CancellationToken |
