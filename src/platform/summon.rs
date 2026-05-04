// SPDX-License-Identifier: MPL-2.0

use crate::platform::{detect_session, SessionKind};

pub enum SummonOutcome {
    Handled,
    ContinueStartup,
}

pub fn handle_summon(focus_scratchpad: bool) -> SummonOutcome {
    match detect_session() {
        SessionKind::X11 => crate::platform::x11::summon(focus_scratchpad),
        SessionKind::Wayland | SessionKind::Unknown => {
            crate::platform::generic::summon(focus_scratchpad)
        }
    }
}
