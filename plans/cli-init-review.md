# CLI Init Review Implementation Plan

## Overview

Review and enhance the Rust unit tests for the `init` command to ensure full coverage of all behaviors specified in `init.bats`, add missing edge cases, apply parameterization with `yare` where appropriate, and ensure tests follow idiomatic Rust patterns.

## Project Structure

```
crates/cli/
├── src/
│   ├── commands/
│   │   ├── init.rs          # Init command implementation
│   │   └── init_tests.rs    # Unit tests for init command logic
│   ├── cli_tests/
│   │   └── init_tests.rs    # CLI argument parsing tests
│   ├── id.rs                # Prefix validation
│   └── id_tests.rs          # Prefix validation tests (uses yare)
└── tests/
    └── init.rs              # Integration tests

tests/specs/cli/unit/
└── init.bats                # Specification tests
```

## Dependencies

- `yare = "3"` - Already in dev-dependencies for parameterized tests
- `tempfile` - Already used for TempDir
- `assert_cmd` - Already used for integration tests

## Implementation Phases

### Phase 1: Coverage Gap Analysis and Documentation

**Objective**: Document exact coverage gaps between BATS specs and Rust tests.

**Deliverable**: Annotated checklist of missing tests.

**BATS Test → Rust Coverage Matrix**:

| BATS Behavior | Unit Test | Integration Test |
|---------------|-----------|------------------|
| Basic init creates `.wok/`, `config.toml`, `issues.db` | ❌ | ✅ `creates_work_dir` |
| Re-init fails with "already initialized" | ❌ | ✅ `fails_if_already_initialized` |
| Partial `.wok/` (no config) allows init | ❌ | ❌ |
| `--path` creates at specified location | ❌ | ❌ |
| `--path` creates parent directories | ❌ | ❌ |
| `--path` fails if already initialized | ❌ | ❌ |
| Prefix derived from directory name | ❌ | ✅ `uses_directory_name_as_default_prefix` |
| Prefix lowercased and alphanumeric only | ❌ | ❌ |
| Invalid directory name for prefix fails | ❌ | ❌ |
| Valid prefixes ("ab", "abc", "abc123") | ✅ `id_tests.rs` | ✅ `valid_alphanumeric_prefix` |
| Invalid prefixes ("ABC", "123", "my-prefix", "a") | ✅ `id_tests.rs` | ✅ `invalid_prefix` |
| Database has required tables | ❌ | ❌ |
| Issue creation uses correct prefix | ❌ | ❌ |
| `--workspace` creates link without DB | ❌ | ✅ `init_with_workspace_creates_link` |
| `--workspace` + `--prefix` | ❌ | ✅ `init_with_workspace_and_prefix` |
| `--workspace` validates prefix | ❌ | ✅ `init_with_workspace_rejects_invalid_prefix` |
| `--workspace` accepts relative path | ❌ | ❌ |
| `--workspace` + `--path` | ❌ | ❌ |
| `--workspace` fails if not exists | ❌ | ✅ `init_with_workspace_fails_if_workspace_not_exist` |
| `.gitignore` includes `config.toml` (local) | ❌ | ✅ `init_defaults_to_local_mode` |
| `.gitignore` excludes `config.toml` (remote) | ❌ | ❌ |
| `--local` is no-op (backwards compat) | ❌ | ✅ `init_local_flag_is_no_op` |
| `--remote .` creates worktree | ❌ | ❌ |
| `--remote` installs git hooks | ❌ | ❌ |

**Files to create/modify**:
- No changes yet, just analysis

---

### Phase 2: Enhance `commands/init_tests.rs` with Unit Tests

**Objective**: Add unit tests for `derive_prefix_from_path` and init logic.

**Tests to add**:

```rust
// derive_prefix_from_path tests - use yare
#[yare::parameterized(
    simple_lowercase = { "myproject", "myproject" },
    mixed_case = { "MyProject", "myproject" },
    with_digits = { "Project123", "project123" },
    alphanumeric_only = { "my-project_v2", "myprojectv2" },
    unicode_stripped = { "проект", "" },  // Should fail
)]
fn derive_prefix_valid(dir_name: &str, expected: &str);

#[yare::parameterized(
    too_short_after_strip = { "a---" },
    all_dashes = { "---" },
    digits_only = { "123" },
    single_char = { "x" },
)]
fn derive_prefix_invalid(dir_name: &str);
```

**Additional unit tests**:
- `test_init_creates_gitignore_local_mode` - verify gitignore contents
- `test_init_creates_gitignore_remote_mode` - verify no config.toml
- `test_workspace_link_creates_gitignore` - verify workspace gitignore

---

### Phase 3: Enhance CLI Argument Parsing Tests with Parameterization

**Objective**: Apply yare to `cli_tests/init_tests.rs` for better coverage.

**Current tests** (6 individual tests) → **Refactor to parameterized**:

