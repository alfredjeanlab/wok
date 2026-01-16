# Fuzz Testing Plan for wk and wk-daemon

## Overview

Implement fuzz testing for the `cmd/wk` CLI and `cmd/wk-daemon` (wk-remote) server to find crashes, panics, and unexpected behavior in input parsing and deserialization code. The project uses Rust, making `cargo-fuzz` with libFuzzer the natural choice.

## Project Structure

```
fuzz/
├── Cargo.toml                    # Fuzz workspace configuration
├── fuzz_targets/
│   ├── hlc_parse.rs              # Hybrid Logical Clock parsing
│   ├── protocol_client.rs        # ClientMessage JSON deserialization
│   ├── protocol_server.rs        # ServerMessage JSON deserialization
│   ├── op_payload.rs             # OpPayload deserialization
│   ├── issue_types.rs            # IssueType/Status/Relation FromStr
│   ├── normalize_title.rs        # Title normalization & quote handling
│   └── import_format.rs          # Import JSONL parsing
└── corpus/                       # Seed inputs for each target
    ├── hlc_parse/
    ├── protocol_client/
    └── ...
```

## Dependencies

Add to workspace or as dev dependency:

```toml
# fuzz/Cargo.toml
[package]
name = "wk-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"
arbitrary = { version = "1", features = ["derive"] }
wk-core = { path = "../lib/core" }

# For CLI-specific targets
[dependencies.wk-cli]
path = "../bin/cli"
package = "wkrs"
```

**Tools required:**
- `cargo-fuzz` (`cargo install cargo-fuzz`)
- Nightly Rust toolchain (required by libFuzzer)

## Implementation Phases

### Phase 1: Setup Fuzz Infrastructure

**Goal:** Create fuzz workspace and verify it compiles

1. Create `fuzz/` directory at repository root
2. Create `fuzz/Cargo.toml` with libfuzzer-sys dependency
3. Add `wk-core` as dependency (most fuzz targets are here)
4. Create minimal fuzz target to verify setup works:

```rust
// fuzz/fuzz_targets/hlc_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str::FromStr;
use wk_core::hlc::Hlc;

fuzz_target!(|data: &str| {
    let _ = Hlc::from_str(data);
});
```

5. Run `cargo +nightly fuzz run hlc_parse` to verify setup

**Verification:** `cargo +nightly fuzz list` shows targets

### Phase 2: Core Data Type Fuzz Targets

**Goal:** Fuzz all `FromStr` implementations in wk-core

**Target: HLC Parsing** (`lib/core/src/hlc.rs`)
```rust
// fuzz/fuzz_targets/hlc_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str::FromStr;
use wk_core::hlc::Hlc;

fuzz_target!(|data: &str| {
    let _ = Hlc::from_str(data);
});
```

**Target: Issue Types** (`lib/core/src/issue.rs`)
```rust
// fuzz/fuzz_targets/issue_types.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str::FromStr;
use wk_core::issue::{IssueType, Status};
use wk_core::dep::Relation;

fuzz_target!(|data: &str| {
    let _ = IssueType::from_str(data);
    let _ = Status::from_str(data);
    let _ = Relation::from_str(data);
});
```

**Seed corpus:**
```
corpus/hlc_parse/
├── valid_1           # "1234567890-0-1"
├── valid_2           # "0-0-0"
├── max_values        # "18446744073709551615-4294967295-4294967295"
├── minimal           # "0-0-0"
└── two_parts         # "123-456"

corpus/issue_types/
├── task              # "task"
├── feature           # "feature"
├── bug               # "bug"
├── todo              # "todo"
├── in_progress       # "in_progress"
└── empty             # ""
```

**Verification:** Each target runs for 60 seconds without crashes

### Phase 3: Protocol Message Fuzz Targets

**Goal:** Fuzz JSON deserialization for WebSocket protocol

**Target: ClientMessage** (`lib/core/src/protocol.rs`)
```rust
// fuzz/fuzz_targets/protocol_client.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use wk_core::protocol::ClientMessage;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = ClientMessage::from_json(s);
    }
});
```

**Target: ServerMessage**
```rust
// fuzz/fuzz_targets/protocol_server.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use wk_core::protocol::ServerMessage;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = ServerMessage::from_json(s);
    }
});
```

**Target: OpPayload** (`lib/core/src/op.rs`)
```rust
// fuzz/fuzz_targets/op_payload.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use wk_core::op::OpPayload;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<OpPayload>(s);
    }
});
```

**Seed corpus for protocol:**
```json
// corpus/protocol_client/op.json
{"type":"op","payload":{"type":"create_issue","id":"abc","issue_type":"task","title":"Test"}}

// corpus/protocol_client/sync.json
{"type":"sync","since":"0-0-0"}

// corpus/protocol_client/snapshot.json
{"type":"snapshot"}

// corpus/protocol_client/ping.json
{"type":"ping","id":1}
```

**Verification:** Run 5+ minutes per target, check coverage with `cargo +nightly fuzz coverage`

