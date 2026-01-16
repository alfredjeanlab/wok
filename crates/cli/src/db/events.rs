// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use rusqlite::params;

use crate::error::Result;
use crate::models::Event;

use super::{parse_db, parse_timestamp, Database};

impl Database {
    /// Log an event
    pub fn log_event(&self, event: &Event) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO events (issue_id, action, old_value, new_value, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.issue_id,
                event.action.as_str(),
                event.old_value,
                event.new_value,
                event.reason,
                event.created_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all events for an issue, ordered by creation time
    pub fn get_events(&self, issue_id: &str) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, action, old_value, new_value, reason, created_at
             FROM events WHERE issue_id = ?1 ORDER BY created_at",
        )?;

        let events = stmt
            .query_map(params![issue_id], |row| {
                let action_str: String = row.get(2)?;
                let created_str: String = row.get(6)?;
                Ok(Event {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    action: parse_db(&action_str, "action")?,
                    old_value: row.get(3)?,
                    new_value: row.get(4)?,
                    reason: row.get(5)?,
                    created_at: parse_timestamp(&created_str, "created_at")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(events)
    }

    /// Get recent events across all issues
    pub fn get_recent_events(&self, limit: usize) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, action, old_value, new_value, reason, created_at
             FROM events ORDER BY created_at DESC LIMIT ?1",
        )?;

        let events = stmt
            .query_map(params![limit as i64], |row| {
                let action_str: String = row.get(2)?;
                let created_str: String = row.get(6)?;
                Ok(Event {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    action: parse_db(&action_str, "action")?,
                    old_value: row.get(3)?,
                    new_value: row.get(4)?,
                    reason: row.get(5)?,
                    created_at: parse_timestamp(&created_str, "created_at")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(events)
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
