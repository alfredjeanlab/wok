// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use yare::parameterized;

/// Mock clock for testing with controllable time.
struct MockClock {
    time_ms: AtomicU64,
}

impl MockClock {
    fn new(initial_ms: u64) -> Self {
        MockClock { time_ms: AtomicU64::new(initial_ms) }
    }

    fn set(&self, ms: u64) {
        self.time_ms.store(ms, AtomicOrdering::SeqCst);
    }

    fn advance(&self, ms: u64) {
        self.time_ms.fetch_add(ms, AtomicOrdering::SeqCst);
    }
}

impl ClockSource for MockClock {
    fn now_ms(&self) -> u64 {
        self.time_ms.load(AtomicOrdering::SeqCst)
    }
}

#[test]
fn hlc_ordering() {
    // Higher wall_ms wins
    let a = Hlc::new(100, 0, 0);
    let b = Hlc::new(200, 0, 0);
    assert!(b > a);

    // Same wall_ms, higher counter wins
    let a = Hlc::new(100, 1, 0);
    let b = Hlc::new(100, 2, 0);
    assert!(b > a);

    // Same wall_ms and counter, higher node_id wins
    let a = Hlc::new(100, 1, 1);
    let b = Hlc::new(100, 1, 2);
    assert!(b > a);
}

#[test]
fn hlc_equality() {
    let a = Hlc::new(100, 1, 42);
    let b = Hlc::new(100, 1, 42);
    assert_eq!(a, b);
}

#[test]
fn hlc_parse_roundtrip() {
    let original = Hlc::new(1234567890, 42, 99);
    let s = original.to_string();
    let parsed: Hlc = s.parse().unwrap();
    assert_eq!(original, parsed);
}

#[parameterized(
    invalid_word = { "invalid" },
    two_parts = { "1-2" },
    four_parts = { "1-2-3-4" },
    bad_wall = { "abc-2-3" },
    bad_counter = { "1-abc-3" },
    bad_node = { "1-2-abc" },
)]
fn hlc_parse_errors(input: &str) {
    assert!(input.parse::<Hlc>().is_err());
}

#[test]
fn hlc_min() {
    let min = Hlc::min();
    assert_eq!(min.wall_ms, 0);
    assert_eq!(min.counter, 0);
    assert_eq!(min.node_id, 0);

    let any = Hlc::new(1, 0, 0);
    assert!(any > min);
}

#[test]
fn hlc_is_after_is_before() {
    let a = Hlc::new(100, 0, 0);
    let b = Hlc::new(200, 0, 0);

    assert!(b.is_after(&a));
    assert!(!a.is_after(&b));
    assert!(a.is_before(&b));
    assert!(!b.is_before(&a));
}

#[test]
fn hlc_clock_monotonic() {
    let clock = MockClock::new(1000);
    let hlc = HlcClock::with_clock(&clock, 42);

    let t1 = hlc.now();
    let t2 = hlc.now();
    let t3 = hlc.now();

    assert!(t2 > t1);
    assert!(t3 > t2);
    assert_eq!(t1.node_id, 42);
}

#[test]
fn hlc_clock_time_advances() {
    let clock = MockClock::new(1000);
    let hlc = HlcClock::with_clock(&clock, 1);

    let t1 = hlc.now();
    assert_eq!(t1.wall_ms, 1000);
    assert_eq!(t1.counter, 0);

    clock.advance(100);
    let t2 = hlc.now();
    assert_eq!(t2.wall_ms, 1100);
    assert_eq!(t2.counter, 0);
    assert!(t2 > t1);
}

#[test]
fn hlc_clock_time_goes_backwards() {
    let clock = MockClock::new(2000);
    let hlc = HlcClock::with_clock(&clock, 1);

    let t1 = hlc.now();
    assert_eq!(t1.wall_ms, 2000);
    assert_eq!(t1.counter, 0);

    // Time goes backwards
    clock.set(1000);
    let t2 = hlc.now();
    // Should maintain wall_ms and increment counter
    assert_eq!(t2.wall_ms, 2000);
    assert_eq!(t2.counter, 1);
    assert!(t2 > t1);
}

