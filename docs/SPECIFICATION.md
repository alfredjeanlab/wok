# Simple Local Issue Tracker CLI - Requirements

This document has been split into semantic sections for easier navigation and maintenance. All detailed specifications are in the `specs/` directory.

## Table of Contents

1. **[Overview & Design Philosophy](specs/01-overview.md)**
   - Overview of the issue tracker
   - Unix philosophy and AI-first design principles
   - Hidden flags and priority tag system
   - Semantic flag policy

2. **[Core Concepts](specs/02-core-concepts.md)**
   - Issue types (feature, task, bug, chore)
   - Status state machine
   - Dependencies (hierarchical)
   - Notes by status
   - External links

3. **[Data Model](specs/03-data-model.md)**
   - SQLite schema and tables
   - Relationships and semantics
   - ID generation

4. **[CLI Interface](specs/04-cli-interface.md)**
   - Command structure
   - All commands (help, setup, lifecycle, viewing, dependencies, links, labels, notes, log, export, import, shell completion, remote, hooks)
   - Output format
   - Claude Code integration

5. **[Business Rules](specs/05-business-rules.md)**
   - Dependency constraints
   - External link constraints
   - Relationship semantics
   - Status transitions
   - Notes behavior
   - Input limits and validation

6. **[Storage & Configuration](specs/06-storage-config.md)**
   - Data directory structure
   - Database location
   - Git integration options

7. **[Remote Sync](specs/07-remote-sync.md)**
   - Configuration
   - Architecture (client, daemon, server)
   - Daemon lifecycle
   - Connection states
   - Sync behavior and offline mode
   - Conflict resolution
   - Error handling

8. **[Project Structure](specs/08-project-structure.md)**
   - Project layout (bin/, tests/)
   - Test suites
   - Running tests
   - CLI behavior