### Phase 4: CLI Input Processing Targets

**Goal:** Fuzz text normalization and import parsing in wk CLI

**Target: Title Normalization** (`bin/cli/src/normalize.rs`)

First, expose the normalize function for fuzzing by adding to `bin/cli/src/lib.rs`:
```rust
// Add to lib.rs public exports
pub use normalize::normalize_title;
```

```rust
// fuzz/fuzz_targets/normalize_title.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use wkrs::normalize_title;

fuzz_target!(|data: &str| {
    let _ = normalize_title(data);
});
```

**Seed corpus for normalization:**
```
corpus/normalize_title/
├── simple            # "Simple title"
├── quoted            # "Title with \"quoted\" text"
├── newlines          # "Title\n\nWith description"
├── unicode_quotes    # "Title with "smart quotes""
├── mixed             # "Test 'single' and \"double\""
└── empty             # ""
```

**Target: Import Format Detection**

This requires exposing import deserialization or testing at a lower level. Alternative approach using arbitrary structured input:

```rust
// fuzz/fuzz_targets/import_format.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

// Test JSONL line parsing (each line is one issue)
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        for line in s.lines() {
            // Try wk format
            let _ = serde_json::from_str::<serde_json::Value>(line);
        }
    }
});
```

**Verification:** Title normalization covers quote edge cases

### Phase 5: Structured Fuzzing with Arbitrary

**Goal:** Use structured fuzzing for more targeted coverage

**Structured OpPayload fuzzing:**
```rust
// fuzz/fuzz_targets/op_structured.rs
#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wk_core::op::{Op, OpPayload};
use wk_core::hlc::Hlc;

#[derive(Arbitrary, Debug)]
struct FuzzOp {
    wall_ms: u64,
    counter: u32,
    node_id: u32,
    payload_type: u8,
    id: String,
    title: String,
    label: String,
}

fuzz_target!(|input: FuzzOp| {
    // Construct valid-ish payloads from structured input
    let hlc = Hlc::new(input.wall_ms, input.counter, input.node_id);

    let payload = match input.payload_type % 4 {
        0 => OpPayload::CreateIssue {
            id: input.id.clone(),
            issue_type: wk_core::issue::IssueType::Task,
            title: input.title.clone(),
        },
        1 => OpPayload::SetTitle {
            issue_id: input.id.clone(),
            title: input.title.clone(),
        },
        2 => OpPayload::AddLabel {
            issue_id: input.id.clone(),
            label: input.label.clone(),
        },
        _ => OpPayload::SetStatus {
            issue_id: input.id.clone(),
            status: wk_core::issue::Status::Done,
            reason: Some(input.title.clone()),
        },
    };

    let op = Op { ts: hlc, payload };

    // Round-trip through JSON
    if let Ok(json) = serde_json::to_string(&op) {
        let _ = serde_json::from_str::<Op>(&json);
    }
});
```

**Verification:** Structured fuzzing finds deeper logic bugs

## Key Implementation Details

### Exposing Internal Functions

Some functions in `bin/cli` are private. Options:
1. Add `#[cfg(fuzzing)]` feature to expose them
2. Create a `fuzz` feature in Cargo.toml that makes functions public
3. Test at integration boundaries only (recommended for CLI)

Example feature flag approach in `bin/cli/Cargo.toml`:
```toml
[features]
fuzzing = []
```

In `bin/cli/src/normalize.rs`:
```rust
#[cfg_attr(feature = "fuzzing", visibility::make(pub))]
pub(crate) fn normalize_title(input: &str) -> (String, Option<String>) {
    // ...
}
```

### Coverage-Guided Improvements

After initial fuzzing:
1. Run `cargo +nightly fuzz coverage <target>`
2. Generate coverage report: `cargo +nightly fuzz coverage <target> --lcov`
3. Identify uncovered code paths
4. Add corpus entries to hit uncovered branches

### Handling Timeouts

Some parsing functions might be slow on pathological input. Configure:
```bash
cargo +nightly fuzz run <target> -- \
    -max_total_time=300 \
    -timeout=5 \
    -max_len=4096
```

## Verification Plan

### Per-Phase Verification

| Phase | Verification |
|-------|--------------|
| 1 | `cargo +nightly fuzz list` shows all targets |
| 2 | 60s run per target, no crashes |
| 3 | 5min run per protocol target, check coverage |
| 4 | Normalization handles quote edge cases |
| 5 | Structured fuzzing achieves higher coverage |

### Success Criteria

1. **All targets compile** on nightly toolchain
2. **No crashes** after 1 hour of fuzzing per target
3. **Coverage > 80%** for targeted functions

### Crash Handling Process

1. Reproduce crash locally
2. Minimize with `cargo +nightly fuzz tmin`
3. Create failing unit test
4. Fix the bug
5. Add minimized input to corpus as regression test
6. Re-run fuzzer to verify fix

### Metrics to Track

- Total fuzzing time per target
- Corpus size growth
- Coverage percentage
- Crashes found (should trend to 0)
- Executions per second (performance baseline)
