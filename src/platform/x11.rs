// SPDX-License-Identifier: MPL-2.0

use crate::commands::AppCommand;
use crate::ipc;
use crate::platform::summon::SummonOutcome;
use std::process::Command;

const WINDOW_CLASSES: &[&str] = &["cosmi-cal", "io.github.sixesandsevens.cosmical"];

pub fn summon(focus_scratchpad: bool) -> SummonOutcome {
    let Some(window_id) = running_window_id() else {
        return SummonOutcome::ContinueStartup;
    };

    if is_cosmical_focused() {
        if minimize_active_window() || forward_toggle_command(focus_scratchpad) {
            return SummonOutcome::Handled;
        }

        return SummonOutcome::ContinueStartup;
    }

    let focused = focus_window(&window_id);
    let forwarded = forward_show_commands(focus_scratchpad);
    if focused || forwarded {
        SummonOutcome::Handled
    } else {
        SummonOutcome::ContinueStartup
    }
}

fn running_window_id() -> Option<String> {
    let Ok(out) = Command::new("wmctrl").args(["-lx"]).output() else {
        return None;
    };

    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(matching_window_id)
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

fn matching_window_id(line: &str) -> Option<String> {
    let mut fields = line.split_whitespace();
    let window_id = fields.next()?;
    let _desktop = fields.next()?;
    let _host = fields.next()?;
    let class = fields.next()?.to_lowercase();

    WINDOW_CLASSES
        .iter()
        .any(|needle| class.contains(needle))
        .then(|| window_id.to_string())
}

fn focus_window(window_id: &str) -> bool {
    Command::new("wmctrl")
        .args(["-ia", window_id])
        .status()
        .is_ok_and(|status| status.success())
}

fn minimize_active_window() -> bool {
    Command::new("xdotool")
        .args(["getactivewindow", "windowminimize"])
        .status()
        .is_ok_and(|status| status.success())
}

fn forward_show_commands(focus_scratchpad: bool) -> bool {
    let mut commands = vec![AppCommand::ShowSurface];
    if focus_scratchpad {
        commands.push(AppCommand::FocusScratchpad);
    } else {
        commands.push(AppCommand::FocusTodayNote);
    }

    if let Err(err) = ipc::forward_commands(&commands) {
        eprintln!("failed to forward summon command to CosmiCal: {err}");
        false
    } else {
        true
    }
}

fn forward_toggle_command(focus_scratchpad: bool) -> bool {
    let command = if focus_scratchpad {
        AppCommand::SummonScratchpad
    } else {
        AppCommand::SummonToggle
    };

    if let Err(err) = ipc::forward_commands(&[command]) {
        eprintln!("failed to forward summon command to CosmiCal: {err}");
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::matching_window_id;

    #[test]
    fn matches_packaged_window_class_from_wmctrl() {
        let line = "0x03a00003  0 host io.github.sixesandsevens.cosmical.CosmiCal CosmiCal";

        assert_eq!(matching_window_id(line).as_deref(), Some("0x03a00003"));
    }

    #[test]
    fn matches_development_window_class_from_wmctrl() {
        let line = "0x03a00003  0 host cosmi-cal.cosmi-cal CosmiCal";

        assert_eq!(matching_window_id(line).as_deref(), Some("0x03a00003"));
    }

    #[test]
    fn ignores_title_only_matches_from_wmctrl() {
        let line = "0x03a00003  0 host org.example.Other io.github.sixesandsevens.cosmical";

        assert_eq!(matching_window_id(line), None);
    }
}
