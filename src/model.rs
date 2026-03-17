// SPDX-License-Identifier: MPL-2.0

use crate::storage;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub const MAX_CLIPBOARD_HISTORY: usize = 20;
/// Ignore clipboard entries larger than this many bytes.
pub const MAX_CLIPBOARD_BYTES: usize = 20_480; // 20 KB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    /// The currently selected date in "YYYY-MM-DD" format.
    pub selected_date: String,
    /// Freeform scratchpad text.
    pub scratchpad: String,
    /// Per-day notes keyed by "YYYY-MM-DD".
    pub day_notes: BTreeMap<String, String>,
    /// Recent clipboard entries, newest first.
    pub clipboard_history: VecDeque<String>,
    /// Pinned clipboard entries.
    pub pinned_clipboard: Vec<String>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            selected_date: crate::calendar::today_string(),
            scratchpad: String::new(),
            day_notes: BTreeMap::new(),
            clipboard_history: VecDeque::new(),
            pinned_clipboard: Vec::new(),
        }
    }
}

impl AppData {
    pub fn load() -> Self {
        storage::load_data().unwrap_or_default()
    }

    /// Returns `true` on success, `false` on failure.
    pub fn save(&self) -> bool {
        match storage::save_data(self) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("Failed to save data: {e}");
                false
            }
        }
    }

    pub fn set_day_note(&mut self, date: String, text: String) {
        if text.is_empty() {
            self.day_notes.remove(&date);
        } else {
            self.day_notes.insert(date, text);
        }
    }

    /// Adds an item to clipboard history with deduplication, size, and cap guardrails.
    pub fn push_clipboard(&mut self, item: String) -> bool {
        // Ignore oversized entries
        if item.len() > MAX_CLIPBOARD_BYTES {
            return false;
        }
        // Ignore blank / whitespace-only
        if item.trim().is_empty() {
            return false;
        }
        // Already at the front — nothing to do
        if self.clipboard_history.front() == Some(&item) {
            return false;
        }
        // Deduplicate: remove existing occurrence before re-inserting at front
        self.clipboard_history.retain(|s| s != &item);
        self.clipboard_history.push_front(item);
        // Cap history length
        while self.clipboard_history.len() > MAX_CLIPBOARD_HISTORY {
            self.clipboard_history.pop_back();
        }
        true
    }

    pub fn pin_clipboard(&mut self, item: String) {
        if !self.pinned_clipboard.contains(&item) {
            self.pinned_clipboard.push(item);
        }
    }

    pub fn unpin_clipboard(&mut self, item: &str) {
        self.pinned_clipboard.retain(|s| s != item);
    }

    pub fn clear_clipboard_history(&mut self) {
        self.clipboard_history.clear();
    }

    pub fn has_day_note(&self, date: &str) -> bool {
        self.day_notes
            .get(date)
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }
}
