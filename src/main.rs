// SPDX-License-Identifier: MPL-2.0

mod app;
mod calendar;
mod commands;
mod config;
mod focus;
mod i18n;
mod ipc;
mod message;
mod model;
mod platform;
mod storage;
mod ui;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    let launch = LaunchPlan::from_args();
    if launch.summon {
        match platform::summon::handle_summon(launch.focus_scratchpad) {
            platform::summon::SummonOutcome::Handled => return Ok(()),
            platform::summon::SummonOutcome::ContinueStartup => {}
        }
    }

    let startup_commands = launch.startup_commands();
    match ipc::prepare_startup(&startup_commands) {
        Ok(ipc::StartupAction::ForwardedToPrimary) => return Ok(()),
        Ok(ipc::StartupAction::StartPrimary) => {}
        Err(err) => {
            eprintln!("IPC startup check failed: {err}");
        }
    }

    if let Some(shell_mode) = launch.shell_mode_env.as_deref() {
        if std::env::var_os("COSMICAL_MODE").is_none() {
            unsafe {
                std::env::set_var("COSMICAL_MODE", shell_mode);
            }
        }
    }

    const SURFACE_WIDTH: f32 = 920.0;
    const SURFACE_HEIGHT: f32 = 560.0;
    const SURFACE_MIN_WIDTH: f32 = 540.0;
    const SURFACE_MIN_HEIGHT: f32 = 420.0;

    let mode = std::env::var("COSMICAL_MODE");
    let settings = match mode.as_deref() {
        Ok("full") => cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(960.0, 720.0))
            .size_limits(
                cosmic::iced::Limits::NONE
                    .min_width(480.0)
                    .min_height(320.0),
            ),
        Ok("dashboard") => {
            // Compact utility window, no OS titlebar.
            cosmic::app::Settings::default()
                .size(cosmic::iced::Size::new(SURFACE_WIDTH, SURFACE_HEIGHT))
                .size_limits(
                    cosmic::iced::Limits::NONE
                        .min_width(SURFACE_MIN_WIDTH)
                        .min_height(SURFACE_MIN_HEIGHT),
                )
                .client_decorations(false)
                .resizable(Some(4.0))
        }
        Ok("surface") | Ok(_) | Err(_) => {
            // Surface is the everyday default: a wide, calm utility window with
            // as little chrome as the compositor will allow.
            // client_decorations(false) asks the compositor to strip OS decorations;
            // whether it honours that depends on the compositor.
            cosmic::app::Settings::default()
                .size(cosmic::iced::Size::new(SURFACE_WIDTH, SURFACE_HEIGHT))
                .size_limits(
                    cosmic::iced::Limits::NONE
                        .min_width(SURFACE_MIN_WIDTH)
                        .min_height(SURFACE_MIN_HEIGHT),
                )
                .client_decorations(false)
                .resizable(None)
        }
    };

    cosmic::app::run::<app::AppModel>(settings, app::LaunchFlags::new(launch.commands))
}

#[derive(Default)]
struct LaunchPlan {
    commands: Vec<commands::AppCommand>,
    shell_mode_env: Option<String>,
    summon: bool,
    focus_scratchpad: bool,
}

impl LaunchPlan {
    fn from_args() -> Self {
        Self::from_iter(std::env::args().skip(1))
    }

    fn from_iter(args: impl IntoIterator<Item = String>) -> Self {
        let mut plan = Self::default();
        let mut summon = false;
        let mut focus_today = false;
        let mut focus_scratchpad = false;

        for arg in args {
            match arg.as_str() {
                "--summon" => {
                    summon = true;
                }
                "--focus-today" => {
                    focus_today = true;
                }
                "--focus-scratchpad" => {
                    focus_scratchpad = true;
                }
                _ => {}
            }
        }

        if summon {
            plan.summon = true;
            plan.shell_mode_env = Some("surface".to_string());
            if focus_scratchpad {
                plan.focus_scratchpad = true;
                plan.commands.push(commands::AppCommand::SummonScratchpad);
            } else {
                plan.commands.push(commands::AppCommand::SummonToggle);
            }
        } else {
            if focus_today {
                plan.shell_mode_env = Some("surface".to_string());
                plan.commands.push(commands::AppCommand::FocusTodayNote);
            }
            if focus_scratchpad {
                plan.focus_scratchpad = true;
                plan.shell_mode_env = Some("surface".to_string());
                plan.commands.push(commands::AppCommand::FocusScratchpad);
            }
        }

        plan
    }

    fn startup_commands(&self) -> Vec<commands::AppCommand> {
        if self.commands.is_empty() {
            vec![
                commands::AppCommand::ShowSurface,
                commands::AppCommand::FocusTodayNote,
            ]
        } else {
            self.commands.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LaunchPlan;
    use crate::commands::AppCommand;

    #[test]
    fn summon_scratchpad_uses_toggle_command() {
        let plan = launch_plan_from(["cosmi-cal", "--summon", "--focus-scratchpad"]);

        assert_eq!(plan.shell_mode_env.as_deref(), Some("surface"));
        assert_eq!(plan.commands, vec![AppCommand::SummonScratchpad]);
    }

    #[test]
    fn summon_uses_toggle_command() {
        let plan = launch_plan_from(["cosmi-cal", "--summon"]);

        assert_eq!(plan.shell_mode_env.as_deref(), Some("surface"));
        assert_eq!(plan.commands, vec![AppCommand::SummonToggle]);
    }

    #[test]
    fn plain_scratchpad_focus_uses_surface_mode() {
        let plan = launch_plan_from(["cosmi-cal", "--focus-scratchpad"]);

        assert_eq!(plan.shell_mode_env.as_deref(), Some("surface"));
        assert_eq!(plan.commands, vec![AppCommand::FocusScratchpad]);
    }

    fn launch_plan_from<'a>(args: impl IntoIterator<Item = &'a str>) -> LaunchPlan {
        LaunchPlan::from_iter(args.into_iter().skip(1).map(str::to_string))
    }
}
