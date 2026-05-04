// SPDX-License-Identifier: MPL-2.0

use crate::commands::AppCommand;
use crate::ipc;
use crate::platform::summon::SummonOutcome;

pub fn summon(focus_scratchpad: bool) -> SummonOutcome {
    let command = if focus_scratchpad {
        AppCommand::SummonScratchpad
    } else {
        AppCommand::SummonToggle
    };

    match ipc::forward_commands(&[command]) {
        Ok(()) => SummonOutcome::Handled,
        Err(_) => SummonOutcome::ContinueStartup,
    }
}
