// SPDX-License-Identifier: MPL-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    ShowSurface,
    FocusTodayNote,
    FocusScratchpad,
}

impl AppCommand {
    pub fn as_wire(&self) -> &'static str {
        match self {
            Self::ShowSurface => "show_surface",
            Self::FocusTodayNote => "focus_today_note",
            Self::FocusScratchpad => "focus_scratchpad",
        }
    }

    pub fn from_wire(input: &str) -> Option<Self> {
        match input.trim() {
            "show_surface" => Some(Self::ShowSurface),
            "focus_today_note" => Some(Self::FocusTodayNote),
            "focus_scratchpad" => Some(Self::FocusScratchpad),
            _ => None,
        }
    }
}
