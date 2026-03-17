// SPDX-License-Identifier: MPL-2.0

use crate::commands::AppCommand;
use crate::config::Config;
use cosmic::widget::text_editor;

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

    // Day note (calendar page and dashboard editor)
    DayNoteAction(text_editor::Action),

    // Scratchpad
    ScratchpadAction(text_editor::Action),

    // Clipboard
    ClipboardTick,
    ClipboardRead(Option<String>),
    RestoreClipboard(String),
    PinClipboard(String),
    UnpinClipboard(String),
    ClearClipboardHistory,
    AppCommand(AppCommand),

    // Persistence
    SaveTick,
    DateRolloverTick,
}
