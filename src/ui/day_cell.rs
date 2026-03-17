// SPDX-License-Identifier: MPL-2.0

//! Calendar day cell rendered as mouse_area + container + text so we have
//! full control over foreground color, independent of button theme quirks.

use crate::message::Message;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Background, Border, Color, Length};
use cosmic::widget;

/// Renders a single calendar day cell.
///
/// - `is_selected` → accent background, white text
/// - `is_today`    → subtle tinted border, accent text
/// - normal        → transparent background, theme foreground text
pub fn day_cell<'a>(
    label: impl Into<String>,
    size: f32,
    is_selected: bool,
    is_today: bool,
    on_press: Message,
) -> cosmic::Element<'a, Message> {
    let label = label.into();

    let theme = cosmic::theme::active();
    let cosmic = theme.cosmic();

    let (bg, text_color, border_color, border_width) = if is_selected {
        (
            Background::Color(cosmic.accent.base.into()),
            Color::WHITE,
            Color::TRANSPARENT,
            0.0f32,
        )
    } else if is_today {
        (
            Background::Color(Color {
                r: cosmic.accent.base.red,
                g: cosmic.accent.base.green,
                b: cosmic.accent.base.blue,
                a: 0.18,
            }),
            cosmic.accent.base.into(),
            cosmic.accent.base.into(),
            1.0f32,
        )
    } else {
        (
            Background::Color(Color::TRANSPARENT),
            cosmic.palette.neutral_9.into(),
            Color::TRANSPARENT,
            0.0f32,
        )
    };

    let cell = widget::container(
        widget::text(label)
            .class(cosmic::theme::Text::Color(text_color))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .style(move |_theme| cosmic::iced::widget::container::Style {
        background: Some(bg),
        border: Border {
            color: border_color,
            width: border_width,
            radius: (size / 2.0).into(),
        },
        ..Default::default()
    });

    widget::mouse_area(cell).on_press(on_press).into()
}
