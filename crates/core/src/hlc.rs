// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hybrid Logical Clock (HLC) for distributed ordering.
//!
//! HLC combines wall clock time with a logical counter to provide causally
//! consistent timestamps even in the presence of clock skew.
//!
//! Format: `{wall_ms}-{counter}-{node_id}`
//!
//! Ordering rules:
//! 1. Higher wall_ms wins
//! 2. If wall_ms equal, higher counter wins
//! 3. If both equal, higher node_id wins (deterministic tiebreaker)

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};

/// A Hybrid Logical Clock timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hlc {
    /// Wall clock time in milliseconds since Unix epoch.
    pub wall_ms: u64,
    /// Logical counter for ordering events at the same wall time.
    pub counter: u32,
    /// Node identifier for deterministic tiebreaking.
    pub node_id: u32,
}

impl Hlc {
    /// Creates a new HLC with the given components.
    pub fn new(wall_ms: u64, counter: u32, node_id: u32) -> Self {
        Hlc { wall_ms, counter, node_id }
    }

    /// Creates an HLC representing the earliest possible time (for queries).
    pub fn min() -> Self {
        Hlc { wall_ms: 0, counter: 0, node_id: 0 }
    }

    /// Parses an HLC from its string representation.
    pub fn parse(s: &str) -> Result<Self> {
        s.parse()
    }

    /// Returns true if this HLC is strictly greater than the other.
    pub fn is_after(&self, other: &Hlc) -> bool {
        self > other
    }

    /// Returns true if this HLC is strictly less than the other.
    pub fn is_before(&self, other: &Hlc) -> bool {
        self < other
    }
}

impl Ord for Hlc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.wall_ms
            .cmp(&other.wall_ms)
            .then_with(|| self.counter.cmp(&other.counter))
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

impl PartialOrd for Hlc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.wall_ms, self.counter, self.node_id)
    }
}

impl FromStr for Hlc {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(Error::InvalidHlc(format!(
                "expected format 'wall_ms-counter-node_id', got '{s}'"
            )));
        }

        let wall_ms = parts[0]
            .parse::<u64>()
            .map_err(|_| Error::InvalidHlc(format!("invalid wall_ms '{}' in '{s}'", parts[0])))?;

        let counter = parts[1]
            .parse::<u32>()
            .map_err(|_| Error::InvalidHlc(format!("invalid counter '{}' in '{s}'", parts[1])))?;

        let node_id = parts[2]
            .parse::<u32>()
            .map_err(|_| Error::InvalidHlc(format!("invalid node_id '{}' in '{s}'", parts[2])))?;

        Ok(Hlc::new(wall_ms, counter, node_id))
    }
}

/// Trait for getting the current wall clock time.
///
/// This allows injecting a mock clock for testing.
pub trait ClockSource: Send + Sync {
    /// Returns the current time in milliseconds since Unix epoch.
    fn now_ms(&self) -> u64;
}

/// System clock implementation using `std::time::SystemTime`.
#[derive(Debug, Default)]
pub struct SystemClock;

impl ClockSource for SystemClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
    }
}

impl<C: ClockSource> ClockSource for &C {
    fn now_ms(&self) -> u64 {
        (*self).now_ms()
    }
}

/// A clock generator that produces monotonically increasing HLC timestamps.
///
/// Thread-safe and handles clock skew by advancing the logical counter
/// when receiving timestamps from the future.
pub struct HlcClock<C: ClockSource = SystemClock> {
    clock: C,
    node_id: u32,
    last_wall_ms: Mutex<u64>,
    last_counter: AtomicU32,
}

impl HlcClock<SystemClock> {
    /// Creates a new HLC clock with the system clock and given node ID.
    pub fn new(node_id: u32) -> Self {
        Self::with_clock(SystemClock, node_id)
    }
}

impl<C: ClockSource> HlcClock<C> {
    /// Creates a new HLC clock with a custom clock source.
    pub fn with_clock(clock: C, node_id: u32) -> Self {
        HlcClock { clock, node_id, last_wall_ms: Mutex::new(0), last_counter: AtomicU32::new(0) }
    }

    /// Returns the node ID for this clock.
    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    /// Generates a new HLC timestamp.
    ///
    /// Guarantees monotonically increasing timestamps even if the wall clock
    /// goes backwards.
    pub fn now(&self) -> Hlc {
        let physical = self.clock.now_ms();
        let mut last_ms = self.last_wall_ms.lock().unwrap_or_else(|e| e.into_inner());

        let (wall_ms, counter) = if physical > *last_ms {
            // Normal case: wall clock advanced
            *last_ms = physical;
            self.last_counter.store(0, AtomicOrdering::SeqCst);
            (physical, 0)
        } else {
            // Clock went backwards or stayed same: increment counter
            let counter = self.last_counter.fetch_add(1, AtomicOrdering::SeqCst) + 1;
            (*last_ms, counter)
        };

        Hlc::new(wall_ms, counter, self.node_id)
    }

    /// Updates the clock based on a received HLC timestamp.
    ///
    /// This ensures causality: any timestamp generated after receiving
    /// a message will be greater than the received timestamp.
    pub fn receive(&self, received: &Hlc) -> Hlc {
        let physical = self.clock.now_ms();
        let mut last_ms = self.last_wall_ms.lock().unwrap_or_else(|e| e.into_inner());

        let (wall_ms, counter) = if physical > *last_ms && physical > received.wall_ms {
            // Our physical clock is ahead of everything
            *last_ms = physical;
            self.last_counter.store(0, AtomicOrdering::SeqCst);
            (physical, 0)
        } else if received.wall_ms > *last_ms {
            // Received timestamp is ahead: adopt its wall time
            *last_ms = received.wall_ms;
            let counter = received.counter + 1;
            self.last_counter.store(counter, AtomicOrdering::SeqCst);
            (received.wall_ms, counter)
        } else if received.wall_ms == *last_ms {
            // Same wall time: increment counter past received
            let our_counter = self.last_counter.load(AtomicOrdering::SeqCst);
            let counter = our_counter.max(received.counter) + 1;
            self.last_counter.store(counter, AtomicOrdering::SeqCst);
            (*last_ms, counter)
        } else {
            // Our last time is ahead: just increment our counter
            let counter = self.last_counter.fetch_add(1, AtomicOrdering::SeqCst) + 1;
            (*last_ms, counter)
        };

        Hlc::new(wall_ms, counter, self.node_id)
    }
}

#[cfg(test)]
#[path = "hlc_tests.rs"]
mod tests;
