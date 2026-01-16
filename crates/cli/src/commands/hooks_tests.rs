// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use wk_core::hooks::HookScope;

#[test]
fn scope_parsing() {
    assert_eq!(HookScope::parse("local"), Some(HookScope::Local));
    assert_eq!(HookScope::parse("project"), Some(HookScope::Project));
    assert_eq!(HookScope::parse("user"), Some(HookScope::User));
    assert_eq!(HookScope::parse("invalid"), None);
}

#[test]
fn scope_case_insensitive() {
    assert_eq!(HookScope::parse("LOCAL"), Some(HookScope::Local));
    assert_eq!(HookScope::parse("Project"), Some(HookScope::Project));
    assert_eq!(HookScope::parse("USER"), Some(HookScope::User));
}
