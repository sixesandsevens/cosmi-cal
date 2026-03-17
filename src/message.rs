// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Message {
    // Nav / meta
    LaunchUrl(String),
    ToggleContextPage(crate::app::ContextPage),
    UpdateConfig(Config),

    // Calendar
    PrevMonth,
    NextMonth,
    SelectDate(String),
    GoToToday,

    // Day note
    DayNoteChanged(String),

    // Scratchpad
    ScratchpadChanged(String),

    // Clipboard
    ClipboardTick,
    RestoreClipboard(String),
    PinClipboard(String),
    UnpinClipboard(String),
    ClearClipboardHistory,

    // Persistence
    SaveTick,
}