```rust
#[yare::parameterized(
    prefix_only = { &["wk", "init", "--prefix", "prj"], Some("prj"), None, None, None },
    path_only = { &["wk", "init", "--path", "/tmp"], None, Some("/tmp"), None, None },
    workspace_only = { &["wk", "init", "--workspace", "/ws"], None, None, Some("/ws"), None },
    remote_only = { &["wk", "init", "--remote", "."], None, None, None, Some(".") },
    prefix_and_path = { &["wk", "init", "--prefix", "prj", "--path", "/tmp"], Some("prj"), Some("/tmp"), None, None },
    all_options = { &["wk", "init", "--prefix", "prj", "--workspace", "/ws", "--remote", "."], Some("prj"), None, Some("/ws"), Some(".") },
)]
fn init_arg_parsing(
    args: &[&str],
    expected_prefix: Option<&str>,
    expected_path: Option<&str>,
    expected_workspace: Option<&str>,
    expected_remote: Option<&str>,
);
```

**Keep individual tests for**:
- `--local` flag (backwards compatibility)
- Error cases (conflicting options if any)

---

### Phase 4: Enhance Integration Tests

**Objective**: Add missing integration tests in `tests/init.rs`.

**Tests to add** (grouped by feature):

**Path handling**:
```rust
#[test]
fn init_with_path_creates_parent_dirs();

#[test]
fn init_with_path_fails_if_already_initialized();
```

**Partial initialization**:
```rust
#[test]
fn init_succeeds_if_wok_dir_exists_without_config();
```

**Prefix derivation edge cases** (parameterized):
```rust
#[yare::parameterized(
    mixed_case = { "MyProject", "myproject" },
    with_symbols = { "my-project_v2", "myprojectv2" },
    with_digits = { "Project123", "project123" },
)]
fn init_derives_prefix_from_directory_name(dir_name: &str, expected_prefix: &str);

#[yare::parameterized(
    too_short = { "a---" },
    digits_only = { "123" },
    all_symbols = { "---" },
)]
fn init_fails_with_underivable_prefix(dir_name: &str);
```

**Workspace edge cases**:
```rust
#[test]
fn init_workspace_accepts_relative_path();

#[test]
fn init_workspace_with_path_option();
```

**Remote mode** (requires git repo setup):
```rust
#[test]
fn init_remote_excludes_config_from_gitignore();

#[test]
fn init_remote_creates_oplog_worktree();
```

---

### Phase 5: Refactor for Idiomatic Rust

**Objective**: Apply Rust testing best practices.

**Improvements**:

1. **Use descriptive test names** following `should_<behavior>_when_<condition>` pattern:
   ```rust
   // Before
   fn test_valid_prefix()

   // After
   fn should_accept_two_char_lowercase_prefix()
   ```

2. **Consolidate test setup** with helper functions:
   ```rust
   fn init_in_temp_dir(args: &[&str]) -> (TempDir, Output) {
       let temp = TempDir::new().unwrap();
       let output = wk()
           .args(args)
           .current_dir(temp.path())
           .output()
           .unwrap();
       (temp, output)
   }
   ```

3. **Use more precise assertions**:
   ```rust
   // Before
   assert!(stdout.contains("Prefix:"));

   // After
   assert_eq!(config.prefix, "expected");
   ```

4. **Group related tests in modules**:
   ```rust
   mod prefix_validation {
       // All prefix-related tests
   }

   mod workspace_mode {
       // All workspace-related tests
   }
   ```

5. **Add doc comments** to test modules explaining what behavior is being tested.

---

### Phase 6: Validation and Cleanup

**Objective**: Ensure all tests pass and coverage is complete.

**Verification steps**:
1. Run `cargo test` - all Rust tests pass
2. Run `make spec-cli ARGS='--filter init'` - BATS specs still pass
3. Run `cargo clippy` - no warnings
4. Run `cargo fmt --check` - formatting correct
5. Cross-reference BATS tests with Rust coverage matrix

**Cleanup**:
- Remove duplicate test logic
- Ensure consistent error messages in assertions
- Remove any `todo:implement` tags from completed specs

## Key Implementation Details

### Yare Parameterization Pattern

```rust
use yare::parameterized;

#[parameterized(
    case_name_1 = { input1, expected1 },
    case_name_2 = { input2, expected2 },
)]
fn test_behavior(input: Type, expected: Type) {
    let result = function_under_test(input);
    assert_eq!(result, expected);
}
```

### Test File Organization

Following project convention from `crates/cli/CLAUDE.md`:
- Unit tests in sibling `_tests.rs` files
- Integration tests in `tests/` directory
- Use `#![allow(clippy::unwrap_used)]` at top of test files

### Prefix Validation Rules (from `id.rs:45-51`)

```rust
prefix.len() >= 2
    && prefix.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    && prefix.chars().any(|c| c.is_ascii_lowercase())
```

## Verification Plan

### Unit Tests
```bash
cargo test --lib init
cargo test --lib id
```

### Integration Tests
```bash
cargo test --test init
```

### Full Validation
```bash
make validate        # All checks
make spec-cli ARGS='--filter init'  # BATS specs
```

### Coverage Check
```bash
make coverage        # Should be ≥85%
```

### Checklist Before Completion
- [ ] All BATS behaviors have corresponding Rust tests
- [ ] Parameterization applied where 3+ similar test cases exist
- [ ] No duplicate test logic
- [ ] Descriptive test names
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `make spec-cli ARGS='--filter init'` passes
