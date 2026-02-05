// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn module_exports_correct_types() {
    // Verify all expected types are exported
    let _: fn(&std::path::Path) -> crate::error::Result<Option<HooksConfig>> = load_hooks_config;
    let _: fn(&HookConfig, &HookPayload, &std::path::Path) -> crate::error::Result<()> =
        execute_hook;
}
