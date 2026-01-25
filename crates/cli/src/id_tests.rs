// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use chrono::TimeZone;
use yare::parameterized;

#[test]
fn test_generate_id() {
    let created_at = Utc.with_ymd_and_hms(2024, 1, 10, 12, 0, 0).unwrap();
    let id = generate_id("prj", "Test issue", &created_at);
    assert!(id.starts_with("prj-"));
    assert_eq!(id.len(), 12); // prj- + 8 hex chars
}

#[test]
fn test_generate_unique_id_no_collision() {
    let created_at = Utc.with_ymd_and_hms(2024, 1, 10, 12, 0, 0).unwrap();
    let base_id = generate_id("prj", "Test", &created_at);
    let id = generate_unique_id("prj", "Test", &created_at, |_| false);
    // When no collision, the unique id should equal the base id
    assert_eq!(id, base_id);
}

#[test]
fn test_generate_unique_id_with_collision() {
    let created_at = Utc::now();
    let base_id = generate_id("prj", "Test", &created_at);

    let id = generate_unique_id("prj", "Test", &created_at, |id| id == base_id);
    assert!(id.ends_with("-2"));
}

#[test]
fn test_generate_unique_id_multiple_collisions() {
    let created_at = Utc::now();
    let base_id = generate_id("prj", "Test", &created_at);
    let collision_2 = format!("{}-2", base_id);
    let collision_3 = format!("{}-3", base_id);

    // Simulate collisions for base, -2, and -3
    let id = generate_unique_id("prj", "Test", &created_at, |id| {
        id == base_id || id == collision_2 || id == collision_3
    });

    assert!(id.ends_with("-4"));
}

#[test]
fn test_generate_unique_id_collision_loop() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let created_at = Utc::now();

    // Use atomic counter since Fn requires shared access
    let call_count = AtomicUsize::new(0);
    let id = generate_unique_id("test", "Collision", &created_at, |_candidate| {
        let count = call_count.fetch_add(1, Ordering::SeqCst);
        // First 5 calls return true (collision), then false
        count < 5
    });

    // With 5 collisions (base, -2, -3, -4, -5), we should get -6
    assert!(id.ends_with("-6"));
}

// Valid prefixes
#[parameterized(
    two_chars = { "ab" },
    three_chars = { "prj" },
    four_chars = { "auth" },
    with_digit = { "a1" },
    digit_first = { "v0" },
    digits_in_middle = { "proj123" },
)]
fn test_validate_prefix_valid(prefix: &str) {
    assert!(validate_prefix(prefix), "'{}' should be valid", prefix);
}

// Invalid prefixes
#[parameterized(
    too_short = { "a" },
    uppercase = { "AB" },
    only_digits = { "12" },
    contains_hyphen = { "a-b" },
    empty = { "" },
)]
fn test_validate_prefix_invalid(prefix: &str) {
    assert!(!validate_prefix(prefix), "'{}' should be invalid", prefix);
}
