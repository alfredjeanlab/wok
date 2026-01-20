# Wok (wk)

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
wk init                                  # Initialize with auto-detected prefix
wk init --prefix myproj                  # Initialize with custom prefix
wk init --remote .                       # Enable git sync (orphan branch)
```

Install Claude Code hooks for AI integration:

```bash
wk hooks install
```

## Issue Tracking

Create, track, and manage work:

```bash
wk new task "My task"                    # Create a new task
wk new feature "New auth"                # Create a feature
wk list                                  # List all open issues
wk ready                                 # Show unblocked todo items
wk start <id>                            # Start working on an issue
wk done <id>                             # Mark issue as complete
wk show <id>                             # View full issue details
wk dep <id1> blocks <id2>                # Add dependency
wk label <id> priority:1                 # Add labels
```

For more details, run `wk help` or `wk help <command>`.

## Remotes

Choose a sync strategy when initializing:

**Local** - Solo work, issues stay on your machine:
```bash
wk init
```

**Single Project** - Track issues in your repo (git branch):
```bash
wk init --remote .
```

**Multiple Projects** - Track issues across multiple projects (separate git repo):
```bash
wk init --remote ~/tracker
```

**Fleet** - Central coordination for agents and automation:
```bash
wk init --remote ws://host:port
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
make install   # Build and install the wk binary
make test      # Run tests
make spec      # Run BATS specs
make bench     # Run benchmarks
make quality   # Evaluate code quality
```

See individual README files in `checks/` for detailed documentation.

## License

MIT - Copyright (c) 2026 Alfred Jean LLC
