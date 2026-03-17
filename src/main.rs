// SPDX-License-Identifier: MPL-2.0

mod app;
mod calendar;
mod config;
mod i18n;
mod message;
mod model;
mod storage;
mod ui;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    let dashboard_only = std::env::var("COSMICAL_MODE").as_deref() == Ok("dashboard");

    let settings = if dashboard_only {
        // Utility-window posture: compact, no OS decorations, fixed default size.
        cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(820.0, 520.0))
            .size_limits(
                cosmic::iced::Limits::NONE
                    .min_width(360.0)
                    .min_height(240.0),
            )
            .client_decorations(false)
            .resizable(Some(4.0))
    } else {
        cosmic::app::Settings::default().size_limits(
            cosmic::iced::Limits::NONE
                .min_width(360.0)
                .min_height(180.0),
        )
    };

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, ())
}
