// SPDX-License-Identifier: MPL-2.0

use crate::calendar;
use crate::message::Message;
use crate::model::AppData;
use crate::ui::day_cell::day_cell;
use chrono::Datelike;
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::{Alignment, Length};
use cosmic::widget;

pub fn view<'a>(data: &'a AppData, cal_year: i32, cal_month: u32) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_xs = spacing.space_xs;

    let today = calendar::today_string();
    let days = calendar::days_in_month(cal_year, cal_month);
    let start_weekday = calendar::month_start_weekday(cal_year, cal_month);
    let month_label = format!("{} {}", calendar::month_name(cal_month), cal_year);

    // ── Month navigation ──────────────────────────────────────────────────────
    let nav_row = widget::row::with_capacity(3)
        .push(widget::button::standard("‹").on_press(Message::PrevMonth))
        .push(
            widget::text::title4(month_label)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .push(widget::button::standard("›").on_press(Message::NextMonth))
        .align_y(Alignment::Center)
        .spacing(space_s);

    // ── Day-of-week header ────────────────────────────────────────────────────
    let dow_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let mut dow_row = widget::row::with_capacity(7).spacing(space_xs);
    for label in &dow_labels {
        dow_row = dow_row.push(
            widget::text(*label)
                .width(Length::Fixed(36.0))
                .align_x(Horizontal::Center),
        );
    }

    // ── Day grid ──────────────────────────────────────────────────────────────
    const CELL: f32 = 36.0;
    let mut grid = widget::column::with_capacity(6).spacing(space_xs);
    let mut day_iter = days.iter().peekable();
    let mut leading = start_weekday;

    while day_iter.peek().is_some() {
        let mut row = widget::row::with_capacity(7).spacing(space_xs);

        for cell in 0..7usize {
            if leading > 0 && cell < leading {
                row = row.push(widget::Space::new().width(Length::Fixed(CELL)));
            } else {
                leading = 0;
                if let Some(date) = day_iter.next() {
                    let ds = calendar::date_string(*date);
                    let is_today = ds == today;
                    let is_selected = ds == data.selected_date;
                    let has_note = data.has_day_note(&ds);
                    let day_num = date.day().to_string();
                    let label = if has_note {
                        format!("{day_num}·")
                    } else {
                        day_num
                    };
                    row = row.push(day_cell(
                        label,
                        CELL,
                        is_selected,
                        is_today,
                        Message::SelectDate(ds),
                    ));
                } else {
                    row = row.push(widget::Space::new().width(Length::Fixed(CELL)));
                }
            }
        }

        grid = grid.push(row);
    }

    // ── Today button ──────────────────────────────────────────────────────────
    let today_btn = widget::button::standard("Today").on_press(Message::GoToToday);

    // ── Day note ──────────────────────────────────────────────────────────────
    let note_header = widget::text::title4(format!("Note — {}", data.selected_date));
    let note_placeholder = if data.selected_day_note().is_empty() {
        "Nothing yet. Add a note for this day…"
    } else {
        ""
    };
    let note_editor = widget::text_input(note_placeholder, data.selected_day_note())
        .on_input(Message::DayNoteChanged)
        .width(Length::Fill);

    widget::column::with_capacity(7)
        .push(nav_row)
        .push(dow_row)
        .push(grid)
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
