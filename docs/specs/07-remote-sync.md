# Remote Sync

Optional synchronization between multiple clients via git or a central WebSocket server.

## Remote Types

wk supports three remote types:

| Type | URL Format | Use Case |
|------|------------|----------|
| Git (same repo) | `git:.` or `.` | Issue tracking in an orphan branch of your project |
| Git (separate repo) | `git:~/tracker` or `git:git@github.com:org/repo` | Shared tracker across multiple projects |
| WebSocket | `ws://` or `wss://` | Real-time sync with private server |

## Configuration

Remote sync is enabled by adding a `[remote]` section to `.wok/config.toml`:

### Git Remote (Same Repo)

```toml
# .wok/config.toml
prefix = "prj"

[remote]
url = "git:."           # Store oplog in orphan branch of current repo
branch = "wk/oplog"     # Optional, default: wk/oplog
```

### Git Remote (Separate Repo)

```toml
[remote]
url = "git:~/work-tracker"              # Local path
# or
url = "git:git@github.com:org/tracker"  # SSH URL
branch = "wk/oplog"
```

### WebSocket Server

```toml
[remote]
url = "wss://your-server:7890"
```

Without a `[remote]` section, the CLI operates in local-only mode.

## Quick Start

```bash
# Initialize with git remote (same repo)
wk init --remote .

# Initialize with git remote (separate repo)
wk init --remote git:~/tracker
wk init --remote git@github.com:org/tracker.git

# Initialize with WebSocket server
wk init --remote wss://your-server:7890

# Check remote status
wk remote status

# Sync with remote
wk remote sync
```

## Git Hooks

When using git remotes, wk can install hooks for automatic sync:

- **post-push**: Syncs after you push your code
- **post-merge**: Syncs after you pull/merge

Hooks are installed automatically during `wk init --remote .`. They ensure your issues stay in sync with your normal git workflow.

## Architecture

### Git Remote Architecture

```
           Project                              Remote
  ┌─────────────────────────┐           ┌─────────────────┐
  │  ┌─────┐      ┌──────┐  │           │  Git Repo       │
  │  │ wk  │─IPC─►│daemon│◄─┼───────────┼──► wk/oplog     │
  │  └──┬──┘      └──┬───┘  │  git      │     branch      │
  │     │            │      │  push/pull│                 │
  │     ▼            ▼      │           └─────────────────┘
  │     ┌────────────┐      │
  │     │  local DB  │      │
  │     └────────────┘      │
  └─────────────────────────┘
```

### Git Remote (Same Repo) Storage

When using `git:.` remote, the oplog is stored in a git worktree at:

```
project/
├── .git/
│   └── wk/
│       └── oplog/           # Git worktree
│           ├── .git         # Worktree link file
│           └── oplog.jsonl  # Operation log
├── .wok/
│   ├── config.toml
│   └── issues.db
└── ... (your project files)
```

This provides branch protection: git prevents deletion of the `wk/oplog` branch while the worktree exists.

### WebSocket Remote Architecture

```
           Client                                       Server
  ┌─────────────────────────┐                   ┌─────────────────┐
  │  ┌─────┐      ┌──────┐  │    WebSocket      │   wk-remote     │
  │  │ wk  │─IPC─►│daemon│◄─┼───────────────────┼──►              │
  │  └──┬──┘      └──┬───┘  │                   │    ┌─────────┐  │
  │     │            │      │                   │    │server DB│  │
  │     ▼            ▼      │                   │    └─────────┘  │
  │     ┌────────────┐      │                   │    ┌─────────┐  │
  │     │  local DB  │      │                   │    │git repo │  │
  │     └────────────┘      │                   │    └─────────┘  │
  └─────────────────────────┘                   └─────────────────┘
                                                         ▲
                                              other daemons connect here
```

- **wk CLI**: Reads/writes local DB directly; communicates with daemon via IPC
- **local DB**: Each client's `issues.db`; source of truth for local operations
- **daemon**: Syncs local DB with remote. One per local DB.
- **wk-remote**: Optional central server with git backing for durability