#[test]
fn hlc_clock_receive_future() {
    let clock = MockClock::new(1000);
    let hlc = HlcClock::with_clock(&clock, 1);

    // Receive a timestamp from the future
    let future = Hlc::new(5000, 10, 2);
    let t1 = hlc.receive(&future);

    // Should adopt the future time
    assert_eq!(t1.wall_ms, 5000);
    assert_eq!(t1.counter, 11); // future.counter + 1
    assert!(t1 > future);
}

#[test]
fn hlc_clock_receive_past() {
    let clock = MockClock::new(5000);
    let hlc = HlcClock::with_clock(&clock, 1);

    let _ = hlc.now(); // Set last_ms to 5000

    // Receive a timestamp from the past
    let past = Hlc::new(1000, 10, 2);
    let t1 = hlc.receive(&past);

    // Should keep our time and increment counter
    assert_eq!(t1.wall_ms, 5000);
    assert!(t1 > past);
}

#[test]
fn hlc_clock_receive_same_time() {
    let clock = MockClock::new(1000);
    let hlc = HlcClock::with_clock(&clock, 1);

    let received = Hlc::new(1000, 5, 2);
    let t1 = hlc.receive(&received);

    // Should have same wall_ms but higher counter
    assert_eq!(t1.wall_ms, 1000);
    assert!(t1.counter > received.counter);
    assert!(t1 > received);
}

#[test]
fn hlc_serialization() {
    let hlc = Hlc::new(12345, 67, 89);
    let json = serde_json::to_string(&hlc).unwrap();
    let parsed: Hlc = serde_json::from_str(&json).unwrap();
    assert_eq!(hlc, parsed);
}

#[test]
fn system_clock_returns_reasonable_time() {
    let clock = SystemClock;
    let now = clock.now_ms();
    // Should be after Jan 1, 2020 (1577836800000 ms)
    assert!(now > 1_577_836_800_000);
}

#[test]
fn clock_source_ref_delegation() {
    // Test that ClockSource impl for &C delegates correctly
    let clock = MockClock::new(42000);
    let clock_ref: &MockClock = &clock;

    // Both should return the same time
    assert_eq!(clock.now_ms(), 42000);
    assert_eq!(clock_ref.now_ms(), 42000);

    // Changing the underlying clock affects the reference
    clock.set(99000);
    assert_eq!(clock_ref.now_ms(), 99000);
}

#[test]
fn hlc_parse_convenience_method() {
    // Test Hlc::parse() method which delegates to FromStr
    let hlc = Hlc::parse("12345-67-89").unwrap();
    assert_eq!(hlc.wall_ms, 12345);
    assert_eq!(hlc.counter, 67);
    assert_eq!(hlc.node_id, 89);

    // Also test error case
    let err = Hlc::parse("invalid");
    assert!(err.is_err());
}

#[test]
fn hlc_clock_receive_our_time_ahead() {
    // Test the branch where our last_ms is ahead of both physical clock and received
    let clock = MockClock::new(5000);
    let hlc = HlcClock::with_clock(&clock, 1);

    // Generate a timestamp to set last_ms to 5000
    let _ = hlc.now();

    // Now set clock backwards
    clock.set(1000);

    // Receive a message also from the past (before our last_ms)
    let received = Hlc::new(2000, 5, 2);
    let result = hlc.receive(&received);

    // Our last_ms (5000) should be maintained, with incremented counter
    assert_eq!(result.wall_ms, 5000);
    // Counter should have been incremented from 0
    assert!(result.counter >= 1);
    assert!(result > received);
}

#[test]
fn hlc_clock_node_id() {
    let clock = MockClock::new(1000);
    let hlc = HlcClock::with_clock(&clock, 42);
    assert_eq!(hlc.node_id(), 42);
}

#[test]
fn hlc_clock_new_with_system_clock() {
    let hlc = HlcClock::new(99);
    assert_eq!(hlc.node_id(), 99);

    // Should generate valid timestamps
    let t = hlc.now();
    assert_eq!(t.node_id, 99);
    // Time should be reasonable (after Jan 1, 2020)
    assert!(t.wall_ms > 1_577_836_800_000);
}
