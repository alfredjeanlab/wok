// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

// Runner tests require database integration and are tested via BATS specs.
// Unit tests here focus on the logic that can be tested in isolation.

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn runner_module_exports() {
    // Verify run_hooks_for_event is exported
    let _: fn(
        &crate::db::Database,
        &std::path::Path,
        &crate::models::Event,
    ) -> crate::error::Result<()> = run_hooks_for_event;
}