## Daemon Lifecycle

The daemon is a background process that manages remote synchronization.

**Commands:**
```bash
wk remote status   # Show daemon state: running/not running, connected/disconnected
wk remote stop     # Stop the daemon
wk remote sync     # Force immediate sync with remote
```

**Behavior:**
- Auto-spawns on first command that requires sync
- Single instance per local database (enforced via PID file + socket)
- For git remotes: performs fetch/merge/push operations
- For WebSocket remotes: maintains persistent connection
- Automatically reconnects on connection loss

**Files** (stored alongside `issues.db`):
- `daemon.pid` - Process ID
- `daemon.sock` - IPC socket for CLI communication

## Connection States

| State | Meaning |
|-------|---------|
| `connected` | Daemon has active connection to remote |
| `disconnected` | Connection lost, daemon attempting to reconnect |
| `not running` | No daemon process (will auto-spawn on next command) |

## What Gets Synced

All issue data syncs bidirectionally:
- Issues (id, type, title, status, timestamps)
- Dependencies (blocks, tracks relationships)
- Tags
- Notes
- Events (audit log)

## Offline Behavior

When remote is unreachable:
1. All commands continue to work locally
2. Operations queue in write-ahead log (WAL)
3. `wk remote status` shows pending operation count
4. When connection restored, queued ops sync automatically

## Multi-Client Sync

Changes propagate to all connected clients:
1. Client A creates/modifies issue
2. Daemon sends operation to remote
3. Remote stores and broadcasts to all clients
4. Other clients receive and apply change

## Conflict Resolution

**Last-write-wins** for same-field edits:
- If Client A and B both edit the same issue's title, the later timestamp wins
- Uses Hybrid Logical Clocks (HLC) for consistent ordering across clients

**Independent fields merge:**
- Client A edits title while Client B changes status - both changes apply
- Tags and notes accumulate (append-only)

## Version Handshake

When the CLI connects to a running daemon, it performs a version handshake:

1. CLI sends `Hello` request with its version
2. Daemon responds with its version
3. CLI compares versions and takes action:

| Scenario | Behavior |
|----------|----------|
| Versions match | Continue normally |
| CLI newer than daemon | Restart daemon automatically |
| Daemon newer than CLI | Warn but continue (backward compatible) |
| Old daemon (no Hello support) | Restart daemon automatically |

This prevents hangs when CLI binary is updated but an old daemon is still running.

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Invalid remote URL | `wk remote sync` fails with clear error message |
| Remote unreachable | Operations queue locally, sync when available |
| Malformed config | Parse error on any command that reads config |
| Daemon crash | Auto-respawns on next command, stale socket cleaned up |
| Daemon version mismatch | Auto-restarts daemon with matching version |
| Server crash | Operations queue locally, sync after server restart |

## Server (wk-remote)

The `wk-remote` binary runs a WebSocket sync server:

```bash
wk-remote --bind 127.0.0.1:7890 --data /path/to/server/data
```

**Options:**
- `--bind <addr:port>` - Address to listen on (default: 0.0.0.0:7890)
- `--data <path>` - Directory for server database (default: .)
- `--verbose` - Enable debug logging

### Git Backing

The server can optionally back its data to a git repository for durability:

```bash
wk-remote --bind 0.0.0.0:7890 --data /var/lib/wk \
  --git \
  --git-branch wk/oplog \
  --git-commit-interval 300 \
  --git-remote origin \
  --git-push-interval 3600
```

**Git Options:**
- `--git` - Enable git backing
- `--git-branch <name>` - Branch name for commits (default: wk/oplog)
- `--git-commit-interval <secs>` - Auto-commit interval (default: 300)
- `--git-remote <name>` - Remote name for pushing (enables push)
- `--git-push-interval <secs>` - Auto-push interval (default: 3600)

**Benefits:**
- **Durability**: Git provides append-only backup
- **Portability**: Export git repo to migrate away from server
- **Audit trail**: Full git history of all operations
- **Disaster recovery**: Rebuild from git if SQLite corrupts
