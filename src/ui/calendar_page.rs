// SPDX-License-Identifier: MPL-2.0

use crate::calendar;
use crate::message::Message;
use crate::model::AppData;
use crate::ui::calendar_grid;
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::{Alignment, Length};
use cosmic::widget;

pub fn view<'a>(data: &'a AppData, cal_year: i32, cal_month: u32) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_xs = spacing.space_xs;

    let month_label = format!("{} {}", calendar::month_name(cal_month), cal_year);

    // ── Month navigation ──────────────────────────────────────────────────────
    let nav = widget::row::with_capacity(3)
        .push(widget::button::standard("‹").on_press(Message::PrevMonth))
        .push(
            widget::text::title4(month_label)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .push(widget::button::standard("›").on_press(Message::NextMonth))
        .align_y(Alignment::Center)
        .spacing(space_s);

    const CELL: f32 = 36.0;

    // ── Today button ──────────────────────────────────────────────────────────
    let today_btn = widget::button::standard("Today").on_press(Message::GoToToday);

    // ── Day note ──────────────────────────────────────────────────────────────
    let note_header = widget::text::title4(format!("Note — {}", data.selected_date));
    let note_placeholder = if data.selected_day_note().is_empty() {
        "Nothing yet. Add a note for this day…"
    } else {
        ""
    };
    let selected = data.selected_date.clone();
    let note_editor = widget::text_input(note_placeholder, data.selected_day_note())
        .on_input(move |t| Message::SetDayNote { date: selected.clone(), text: t })
        .width(Length::Fill);

    widget::column::with_capacity(7)
        .push(nav)
        .push(calendar_grid::dow_row(CELL, space_xs))
        .push(calendar_grid::day_grid(data, cal_year, cal_month, CELL, space_xs))
        .push(
            widget::row::with_capacity(1)
                .push(widget::Space::new().width(Length::Fill))
                .push(today_btn),
        )
        .push(widget::divider::horizontal::default())
        .push(note_header)
        .push(note_editor)
        .spacing(space_s)
        .padding(space_s)
        .into()
}
