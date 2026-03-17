// SPDX-License-Identifier: MPL-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    SummonToggle,
    ShowSurface,
    DismissSurface,
    FocusTodayNote,
    FocusScratchpad,
}

impl AppCommand {
    pub fn as_wire(&self) -> &'static str {
        match self {
            Self::SummonToggle => "summon_toggle",
            Self::ShowSurface => "show_surface",
            Self::DismissSurface => "dismiss_surface",
            Self::FocusTodayNote => "focus_today_note",
            Self::FocusScratchpad => "focus_scratchpad",
        }
    }

    pub fn from_wire(input: &str) -> Option<Self> {
        match input.trim() {
            "summon_toggle" => Some(Self::SummonToggle),
            "show_surface" => Some(Self::ShowSurface),
            "dismiss_surface" => Some(Self::DismissSurface),
            "focus_today_note" => Some(Self::FocusTodayNote),
            "focus_scratchpad" => Some(Self::FocusScratchpad),
            _ => None,
        }
    }
}
