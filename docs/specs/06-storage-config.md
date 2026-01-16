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
wk init --prefix prj

# Initialize at shared location (e.g., monorepo root)
wk init --path /path/to/shared --prefix prj
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
