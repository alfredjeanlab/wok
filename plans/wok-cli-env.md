# Plan: Centralized `env.rs` Module for wok CLI

## Overview

Add a centralized `env.rs` module to `crates/cli/src/` that becomes the single source of truth for all runtime environment variable access. A `build.rs` script generates string constants for env var names so they're never stringly-typed in application code. All existing call sites (`timings.rs`, `colors.rs`, `config.rs`, `daemon/lifecycle.rs`) are updated to use the new module.

## Project Structure

```
crates/cli/
├── build.rs              # NEW — generates env var name constants
├── src/
│   ├── env.rs            # NEW — typed accessors for all runtime env vars
│   ├── env_tests.rs      # NEW — unit tests for env.rs
│   ├── lib.rs            # MODIFIED — add `pub mod env;`
│   ├── timings.rs        # MODIFIED — use crate::env::wk_timings()
│   ├── colors.rs         # MODIFIED — use crate::env::{no_color, force_color}
│   ├── config.rs         # MODIFIED — use crate::env::{wok_state_dir_var, xdg_state_home}
│   └── daemon/
│       └── lifecycle.rs  # MODIFIED — use crate::env::daemon_binary()
```

## Dependencies

No new external dependencies. Uses only `std::env::var` and `std::path::PathBuf`.

## Implementation Phases

### Phase 1: Create `build.rs` with env var name constants

Create `crates/cli/build.rs` that writes a generated file defining string constants for each environment variable name:

```rust
// crates/cli/build.rs
use std::io::Write;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = std::path::Path::new(&out_dir).join("env_vars.rs");
    let mut f = std::fs::File::create(path).unwrap();

    let vars = [
        ("WK_TIMINGS", "WK_TIMINGS"),
        ("NO_COLOR", "NO_COLOR"),
        ("COLOR", "COLOR"),
        ("WOK_STATE_DIR", "WOK_STATE_DIR"),
        ("XDG_STATE_HOME", "XDG_STATE_HOME"),
        ("WOK_DAEMON_BINARY", "WOK_DAEMON_BINARY"),
    ];

    for (const_name, env_name) in &vars {
        writeln!(f, "pub const {const_name}: &str = \"{env_name}\";").unwrap();
    }
}
```

**Verify**: `cargo check` passes.

### Phase 2: Create `env.rs` with typed accessors

Create `crates/cli/src/env.rs` with:

- `include!` of the generated constants file (under a `pub mod vars` or similar)
- Typed accessor functions for each env var

```rust
// crates/cli/src/env.rs

/// Generated environment variable name constants.
pub mod vars {
    include!(concat!(env!("OUT_DIR"), "/env_vars.rs"));
}

/// Returns true if `WK_TIMINGS` is set (any value).
pub fn wk_timings() -> bool {
    std::env::var(vars::WK_TIMINGS).is_ok()
}

/// Returns true if `NO_COLOR=1`.
pub fn no_color() -> bool {
    std::env::var(vars::NO_COLOR).is_ok_and(|v| v == "1")
}

/// Returns true if `COLOR=1`.
pub fn force_color() -> bool {
    std::env::var(vars::COLOR).is_ok_and(|v| v == "1")
}

/// Returns the value of `WOK_STATE_DIR` if set.
pub fn state_dir() -> Option<std::path::PathBuf> {
    std::env::var(vars::WOK_STATE_DIR).ok().map(std::path::PathBuf::from)
}

/// Returns the value of `XDG_STATE_HOME` if set.
pub fn xdg_state_home() -> Option<std::path::PathBuf> {
    std::env::var(vars::XDG_STATE_HOME).ok().map(std::path::PathBuf::from)
}

/// Returns the value of `WOK_DAEMON_BINARY` if set.
pub fn daemon_binary() -> Option<std::path::PathBuf> {
    std::env::var(vars::WOK_DAEMON_BINARY).ok().map(std::path::PathBuf::from)
}
```

Register the module in `lib.rs`:

```rust
pub mod env;
```

**Verify**: `cargo check` passes.

### Phase 3: Create `env_tests.rs`

Create `crates/cli/src/env_tests.rs` following the sibling test file convention. Add the `#[cfg(test)]` link in `env.rs`.

