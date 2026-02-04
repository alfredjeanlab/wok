# Wok - Specifications

Product specifications defining wok's design, behavior, and interfaces.

## Table of Contents

1. **[Overview & Design Philosophy](01-overview.md)**
   - Overview of the issue tracker
   - Unix philosophy and AI-first design principles
   - Hidden flags and priority tag system
   - Semantic flag policy

2. **[Core Concepts](02-core-concepts.md)**
   - Issue types (feature, task, bug, chore, idea)
   - Status state machine
   - Dependencies (hierarchical)
   - Notes by status
   - External links

3. **[Data Model](03-data-model.md)**
   - SQLite schema and tables
   - Relationships and semantics
   - ID generation

4. **[CLI Interface](04-cli-interface.md)**
   - Command structure
   - All commands (help, setup, lifecycle, viewing, search, dependencies, links, labels, notes, log, export, import, shell completion, schema, daemon, remote, hooks)
   - Output format
   - Claude Code integration

5. **[Business Rules](05-business-rules.md)**
   - Dependency constraints
   - External link constraints
   - Relationship semantics
   - Status transitions
   - Notes behavior
   - Input limits and validation

6. **[Storage & Configuration](06-storage-config.md)**
   - Data directory structure
   - Database location
   - Git integration options

7. **[Project Structure](08-project-structure.md)**
   - Project layout (crates/, tests/)
   - Test suites
   - Running tests
   - CLI behavior
