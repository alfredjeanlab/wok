# Data Model (SQLite Schema)

## Tables

```sql
-- Core issue table
CREATE TABLE issues (
    id TEXT PRIMARY KEY,           -- e.g. "PROJ-a3f2" (prefix + hash)
    type TEXT NOT NULL,            -- feature|task|bug|chore
    title TEXT NOT NULL,           -- short title, no description
    status TEXT NOT NULL DEFAULT 'todo',  -- todo|in_progress|done|closed
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Dependencies with relationship types
CREATE TABLE deps (
    from_id TEXT NOT NULL,         -- source issue
    to_id TEXT NOT NULL,           -- target issue
    rel TEXT NOT NULL,             -- relationship type: blocks|tracked-by|tracks
    created_at TEXT NOT NULL,
    PRIMARY KEY (from_id, to_id, rel),
    FOREIGN KEY (from_id) REFERENCES issues(id),
    FOREIGN KEY (to_id) REFERENCES issues(id),
    CHECK (from_id != to_id)
);
-- Semantics:
--   A blocks B     = B should wait for A (informational, used for `ready` command and --blocked)
--   A tracked-by B = A belongs to B (A is part of feature B)
--   A tracks B     = A contains B (A is a feature containing B)

-- Labels as raw strings
CREATE TABLE labels (
    issue_id TEXT NOT NULL,
    label TEXT NOT NULL,           -- raw text, e.g. "project:auth" or "urgent"
    PRIMARY KEY (issue_id, label),
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Status-aware notes
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    status TEXT NOT NULL,          -- status when note was added (todo|in_progress|done)
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Event log (audit trail)
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    action TEXT NOT NULL,          -- created|edited|started|stopped|done|closed|reopened|labeled|unlabeled|related|unrelated|linked|unlinked|noted|unblocked
    old_value TEXT,                -- previous value (for changes)
    new_value TEXT,                -- new value
    reason TEXT,                   -- reason for close/reopen/prior
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- External links to issue trackers
CREATE TABLE links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    link_type TEXT,              -- github|jira|gitlab|confluence|NULL
    url TEXT,                    -- full URL (may be NULL for shorthand)
    external_id TEXT,            -- external issue ID (e.g., "PE-5555")
    rel TEXT,                    -- import|blocks|tracks|tracked-by|NULL
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Prefix registry (auto-populated)
CREATE TABLE prefixes (
    prefix TEXT PRIMARY KEY,       -- e.g. "proj", "api"
    created_at TEXT NOT NULL,      -- when prefix was first used
    issue_count INTEGER NOT NULL DEFAULT 0
);

-- Indexes
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_type ON issues(type);
CREATE INDEX idx_deps_to ON deps(to_id);
CREATE INDEX idx_deps_rel ON deps(rel);
CREATE INDEX idx_labels_label ON labels(label);
CREATE INDEX idx_events_issue ON events(issue_id);
CREATE INDEX idx_links_issue ON links(issue_id);
CREATE INDEX idx_prefixes_count ON prefixes(issue_count DESC);
```

## ID Generation

IDs use a configurable prefix + short hash of (title + timestamp):

```
{prefix}-{hash}
```

- **Prefix**: Lowercase, 2-16 chars, configured in `.wok/config.toml`. Default: current directory name lowercased, truncated to 16 chars, non-alpha chars removed. Must be at least 2 chars after sanitization.
- **Hash**: First 4 chars of SHA256(title + created_at timestamp)
- **Collision handling**: If ID exists, append incrementing suffix: `prj-a3f2`, `prj-a3f2-2`, `prj-a3f2-3`

Example: `prj-a3f2`, `auth-9bc1`
