// SPDX-License-Identifier: MPL-2.0

use crate::calendar;
use crate::message::Message;
use crate::model::AppData;
use crate::ui::calendar_grid;
use cosmic::iced::Length;
use cosmic::widget::{self, text_editor};

pub fn view<'a>(
    data: &'a AppData,
    cal_year: i32,
    cal_month: u32,
    note_content: &'a text_editor::Content,
    day_note_editor_id: widget::Id,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_xs = spacing.space_xs;

    let month_label = format!("{} {}", calendar::month_name(cal_month), cal_year);

    // ── Month navigation ──────────────────────────────────────────────────────
    let nav = calendar_grid::nav_row_title(month_label, space_s);

    const CELL: f32 = 36.0;

    // ── Today button ──────────────────────────────────────────────────────────
    let today_btn = widget::button::standard("Today").on_press(Message::GoToToday);

    // ── Day note ──────────────────────────────────────────────────────────────
    let today = calendar::today_string();
    let note_label = if data.selected_date == today {
        format!("Note — {} · Today", data.selected_date)
    } else if data.selected_date > today {
        format!("Note — {} · Upcoming", data.selected_date)
    } else {
        format!("Note — {}", data.selected_date)
    };
    let note_header = widget::text::title4(note_label);

    let note_editor = text_editor(note_content)
        .id(day_note_editor_id)
        .placeholder("No note for this day yet. Select a day and start typing.")
        .on_action(Message::DayNoteAction)
        .height(Length::Fixed(160.0));

    widget::column::with_capacity(7)
        .push(nav)
        .push(calendar_grid::dow_row(CELL, space_xs))
        .push(calendar_grid::day_grid(
            data, cal_year, cal_month, CELL, space_xs,
        ))
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
