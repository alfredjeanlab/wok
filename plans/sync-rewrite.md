# Sync Rewrite Plan

## Overview

Rewrite wok's synchronization model to eliminate remote/WebSocket/git sync entirely and replace it with a simpler two-mode architecture:

1. **User-level mode** (default) — Single `issues.db` at `~/.local/state/wok/`, shared across all projects. A user-level daemon (`wokd`) owns the database and handles concurrent access via IPC.
2. **Private mode** (`--private`) — Project-local `./issues.db` with direct SQLite access, no daemon. Renamed from the current "local" mode.

The remote sync server (`crates/remote/`) and all WebSocket/git sync machinery are removed. The daemon's sole purpose becomes **database ownership** — serializing concurrent CLI access to the shared SQLite database, not syncing with a remote server.

## Project Structure

After the rewrite:

```
wok/
├── crates/
│   ├── cli/              # `wok` binary (CLI)
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs
│   │   │   ├── cli.rs           # Clap definitions (updated)
│   │   │   ├── config.rs        # Config (simplified, no RemoteConfig)
│   │   │   ├── mode.rs          # OperatingMode: Private | UserLevel
│   │   │   ├── commands/
│   │   │   │   ├── mod.rs       # open_db() routes by mode
│   │   │   │   ├── init.rs      # `wok init` (updated)
│   │   │   │   ├── daemon.rs    # `wok daemon {status,stop,start,logs}`
│   │   │   │   ├── ...          # (other commands unchanged)
│   │   │   │   └── (remote.rs DELETED)
│   │   │   ├── daemon/
│   │   │   │   ├── mod.rs       # Public API: connect_or_start, detect, stop
│   │   │   │   ├── client.rs    # CLI-side IPC client (connect, send, recv)
│   │   │   │   ├── lifecycle.rs # spawn, detect, cleanup, version handshake
│   │   │   │   └── ipc.rs       # Request/Response enums, framing
│   │   │   │   └── (runner.rs DELETED — moved to wokd)
│   │   │   │   └── (connection.rs DELETED)
│   │   │   │   └── (cache.rs DELETED)
│   │   │   │   └── (sync.rs DELETED)
│   │   │   ├── (sync/ DELETED entirely)
│   │   │   ├── (worktree.rs DELETED)
│   │   │   ├── (wal.rs DELETED)
│   │   │   └── (git_hooks.rs DELETED)
│   │   └── Cargo.toml           # Remove tokio-tungstenite, simplify deps
│   ├── core/             # Shared library (largely unchanged)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db.rs            # Core database (unchanged)
│   │   │   ├── issue.rs         # Issue types (unchanged)
│   │   │   ├── hlc.rs           # HLC (unchanged)
│   │   │   ├── op.rs            # Op types (unchanged)
│   │   │   ├── merge.rs         # Merge semantics (unchanged)
│   │   │   ├── (protocol.rs DELETED — WebSocket messages)
│   │   │   ├── (oplog.rs DELETED — no remote sync)
│   │   │   └── identity.rs      # (keep or delete based on usage)
│   │   └── Cargo.toml
│   ├── daemon/           # `wokd` binary (NEW crate)
│   │   ├── src/
│   │   │   ├── main.rs          # Entry point, arg parsing
│   │   │   ├── server.rs        # Event loop: IPC listener + request dispatch
│   │   │   ├── ipc.rs           # Re-export/use shared IPC types
│   │   │   └── state.rs         # DaemonState: Database handle, HLC clock
│   │   └── Cargo.toml           # tokio, wk-core, rusqlite, fs2
│   └── (remote/ DELETED entirely)
├── tests/specs/
├── docs/specs/
└── plans/
```

### Key structural changes

- **New crate:** `crates/daemon/` produces the `wokd` binary
- **Deleted crate:** `crates/remote/` (WebSocket relay server)
- **Deleted modules in cli:** `sync/`, `worktree.rs`, `wal.rs`, `git_hooks.rs`, `daemon/runner.rs`, `daemon/connection.rs`, `daemon/cache.rs`, `daemon/sync.rs`, `commands/remote.rs`
- **Deleted modules in core:** `protocol.rs`, `oplog.rs`

## Dependencies

### New crate `crates/daemon/` (`wokd`)

