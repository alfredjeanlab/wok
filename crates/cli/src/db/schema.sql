-- Core issue table
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'todo',
    assignee TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Dependencies with relationship types
CREATE TABLE IF NOT EXISTS deps (
    from_id TEXT NOT NULL,
    to_id TEXT NOT NULL,
    rel TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (from_id, to_id, rel),
    FOREIGN KEY (from_id) REFERENCES issues(id),
    FOREIGN KEY (to_id) REFERENCES issues(id),
    CHECK (from_id != to_id)
);

-- Labels as raw strings
CREATE TABLE IF NOT EXISTS labels (
    issue_id TEXT NOT NULL,
    label TEXT NOT NULL,
    PRIMARY KEY (issue_id, label),
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Status-aware notes
CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    status TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Event log (audit trail)
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    action TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- External links to issue trackers
CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    link_type TEXT,              -- github|jira|gitlab|confluence|NULL
    url TEXT,                    -- full URL (may be NULL for shorthand)
    external_id TEXT,            -- external issue ID (e.g., "PE-5555")
    rel TEXT,                    -- import|blocks|tracks|tracked-by|NULL
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
CREATE INDEX IF NOT EXISTS idx_issues_type ON issues(type);
CREATE INDEX IF NOT EXISTS idx_deps_to ON deps(to_id);
CREATE INDEX IF NOT EXISTS idx_deps_rel ON deps(rel);
CREATE INDEX IF NOT EXISTS idx_labels_label ON labels(label);
CREATE INDEX IF NOT EXISTS idx_events_issue ON events(issue_id);
CREATE INDEX IF NOT EXISTS idx_links_issue ON links(issue_id);
