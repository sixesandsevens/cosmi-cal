// SPDX-License-Identifier: MPL-2.0

/// Read the current clipboard text. Returns None on failure or if empty.
pub fn get_text() -> Option<String> {
    let mut cb = arboard::Clipboard::new().ok()?;
    let text = cb.get_text().ok()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Write text to the clipboard. Returns an error message on failure.
pub fn set_text(text: &str) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text.to_string()).map_err(|e| e.to_string())
}
