# Wok

A collaborative, offline-first, AI-friendly issue tracker.

## Installation

### Homebrew (macOS)

```bash
brew install alfredjeanlab/tap/wok
```

### Linux / Manual

```bash
curl -fsSL https://github.com/alfredjeanlab/wok/releases/latest/download/install.sh | bash
```

## Setup

Initialize a tracker in your project:

```bash
wok init                                  # Initialize with auto-detected prefix
wok init --prefix myproj                  # Initialize with custom prefix
wok init --remote .                       # Enable git sync (orphan branch)
```

Install Claude Code hooks for AI integration:

```bash
wok hooks install
```

## Issue Tracking

Create, track, and manage work:

```bash
wok new task "My task"                    # Create a new task
wok new feature "New auth"                # Create a feature
wok list                                  # List all open issues
wok ready                                 # Show unblocked todo items
wok start <id>                            # Start working on an issue
wok done <id>                             # Mark issue as complete
wok show <id>                             # View full issue details
wok dep <id1> blocks <id2>                # Add dependency
wok label <id> priority:1                 # Add labels
```

For more details, run `wok help` or `wok help <command>`.

## Remotes

Choose a sync strategy when initializing:

**Local** - Solo work, issues stay on your machine:
```bash
wok init
```

**Single Project** - Track issues in your repo (git branch):
```bash
wok init --remote .
```

**Multiple Projects** - Track issues across multiple projects (separate git repo):
```bash
wok init --remote ~/tracker
```

**Fleet** - Central coordination for agents and automation:
```bash
wok init --remote ws://host:port
```

Data is stored in `.wok/issues.db` by default.

## Development

### Dependencies

Install required tools (macOS):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
brew install bats      # BATS test framework
```

### Building and Testing

Run `make` to see all available targets:

```bash
make install   # Build and install the wok binary
make check     # Run fmt, clippy, check, audit, test
make spec      # Run BATS specs
```

See individual README files in `tests/` for detailed documentation.

## License

MIT - Copyright (c) 2026 Alfred Jean LLC
