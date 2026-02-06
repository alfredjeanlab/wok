// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::error::Result;
use crate::models::Event;

use super::Database;

impl Database {
    /// Log an event
    pub fn log_event(&self, event: &Event) -> Result<i64> {
        let core_event: wk_core::Event = event.clone().into();
        Ok(self.0.log_event(&core_event)?)
    }

    /// Get all events for an issue, ordered by creation time
    pub fn get_events(&self, issue_id: &str) -> Result<Vec<Event>> {
        let events = self.0.get_events(issue_id)?;
        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    /// Get recent events across all issues
    pub fn get_recent_events(&self, limit: usize) -> Result<Vec<Event>> {
        let events = self.0.get_recent_events(limit)?;
        Ok(events.into_iter().map(|e| e.into()).collect())
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
