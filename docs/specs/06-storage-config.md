# Storage & Configuration

## Data Directory

The `.wok/` directory contains:
- `config.toml` - Project configuration
- `issues.db` - SQLite database (unless `workspace` is set)

```toml
# .wok/config.toml
prefix = "prj"

# Optional: store issues.db in a different location (absolute or relative path)
# workspace = "../shared-issues"
```

When `workspace` is set, `issues.db` lives at that path instead of `.wok/`.

When `--workspace` is used without `--prefix`:
- Creates `.wok/config.toml` with only `workspace = "<path>"`
- No prefix is set in config (will be loaded from workspace's config)
- No local `issues.db` is created

## Database Location

The CLI finds `.wok/` by walking up from cwd. If not found, error with helpful message.

```bash
# Initialize in current directory
wok init --prefix prj

# Initialize at shared location (e.g., monorepo root)
wok init --path /path/to/shared --prefix prj
```

## Prefix Registry

The database maintains a `prefixes` table that automatically tracks all prefixes used in issue IDs:

```sql
CREATE TABLE prefixes (
    prefix TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    issue_count INTEGER NOT NULL DEFAULT 0
);
```

Prefixes are tracked automatically:
- When creating an issue, the prefix is registered and its count incremented
- When renaming a prefix via `wok config rename`, the table is updated
- Existing databases are backfilled on first open

List all prefixes with `wok config prefixes`:

```bash
wok config prefixes
# proj: 5 issues (default)
# api: 2 issues

wok config prefixes -o json
# {"default": "proj", "prefixes": [...]}
```

Create issues with different prefixes using `--prefix`:

```bash
wok new "API task" --prefix api
# Creates api-XXXX instead of using config prefix
```

## Git Integration (User Choice)

The CLI does NOT automatically configure git. Users choose:

**Option A: Track in git** (collaborative)
```bash
git add .wok/
```

**Option B: Private (gitignore)**
```bash
echo ".wok/" >> .gitignore
```

**Option C: Private (git/info/exclude)**
```bash
echo ".wok/" >> .git/info/exclude  # local-only, not in repo
```
