// SPDX-License-Identifier: MPL-2.0

use crate::commands::AppCommand;
use crate::ipc;
use crate::platform::summon::SummonOutcome;
use std::process::Command;

const WINDOW_CLASSES: &[&str] = &["cosmi-cal", "io.github.sixesandsevens.cosmical"];

pub fn summon(focus_scratchpad: bool) -> SummonOutcome {
    if !is_running_window_present() {
        return SummonOutcome::ContinueStartup;
    }

    if is_cosmical_focused() {
        minimize_active_window();
        return SummonOutcome::Handled;
    }

    focus_existing_window();
    forward_show_commands(focus_scratchpad);
    SummonOutcome::Handled
}

fn is_running_window_present() -> bool {
    let Ok(out) = Command::new("wmctrl").args(["-lx"]).output() else {
        return false;
    };

    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    WINDOW_CLASSES.iter().any(|class| text.contains(*class))
}

fn is_cosmical_focused() -> bool {
    let Ok(out) = Command::new("xdotool")
        .args(["getactivewindow", "getwindowclassname"])
        .output()
    else {
        return false;
    };

    let class = String::from_utf8_lossy(&out.stdout);
    WINDOW_CLASSES
        .iter()
        .any(|needle| class.trim().eq_ignore_ascii_case(needle))
}

fn focus_existing_window() {
    for class in WINDOW_CLASSES {
        if Command::new("wmctrl")
            .args(["-x", "-a", *class])
            .status()
            .is_ok_and(|status| status.success())
        {
            return;
        }
    }
}

fn minimize_active_window() {
    let _ = Command::new("xdotool")
        .args(["getactivewindow", "windowminimize"])
        .status();
}

fn forward_show_commands(focus_scratchpad: bool) {
    let mut commands = vec![AppCommand::ShowSurface];
    if focus_scratchpad {
        commands.push(AppCommand::FocusScratchpad);
    } else {
        commands.push(AppCommand::FocusTodayNote);
    }

    if let Err(err) = ipc::forward_commands(&commands) {
        eprintln!("failed to forward summon command to CosmiCal: {err}");
    }
}
