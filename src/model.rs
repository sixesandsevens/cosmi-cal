// SPDX-License-Identifier: MPL-2.0

use crate::storage;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub const MAX_CLIPBOARD_HISTORY: usize = 20;

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

    pub fn save(&self) {
        if let Err(e) = storage::save_data(self) {
            eprintln!("Failed to save data: {e}");
        }
    }

    pub fn selected_day_note(&self) -> &str {
        self.day_notes
            .get(&self.selected_date)
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn set_selected_day_note(&mut self, text: String) {
        if text.is_empty() {
            self.day_notes.remove(&self.selected_date);
        } else {
            self.day_notes.insert(self.selected_date.clone(), text);
        }
    }

    /// Adds an item to the front of clipboard history, deduplicating and capping the list.
    pub fn push_clipboard(&mut self, item: String) {
        if item.trim().is_empty() {
            return;
        }
        if self.clipboard_history.front() == Some(&item) {
            return;
        }
        self.clipboard_history.retain(|s| s != &item);
        self.clipboard_history.push_front(item);
        while self.clipboard_history.len() > MAX_CLIPBOARD_HISTORY {
            self.clipboard_history.pop_back();
        }
    }

    pub fn pin_clipboard(&mut self, item: String) {
        if !self.pinned_clipboard.contains(&item) {
            self.pinned_clipboard.push(item);
        }
    }

    pub fn unpin_clipboard(&mut self, item: &str) {
        self.pinned_clipboard.retain(|s| s != item);
    }

    pub fn has_day_note(&self, date: &str) -> bool {
        self.day_notes
            .get(date)
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }
}