```toml
[dependencies]
wk-core = { path = "../core" }
tokio = { version = "1", features = ["rt", "net", "signal", "time", "macros", "io-util"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
fs2 = "0.4"
dirs = "6"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

### Removed dependencies from `crates/cli/`

- `tokio-tungstenite` (WebSocket client)
- `futures-util` (async stream combinators)
- `tokio-util` (if only used for sync)
- Reduce tokio features (only need `rt` for auto-start spawn, possibly remove entirely if spawn is sync)

### Removed crate

- `crates/remote/` removed from workspace members

## Implementation Phases

### Phase 1: Introduce `crates/daemon/` with `wokd` binary

**Goal:** Create the daemon crate with IPC server, database ownership, and the full request/response protocol. The daemon doesn't need to do anything the CLI can't already do — it just serializes access.

**Steps:**

1. Create `crates/daemon/` with `Cargo.toml`, add to workspace members
2. Define shared IPC types (or re-export from a shared location):
   - Move `DaemonRequest`/`DaemonResponse` to `wk-core` (or a new `wk-ipc` module in core) so both `wok` and `wokd` can use them
   - Extend the protocol for database operations:

```rust
// Shared IPC protocol (in wk-core or kept in cli with daemon depending on it)
enum DaemonRequest {
    Ping,
    Hello { version: String },
    Status,
    Shutdown,
    // Database operations — CLI sends these, daemon executes against SQLite
    Query { sql_tag: QueryTag },
    Mutate { op: Op },
}

enum QueryTag {
    ListIssues { filters: IssueFilters },
    GetIssue { id: String },
    SearchIssues { query: String, filters: IssueFilters },
    GetEvents { issue_id: Option<String>, limit: Option<usize> },
    // ... one variant per read operation the CLI needs
}

