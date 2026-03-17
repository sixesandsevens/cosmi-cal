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
mod storage;
mod ui;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    let launch = LaunchPlan::from_args();
    if !launch.commands.is_empty() && ipc::send_commands(&launch.commands).is_ok() {
        return Ok(());
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
}

impl LaunchPlan {
    fn from_args() -> Self {
        let mut plan = Self::default();

        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--summon" => {
                    plan.shell_mode_env = Some("surface".to_string());
                    plan.commands.push(commands::AppCommand::ShowSurface);
                    plan.commands.push(commands::AppCommand::FocusTodayNote);
                }
                "--focus-today" => {
                    plan.shell_mode_env = Some("surface".to_string());
                    plan.commands.push(commands::AppCommand::FocusTodayNote);
                }
                "--focus-scratchpad" => {
                    plan.shell_mode_env = Some("full".to_string());
                    plan.commands.push(commands::AppCommand::FocusScratchpad);
                }
                _ => {}
            }
        }

        plan
    }
}
