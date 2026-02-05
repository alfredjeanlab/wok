// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Event name mapping for issue hooks.
//!
//! Maps the core `Action` enum to hook event names like "issue.created".

use crate::models::Action;

/// Event types for hooks, derived from the core Action enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    Created,
    Edited,
    Started,
    Stopped,
    Done,
    Closed,
    Reopened,
    Labeled,
    Unlabeled,
    Assigned,
    Unassigned,
    Noted,
    Linked,
    Unlinked,
    Related,
    Unrelated,
    Unblocked,
}

impl HookEvent {
    /// Get the event name as used in config (e.g., "issue.created").
    pub fn as_event_name(&self) -> &'static str {
        match self {
            HookEvent::Created => "issue.created",
            HookEvent::Edited => "issue.edited",
            HookEvent::Started => "issue.started",
            HookEvent::Stopped => "issue.stopped",
            HookEvent::Done => "issue.done",
            HookEvent::Closed => "issue.closed",
            HookEvent::Reopened => "issue.reopened",
            HookEvent::Labeled => "issue.labeled",
            HookEvent::Unlabeled => "issue.unlabeled",
            HookEvent::Assigned => "issue.assigned",
            HookEvent::Unassigned => "issue.unassigned",
            HookEvent::Noted => "issue.noted",
            HookEvent::Linked => "issue.linked",
            HookEvent::Unlinked => "issue.unlinked",
            HookEvent::Related => "issue.related",
            HookEvent::Unrelated => "issue.unrelated",
            HookEvent::Unblocked => "issue.unblocked",
        }
    }

    /// Check if a pattern matches this event.
    ///
    /// Supports exact matches and the "issue.*" wildcard.
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        pattern == "issue.*" || pattern == self.as_event_name()
    }
}

impl From<Action> for HookEvent {
    fn from(action: Action) -> Self {
        match action {
            Action::Created => HookEvent::Created,
            Action::Edited => HookEvent::Edited,
            Action::Started => HookEvent::Started,
            Action::Stopped => HookEvent::Stopped,
            Action::Done => HookEvent::Done,
            Action::Closed => HookEvent::Closed,
            Action::Reopened => HookEvent::Reopened,
            Action::Labeled => HookEvent::Labeled,
            Action::Unlabeled => HookEvent::Unlabeled,
            Action::Assigned => HookEvent::Assigned,
            Action::Unassigned => HookEvent::Unassigned,
            Action::Noted => HookEvent::Noted,
            Action::Linked => HookEvent::Linked,
            Action::Unlinked => HookEvent::Unlinked,
            Action::Related => HookEvent::Related,
            Action::Unrelated => HookEvent::Unrelated,
            Action::Unblocked => HookEvent::Unblocked,
        }
    }
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
