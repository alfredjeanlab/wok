# Landing Checklist

- [ ] Code compiles: `cargo check`
- [ ] Unit tests in sibling files (see below)
- [ ] No dead code warnings
  - Truly unused → delete it
  - Test-only → `#[cfg(test)]`
  - `#[allow(dead_code)]` requires justifying comment
- [ ] No `======` comment banners
- [ ] No escape hatches in non-test code
  - `unsafe`, `unwrap`, `expect`, numeric casts → use safe alternatives
  - Test files (`*_tests.rs`) may use `#![allow(...)]`
- [ ] No vulnerabilities: `cargo audit`
  - Log any issues found with `wk new bug "..."`
- [ ] Linting passes: `cargo clippy`
- [ ] Unit tests pass: `cargo test`
- [ ] Formatting passes: `cargo fmt`
- [ ] Coverage: `make coverage` (≥90% lines)
- [ ] Commit: `git commit`
- [ ] Push: `git push`

## Unit Test Convention

Use sibling `_tests.rs` files instead of inline `#[cfg(test)]` modules:

```rust
// src/parser.rs
#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
```

```rust
// src/parser_tests.rs
use super::*;

#[test]
fn parses_empty_input() { ... }
```

**Why separate files?**
- Shorter source files fit better in LLM context windows
- LOC metrics reflect implementation conciseness, not test volume
- Integration tests remain in `tests/` as usual
