# Show Multiple Issues with JSONL Output

## Overview

Extend `wok show` to accept multiple issue IDs. In text mode, separate each issue with `---` (standard document separator). In JSON mode, output JSONL (JSON Lines) format—one JSON object per line—so that single and multiple issue output share the same format structure.

## Project Structure

Files to modify:
```
crates/cli/
├── src/
│   ├── cli.rs           # Update Show command args
│   ├── lib.rs           # Update dispatch to pass Vec<String>
│   └── commands/
│       └── show.rs      # Core implementation changes
tests/specs/
└── cli/unit/
    └── show.bats        # Add tests for multiple IDs
docs/specs/
└── show.md              # Update documentation (if exists)
```

## Dependencies

No new external dependencies required. Uses existing:
- `serde_json` for JSONL output (already available)
- Existing database methods for issue retrieval

## Implementation Phases

### Phase 1: Update CLI Argument Parsing

**File:** `crates/cli/src/cli.rs`

Change the `Show` variant from single ID to multiple IDs:

```rust
Show {
    /// Issue ID(s)
    #[arg(required = true, num_args = 1..)]
    ids: Vec<String>,

    /// Output format (text, json)
    #[arg(long = "output", short = 'o', default_value = "text")]
    output: String,
}
```

**File:** `crates/cli/src/lib.rs`

Update the dispatch:

```rust
Command::Show { ids, output } => commands::show::run(&ids, &output),
```

**Milestone:** Command parses multiple IDs; existing single-ID behavior unchanged.

### Phase 2: Refactor show.rs for Multiple Issues

**File:** `crates/cli/src/commands/show.rs`

Update function signatures:

```rust
pub fn run(ids: &[String], format: &str) -> Result<()> {
    let (db, _, _) = open_db()?;
    run_impl(&db, ids, format)
}

fn run_impl(db: &Database, ids: &[String], format: &str) -> Result<()> {
    // Resolve all IDs first
    let resolved_ids: Vec<String> = ids
        .iter()
        .map(|id| db.resolve_id(id))
        .collect::<Result<Vec<_>>>()?;

    match format {
        "json" => output_json(db, &resolved_ids),
        "text" | _ => output_text(db, &resolved_ids),
    }
}
```

**Milestone:** Multiple IDs resolve correctly; errors if any ID is invalid/ambiguous.

### Phase 3: Implement JSONL Output

**File:** `crates/cli/src/commands/show.rs`

Add JSON output function using JSONL format (one compact JSON object per line):

```rust
fn output_json(db: &Database, ids: &[String]) -> Result<()> {
    for id in ids {
        let details = build_issue_details(db, id)?;
        // Use to_string (not to_string_pretty) for JSONL
        let json = serde_json::to_string(&details)?;
        println!("{}", json);
    }
    Ok(())
}
```

Extract existing detail-building logic into a helper:

```rust
fn build_issue_details(db: &Database, id: &str) -> Result<IssueDetails> {
    let issue = db.get_issue(id)?;
    let labels = db.get_labels(id)?;
    let blockers = db.get_blockers(id)?;
    let blocking = db.get_blocking(id)?;
    let parents = db.get_tracking(id)?;
    let children = db.get_tracked(id)?;
    let notes = db.get_notes(id)?;
    let links = db.get_links(id)?;
    let events = db.get_events(id)?;

    Ok(IssueDetails {
        issue,
        labels,
        blockers,
        blocking,
        parents,
        children,
        notes,
        links,
        events,
    })
}
```

**Milestone:** `wok show ID1 ID2 -o json` outputs two lines of JSON.

### Phase 4: Implement Text Output with Separators

**File:** `crates/cli/src/commands/show.rs`

Add text output function with `---` separators between issues:

```rust
fn output_text(db: &Database, ids: &[String]) -> Result<()> {
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            println!("---");
        }
        output_single_text(db, id)?;
    }
    Ok(())
}

fn output_single_text(db: &Database, id: &str) -> Result<()> {
    let issue = db.get_issue(id)?;
    let labels = db.get_labels(id)?;
    let blockers = db.get_blockers(id)?;
    let blocking = db.get_blocking(id)?;
    let parents = db.get_tracking(id)?;
    let children = db.get_tracked(id)?;
    let notes = db.get_notes_by_status(id)?;
    let links = db.get_links(id)?;
    let events = db.get_events(id)?;

    print!("{}", format_issue_details(
        &issue, &labels, &blockers, &blocking,
        &parents, &children, &notes, &links, &events
    ));
    Ok(())
}
```

**Milestone:** `wok show ID1 ID2` outputs both issues separated by `---`.

### Phase 5: Add Tests

**File:** `tests/specs/cli/unit/show.bats`

Add test cases:

```bash
@test "show: multiple issues in text mode separated by ---" {
    wk new "First issue"
    wk new "Second issue"
    run wk show 1 2
    assert_success
    assert_output --partial "First issue"
    assert_output --partial "---"
    assert_output --partial "Second issue"
}

@test "show: multiple issues in json mode outputs JSONL" {
    wk new "First issue"
    wk new "Second issue"
    run wk show 1 2 -o json
    assert_success
    # Count lines - should be 2 (one per issue)
    line_count=$(echo "$output" | wc -l | tr -d ' ')
    assert_equal "$line_count" "2"
    # Each line should be valid JSON
    echo "$output" | head -1 | jq . >/dev/null
    echo "$output" | tail -1 | jq . >/dev/null
}

@test "show: single issue json format unchanged (compact)" {
    wk new "Test issue"
    run wk show 1 -o json
    assert_success
    # Single line of JSON
    line_count=$(echo "$output" | wc -l | tr -d ' ')
    assert_equal "$line_count" "1"
}

@test "show: fails if any ID is invalid" {
    wk new "Test issue"
    run wk show 1 nonexistent
    assert_failure
    assert_output --partial "not found"
}
```

**Milestone:** All new tests pass.

## Key Implementation Details

### JSONL Format Rationale

Using JSONL (one JSON object per line) instead of a JSON array ensures:
1. **Consistency**: Single and multiple issue outputs have identical per-issue format
2. **Streaming**: Output can be processed line-by-line without loading all into memory
3. **Compatibility**: Tools like `jq` can process JSONL with `jq -s` or line-by-line
4. **Precedent**: The existing `export` command uses JSONL format

### Breaking Change: Single Issue JSON

The current single-issue JSON output uses `to_string_pretty()` (multi-line formatted JSON). With JSONL, single-issue output changes to compact single-line JSON. This is intentional for format consistency but is technically a breaking change for scripts parsing the output.

If backward compatibility is critical, an alternative would be:
- Keep pretty JSON for single issue
- Use JSONL only for multiple issues

However, the user's requirement explicitly states "the format of 'show one item' and the format of 'show multiple' is the same" so JSONL for both is the correct approach.

### Error Handling Strategy

Errors fail fast—if any ID is invalid or ambiguous, the command fails before outputting anything. This matches the existing behavior of other commands and avoids partial output that could confuse scripts.

Alternative considered (but not recommended): Output valid issues, then report errors. This would require a `BulkResult` pattern like `lifecycle.rs`, but complicates script parsing and goes against the principle of atomic operations.

### Separator Choice

The `---` separator is used because:
1. It's a standard YAML/TOML document separator
2. It's visually distinct and unlikely to appear in issue content
3. It matches the user's explicit request

## Verification Plan

1. **Unit tests**: Run `make spec ARGS='--file cli/unit/show.bats'`
2. **Manual testing**:
   ```bash
   # Create test issues
   wk new "Issue A"
   wk new "Issue B"
   wk note 1 "A note on issue A"

   # Test text mode
   wk show 1 2           # Should show both with --- separator
   wk show 1             # Single issue (no separator)

   # Test JSON mode
   wk show 1 2 -o json   # Two lines of JSON
   wk show 1 -o json     # One line of JSON
   wk show 1 2 -o json | jq -s .  # Parse as array with jq

   # Test error cases
   wk show 1 nonexistent # Should fail
   ```
3. **Format validation**: Pipe JSON output through `jq` to ensure validity
4. **Full validation**: Run `make validate` before committing