enum DaemonResponse {
    Pong,
    Hello { version: String },
    Status(DaemonStatus),
    ShuttingDown,
    Error { message: String },
    Issues { issues: Vec<Issue> },
    Issue { issue: Option<Issue> },
    Events { events: Vec<Event> },
    Mutated { id: String },
    // ... one variant per response type
}
```

3. Implement `wokd` main:
   - Parse args: `wokd [--foreground] [--state-dir PATH]`
   - State dir defaults to `$WOK_STATE_DIR` or `$XDG_STATE_HOME/wok` or `~/.local/state/wok/`
   - Startup sequence (modeled on otterjobs):
     1. Write log marker `--- wokd: starting (pid: <pid>)`
     2. Acquire flock on `daemon.lock` (non-blocking `try_lock_exclusive`)
     3. Create dirs, write `daemon.version` and `daemon.pid`
     4. Open SQLite database at `<state_dir>/issues.db`
     5. Bind Unix socket at `<state_dir>/daemon.sock`
     6. Print `READY` to stdout
     7. Enter event loop

4. Implement the event loop in `server.rs`:
   - `tokio::select!` over: socket accept, SIGTERM/SIGINT, idle timer
   - Each accepted connection: read request, dispatch, write response
   - Database handle wrapped in state struct (single-threaded access via event loop)

5. Implement shutdown sequence:
   1. Stop accepting connections
   2. Remove socket, PID, version files
   3. Release flock
   4. Exit

**Milestone:** `wokd` can start, accept IPC connections, respond to Ping/Hello/Status/Shutdown, and serve database queries.

### Phase 2: Update `wok init` and operating mode

**Goal:** Change `wok init` to default to user-level mode. Add `--private` flag. Remove `--remote`/`--local` flags.

**Steps:**

1. Update `OperatingMode` enum:

```rust
pub enum OperatingMode {
    /// Private mode: direct SQLite at ./.wok/issues.db, no daemon
    Private,
    /// User-level mode: daemon at ~/.local/state/wok/, IPC for all operations
    UserLevel,
}
```

2. Update mode detection:
   - If `.wok/config.toml` contains `private = true` → `Private`
   - Otherwise → `UserLevel`

3. Update `Config` struct:
   - Remove `remote: Option<RemoteConfig>` field
   - Remove `workspace: Option<String>` field (user-level mode uses XDG, private mode uses `.wok/`)
   - Add `private: bool` field (default `false`)

```rust
pub struct Config {
    pub prefix: String,
    #[serde(default)]
    pub private: bool,
}
```

4. Update `wok init`:
   - Remove `--remote`, `--local`, `--workspace` flags
   - Add `--private` flag
   - Default behavior: create `.wok/config.toml` with prefix only (user-level mode)
   - `--private`: set `private = true` in config, create `.wok/issues.db`
   - Remove git hook installation, worktree setup

5. Update `get_db_path()`:
   - Private mode: `.wok/issues.db`
   - User-level mode: `~/.local/state/wok/issues.db` (via `dirs::state_dir()` or manual XDG)

6. Update `.gitignore` generation:
   - Private mode: ignore `issues.db`, `config.toml`
   - User-level mode: ignore `config.toml` (no local db to ignore)

**Milestone:** `wok init` creates projects in user-level mode by default. `wok init --private` creates private-mode projects.

### Phase 3: Route CLI commands through daemon in user-level mode

**Goal:** In user-level mode, all database operations go through the daemon via IPC. In private mode, they use direct SQLite (existing behavior).

**Steps:**

1. Update `commands/mod.rs::open_db()` to branch on mode:

```rust
pub fn open_db() -> Result<DatabaseHandle> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let mode = OperatingMode::detect(&config);

    match mode {
        OperatingMode::Private => {
            let db_path = work_dir.join("issues.db");
            Ok(DatabaseHandle::Direct(Database::open(&db_path)?))
        }
        OperatingMode::UserLevel => {
            let state_dir = wok_state_dir();
            let conn = connect_or_start_daemon(&state_dir)?;
            Ok(DatabaseHandle::Daemon(conn))
        }
    }
}
```

2. Introduce a `DatabaseHandle` abstraction (or trait) that command implementations use:
   - Option A (trait): `trait DbOps` with methods like `list_issues()`, `create_issue()`, etc. Two implementations: `DirectDb` (SQLite) and `DaemonDb` (IPC)
   - Option B (enum): `enum DatabaseHandle { Direct(Database), Daemon(DaemonConnection) }` with methods that dispatch

   **Recommended: Option B** — simpler, avoids trait object overhead, matches existing pattern.

3. Update `daemon/lifecycle.rs` for the new daemon binary:
   - `spawn_daemon()` now invokes `wokd` (separate binary) instead of `wok remote run`
   - Auto-detect `wokd` binary path: same directory as `wok` binary, or `WOK_DAEMON_BINARY` env var
   - Keep existing connection protocol: check socket → connect → version handshake → auto-start

4. Implement `connect_or_start_daemon()` in CLI (modeled on otterjobs):

```
1. Check socket exists at ~/.local/state/wok/daemon.sock
2. If no socket → start daemon, poll for ready
3. If socket exists but can't connect → stale socket cleanup → start daemon
4. If connected → version handshake (Hello)
5. If version mismatch → stop old daemon → start new daemon
6. Return connected stream
```

5. Update each command to use `DatabaseHandle` instead of raw `Database`:
   - Commands that only read: `list`, `show`, `search`, `log`, `tree`, `ready`, `export`
   - Commands that mutate: `new`, `start`, `done`, `close`, `reopen`, `edit`, `label`, `unlabel`, `note`, `dep`, `undep`, `link`, `unlink`, `import`

**Milestone:** All commands work in both private mode (direct) and user-level mode (via daemon). Running `wok list` in a user-level project auto-starts `wokd` and queries via IPC.

### Phase 4: Harden the IPC protocol

**Goal:** Add robustness features modeled on the otterjobs daemon protocol.

**Steps:**

1. **Version negotiation:** Already exists (`Hello` handshake). Ensure `wokd` writes `daemon.version` file on startup so CLI can detect mismatches without connecting.

2. **Lock file with PID:**
   - `daemon.lock` — flock-based, non-blocking `try_lock_exclusive()`
   - `daemon.pid` — written after lock acquired, contains PID
   - On startup, if lock held → exit immediately (another daemon is running)

3. **Stale socket cleanup:**
   - If socket exists but connect fails:
     1. Read PID from `daemon.pid`
     2. Check if process alive via `kill(pid, 0)`
     3. If dead → remove socket, PID, lock files → start fresh
   - Already partially implemented; make it robust

4. **Startup error reporting:**
   - `wokd` writes `--- wokd: starting (pid: <pid>)` marker to `daemon.log`
   - On startup failure, errors go to `daemon.log`
   - CLI reads log from last marker on timeout, extracts ERROR lines
   - Display specific error to user instead of generic "connection timeout"

5. **Daemon logging:**
   - `wokd` logs to `~/.local/state/wok/daemon.log`
   - Use `tracing` with file appender
   - CLI command `wok daemon logs` reads this file
   - CLI command `wok daemon logs -f` tails the file

6. **Auto-start:** Already planned in Phase 3. Ensure it's seamless — user never thinks about the daemon.

7. **Graceful shutdown:**
   - `wok daemon stop` sends `Shutdown` via IPC
   - `wokd` breaks event loop, cleans up files, exits
   - If graceful fails, CLI sends SIGTERM, then SIGKILL after timeout
   - Shutdown sequence: stop accepting → remove socket → remove PID/version → release lock → exit

**Milestone:** Daemon lifecycle is robust against crashes, stale files, version mismatches, and concurrent starts.

### Phase 5: Remove remote/sync infrastructure

**Goal:** Delete all remote sync code and the `crates/remote/` crate.

**Steps:**

1. Delete `crates/remote/` entirely
2. Remove `"crates/remote"` from workspace `Cargo.toml` members
3. Delete from `crates/cli/`:
   - `src/sync/` directory (WebSocket client, offline queue, transport)
   - `src/worktree.rs` (git oplog worktree)
   - `src/wal.rs` (pending ops WAL for git sync)
   - `src/git_hooks.rs` (post-push/post-merge hooks)
   - `src/daemon/runner.rs` (daemon main loop — replaced by `wokd`)
   - `src/daemon/connection.rs` (WebSocket connection manager)
   - `src/daemon/cache.rs` (server message handler)
   - `src/daemon/sync.rs` (sync operations)
   - `src/commands/remote.rs` (remote subcommands)
4. Delete from `crates/core/`:
   - `src/protocol.rs` (WebSocket message types)
   - `src/oplog.rs` (operation log for sync)
5. Remove from `crates/cli/src/lib.rs`:
   - `mod sync;`, `mod worktree;`, `mod wal;`, `mod git_hooks;`
   - `RemoteCommand` from CLI enum and `run()` match
6. Remove from `crates/cli/Cargo.toml`:
   - `tokio-tungstenite`
   - `futures-util`
   - Reduce `tokio` features if possible
7. Update `crates/cli/src/commands/mod.rs`:
   - Remove `write_pending_op()`, `queue_op()` (sync queue helpers)
   - Simplify `apply_mutation()` to just log event + apply to db
   - Remove `OfflineQueue` imports
8. Add `wok daemon` subcommand to CLI (replacing `wok remote`):
   - `wok daemon status` — show daemon status
   - `wok daemon stop` — stop daemon
   - `wok daemon start` — explicitly start daemon
   - `wok daemon logs [-f]` — view daemon logs

**Milestone:** No sync code remains. `crates/remote/` is gone. CLI only has IPC client code for talking to `wokd`.

### Phase 6: Update specs and docs

**Goal:** Update all tests, specs, and documentation to reflect the new architecture.

**Steps:**

1. Update `docs/specs/`:
   - Remove remote sync specs
   - Add user-level mode specs
   - Add daemon lifecycle specs
   - Update init specs for `--private`

2. Update `tests/specs/`:
   - Remove remote-related BATS tests
   - Add tests for `wok init` (default = user-level)
   - Add tests for `wok init --private`
   - Add tests for `wok daemon {status,stop,start}`
   - Add tests for auto-start behavior
   - Add tests for private vs user-level database routing

3. Update CLAUDE.md files if needed

4. Run `make validate` to confirm everything passes

**Milestone:** All specs pass. Documentation reflects the new architecture.

## Key Implementation Details

### Database handle abstraction

The central design decision is how CLI commands access the database. In private mode, they open SQLite directly. In user-level mode, they must go through the daemon.

```rust
pub enum DatabaseHandle {
    Direct {
        db: Database,
        config: Config,
        work_dir: PathBuf,
    },
    Daemon {
        conn: DaemonConnection,
        config: Config,
        work_dir: PathBuf,
    },
}

