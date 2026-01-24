# Text Normalization Requirements

## 1. Basic Trimming (All Fields)

All text fields (title, description, note, reason) have leading and trailing
whitespace trimmed before any other processing.

## 2. Title Processing Decision Tree

```
INPUT: raw title string

1. TRIM whitespace from start and end
2. SCAN for double-newline ("\n\n") position
   - If NO double-newline found: GOTO step 4
   - If double-newline found: continue to step 3

3. DETERMINE split point
   - Count words and characters from start
   - Find first double-newline AFTER (first 3 words OR 20 characters)
   - If double-newline is BEFORE threshold: don't split, GOTO step 4
   - If double-newline is AT OR AFTER threshold:
     a. SPLIT: everything before = title, everything after = prepend to description
     b. TRIM both parts

4. NORMALIZE remaining title:
   a. TOKENIZE into: quoted strings, unquoted segments
   b. For UNQUOTED segments:
      - Replace all newlines (\n, \r\n, \r) with single space
      - Replace consecutive whitespace with single space
   c. For QUOTED strings (", ', `, and typographic variants):
      - Preserve content but escape newlines as \n literal
   d. REASSEMBLE tokens
```

## 3. Quote Recognition

Recognized quote characters:
- Double quote: " (U+0022)
- Single quote: ' (U+0027)
- Backtick: ` (U+0060)
- Typographic double quotes: " " (U+201C, U+201D)
- Typographic single quotes: ' ' (U+2018, U+2019)

Quote pairing rules:
- Opening quote must match closing quote type
- Typographic open matches typographic close of same type
- Unclosed quotes at end of string: treat rest as quoted

## 4. Examples

### Simple trim
Input:  "  hello world  "
Output: "hello world"

### Newline to space
Input:  "hello\nworld"
Output: "hello world"

### Whitespace collapse
Input:  "hello   world"
Output: "hello world"

### Title split
Input:  "Fix the bug\n\nThis is a detailed description"
Output: title="Fix the bug", description="This is a detailed description"

### No split (before threshold)
Input:  "Hi\n\nthere"
Output: title="Hi there" (no split - "Hi" is only 1 word and 2 chars)

### Quoted newline preservation
Input:  'Error: "line1\nline2"'
Output: 'Error: "line1\nline2"' (newline escaped in quotes)

### Mixed
Input:  '  Fix\n\n  "error\nmsg"  in   module  '
Output: 'Fix "error\nmsg" in module'

# wk search - Full-Text Search Command

## CLI Interface

```bash
wk search <query> [options]
```

Performs full-text search across issue titles, descriptions, notes, labels, and external links.

### Options

- `<query>` - Required search string (supports case-insensitive substring matching)
- `--status <status>` / `-s <status>` - Filter by status (comma-separated for OR, repeat for AND)
- `--type <type>` / `-t <type>` - Filter by issue type (comma-separated for OR, repeat for AND)
- `--label <label>` / `-l <label>` - Filter by label (comma-separated for OR, repeat for AND)
- `--output <format>` / `-o <format>` - Output format: `text` (default) or `json`

### Output Format

**Text format** (default): Same as `wk list` output - one line per matching issue with ID, type, status, and title.

**JSON format**: Structured output with array of issues containing:
```json
{
  "issues": [
    {
      "id": "prj-1",
      "issue_type": "task",
      "status": "todo",
      "title": "Issue title",
      "labels": ["priority:high", "backend"]
    }
  ]
}
```

### Search Behavior

The search query is matched case-insensitively against:
1. Issue title
2. Issue description
3. Note content (all notes associated with the issue)
4. Labels (exact label names)
5. External link URLs
6. External link IDs (e.g., PR-123)

Results are matched if the query appears as a substring in any of these fields.

Special characters `%` and `_` in the query are escaped to prevent SQL LIKE syntax interpretation.

Results are sorted by:
1. Priority (ascending: 0=highest first)
2. Created date (descending: newest first)

Same as `wk list` sorting.

### Examples

```bash
# Search for 'login' in all fields
wk search "login"

# Search with case-insensitive matching
wk search "AUTHENTICATION"

# Search only in todo issues
wk search "auth" --status todo

# Search only bugs with specific label
wk search "crash" --type bug --label urgent

# Search with multiple label filters (AND logic)
wk search "task" --label backend --label urgent

# Search with label OR logic
wk search "task" --label "backend,frontend"

# Search with output in JSON format
wk search "oauth" --output json

# Search with no matches (returns empty)
wk search "nonexistent"
```

### Filtering Logic

Filters use the same grouping logic as `wk list`:
- **Status / Type / Label filters**: Comma-separated values within a single flag are OR'd
- **Multiple flags**: Each separate flag argument creates an AND condition
- **No filters**: Returns all matching issues (respects default behavior to show only open issues? No - search shows all matching issues regardless of status unless explicitly filtered)

### Implementation Notes

- Uses SQLite LIKE matching with COLLATE NOCASE for substring matching
- No full-text search index required (FTS5) - LIKE is sufficient for typical scale
- Query results are returned in memory and filtered to avoid complex SQL
- Reuses filter parsing logic from `wk list` command