Tests should cover:
- `wk_timings()` returns false when unset, true when set
- `no_color()` returns true only when value is `"1"`, false for other values or unset
- `force_color()` returns true only when value is `"1"`
- `state_dir()` returns `None` when unset, `Some(PathBuf)` when set
- `xdg_state_home()` returns `None` when unset, `Some(PathBuf)` when set
- `daemon_binary()` returns `None` when unset, `Some(PathBuf)` when set
- `vars::*` constants match expected string values

Note: Since env var tests mutate process-global state, use `std::env::set_var`/`remove_var` and run with `cargo test -- --test-threads=1` or use unique env var prefixes in tests if needed. The existing `config_tests.rs` likely has a pattern to follow.

**Verify**: `cargo test` passes.

### Phase 4: Update call sites

Update each file to use `crate::env::*` instead of direct `std::env::var()`:

1. **`timings.rs`** (line 14): Replace `std::env::var("WK_TIMINGS").is_ok()` with `crate::env::wk_timings()`

2. **`colors.rs`** (lines 31, 36): Replace:
   - `std::env::var("NO_COLOR").is_ok_and(|v| v == "1")` → `crate::env::no_color()`
   - `std::env::var("COLOR").is_ok_and(|v| v == "1")` → `crate::env::force_color()`

3. **`config.rs`** (lines 126-131): Replace:
   - `std::env::var("WOK_STATE_DIR")` → `crate::env::state_dir()`
   - `std::env::var("XDG_STATE_HOME")` → `crate::env::xdg_state_home()`
   - Keep the fallback logic (`dirs::home_dir()` etc.) in `config.rs`

   The `wok_state_dir()` function becomes:
   ```rust
   pub fn wok_state_dir() -> PathBuf {
       if let Some(dir) = crate::env::state_dir() {
           return dir;
       }
       if let Some(dir) = crate::env::xdg_state_home() {
           return dir.join("wok");
       }
       dirs::home_dir()
           .map(|h| h.join(".local/state/wok"))
           .unwrap_or_else(|| PathBuf::from(".local/state/wok"))
   }
   ```

4. **`daemon/lifecycle.rs`** (line 168): Replace `std::env::var("WOK_DAEMON_BINARY")` → `crate::env::daemon_binary()`

   ```rust
   fn find_wokd_binary() -> Result<PathBuf> {
       if let Some(path) = crate::env::daemon_binary() {
           return Ok(path);
       }
       // ... rest unchanged
   }
   ```

**Verify**: `cargo check` and `cargo test` pass. Grep for stray `std::env::var` calls in `crates/cli/src/` to confirm none remain (except `std::env::current_dir`, `std::env::current_exe`, `std::env::set_current_dir`, and `std::env::args` which are not env var reads).

### Phase 5: Final validation

- Run `make check-fast`
- Confirm no clippy warnings
- Confirm no dead code warnings
- Grep `crates/cli/src/` for direct `std::env::var(` calls — only non-env-var uses should remain

## Key Implementation Details

- **`build.rs` constants**: The generated file contains simple `pub const` string slices. Using `include!` keeps them available at compile time without a proc macro. The constants live in `env::vars::` namespace.
- **No lazy_static or OnceLock**: Each accessor calls `std::env::var` directly. This is intentional — env vars can change (e.g., in tests), and the overhead of a syscall per check is negligible for CLI startup.
- **Accessor return types**: Bool accessors return `bool`, path accessors return `Option<PathBuf>`. The `config.rs` fallback chain stays in `config.rs` since it involves non-env-var logic (`dirs::home_dir`).
- **Test isolation**: Env var tests may need `--test-threads=1` or use `unsafe { std::env::set_var() }`. Check how `config_tests.rs` handles this.

## Verification Plan

1. `cargo check` — compiles without errors
2. `cargo clippy -- -D warnings` — no lint warnings
3. `cargo test` — all tests pass including new `env_tests.rs`
4. `cargo fmt --check` — formatting correct
5. Grep audit: `rg 'std::env::var\(' crates/cli/src/` shows only non-env-var uses (`current_dir`, `current_exe`, `args`, and `OUT_DIR` in build.rs)
6. `make check-fast` — full validation suite passes