impl DatabaseHandle {
    pub fn list_issues(&self, filters: &IssueFilters) -> Result<Vec<Issue>> {
        match self {
            Self::Direct { db, .. } => db.list_issues(filters),
            Self::Daemon { conn, .. } => {
                let resp = conn.request(DaemonRequest::Query {
                    sql_tag: QueryTag::ListIssues { filters: filters.clone() },
                })?;
                match resp {
                    DaemonResponse::Issues { issues } => Ok(issues),
                    DaemonResponse::Error { message } => Err(Error::Daemon(message)),
                    _ => Err(Error::Daemon("unexpected response".into())),
                }
            }
        }
    }

    pub fn apply_op(&self, op: OpPayload) -> Result<String> {
        match self {
            Self::Direct { db, config, work_dir } => {
                // Direct SQLite mutation (existing code path)
                // ...
            }
            Self::Daemon { conn, .. } => {
                let hlc = generate_hlc();
                let op = Op::new(hlc, op);
                let resp = conn.request(DaemonRequest::Mutate { op })?;
                // ...
            }
        }
    }
}
```

### IPC message design

Keep the existing length-prefixed JSON framing (4-byte big-endian length + JSON). It's simple, debuggable, and fast enough for local IPC.

The `QueryTag` enum should mirror the existing `Database` method signatures rather than exposing raw SQL. This keeps the protocol stable and type-safe.

### XDG state directory resolution

```rust
fn wok_state_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("WOK_STATE_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(dir).join("wok");
    }
    dirs::home_dir()
        .map(|h| h.join(".local/state/wok"))
        .expect("no home directory")
}
```

State directory layout:
```
~/.local/state/wok/
├── daemon.sock          # Unix socket for IPC
├── daemon.pid           # PID file
├── daemon.lock          # flock file
├── daemon.version       # Binary version for mismatch detection
├── daemon.log           # Daemon log output
└── issues.db            # Shared SQLite database
```

### Daemon auto-detection of `wokd` binary

```rust
fn find_wokd_binary() -> Result<PathBuf> {
    // 1. Check WOK_DAEMON_BINARY env var
    if let Ok(path) = std::env::var("WOK_DAEMON_BINARY") {
        return Ok(PathBuf::from(path));
    }

    // 2. Look next to the current executable
    if let Ok(exe) = std::env::current_exe() {
        let wokd = exe.with_file_name("wokd");
        if wokd.exists() {
            return Ok(wokd);
        }
    }

    // 3. Fall back to PATH
    Ok(PathBuf::from("wokd"))
}
```

### Prefix handling in user-level mode

In user-level mode, multiple projects share one database. Each project has its own prefix (from `.wok/config.toml`). The CLI passes the prefix with mutation requests so the daemon can scope issue IDs correctly. The daemon database stores issues from all projects — the prefix in the issue ID is sufficient to distinguish them.

### HLC clock in the daemon

The daemon owns the HLC clock for user-level mode. This prevents clock skew between concurrent CLI invocations. The daemon maintains a single `HlcClock` instance in its state and generates timestamps for all mutations.

In private mode, the CLI generates HLCs directly (existing behavior, since there's no concurrency concern with direct SQLite access using WAL mode).

## Verification Plan

### Unit tests

- `crates/daemon/`: IPC request handling, database operations, startup/shutdown
- `crates/cli/`: `DatabaseHandle` dispatch, mode detection, daemon lifecycle
- `crates/core/`: Unchanged tests still pass

### Integration tests (BATS specs)

1. **Init behavior:**
   - `wok init proj` creates user-level config (no `private = true`)
   - `wok init --private proj` creates private config with local db
   - `wok init` derives prefix from directory name (existing behavior)

2. **Private mode (regression):**
   - All existing commands work with `--private` (direct SQLite, no daemon)
   - This should be nearly identical to current "local" mode behavior

3. **User-level mode:**
   - `wok new bug "test"` in user-level project auto-starts daemon
   - `wok list` returns issues created via daemon
   - Multiple projects with different prefixes share one database
   - `wok daemon status` shows running daemon info
   - `wok daemon stop` stops the daemon

4. **Daemon lifecycle:**
   - Auto-start on first command
   - Version mismatch triggers restart
   - Stale socket cleanup after crash (kill daemon, verify next command recovers)
   - `wok daemon stop` → next command auto-starts fresh

5. **Edge cases:**
   - Running `wok` commands concurrently in user-level mode
   - Switching between private and user-level projects in same shell
   - `WOK_STATE_DIR` override for test isolation

### Manual verification

- `make check` passes (fmt, clippy, audit, build, test)
- `make spec` passes (all BATS specs)
- No dead code warnings
- No remaining references to `remote`, `WebSocket`, `worktree`, or `oplog` in non-test code
