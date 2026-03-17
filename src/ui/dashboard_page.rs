// SPDX-License-Identifier: MPL-2.0

//! Dashboard — a single-glance summary of calendar, today's note, and
//! recent clipboard items. This is the first sketch of the "desktop surface"
//! end-goal: all the important information at a glance, no tab switching.

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
    let space_m = spacing.space_m;

    // ── Mini calendar ─────────────────────────────────────────────────────────
    let mini_cal = mini_calendar(data, cal_year, cal_month, space_xs);

    // ── Today's note ──────────────────────────────────────────────────────────
    let today = calendar::today_string();
    let today_note_text = data.day_notes.get(&today).map(String::as_str).unwrap_or("");
    let note_section = widget::column::with_capacity(2)
        .push(widget::text::title4(format!("Today · {today}")))
        .push(
            widget::text_input("No note for today yet. Write one…", today_note_text)
                .on_input(Message::DayNoteChanged)
                .width(Length::Fill),
        )
        .spacing(space_xs);

    // ── Clipboard preview ─────────────────────────────────────────────────────
    let mut clip_col = widget::column::with_capacity(6).spacing(space_xs);
    clip_col = clip_col.push(widget::text::title4("Recent Clipboard"));

    if data.clipboard_history.is_empty() {
        clip_col = clip_col.push(widget::text("Nothing copied yet."));
    } else {
        for item in data.clipboard_history.iter().take(5) {
            let preview = truncate(item, 48);
            let owned = item.clone();
            clip_col = clip_col.push(
                widget::row::with_capacity(2)
                    .push(widget::text(preview).width(Length::Fill))
                    .push(
                        widget::button::standard("Restore")
                            .on_press(Message::RestoreClipboard(owned)),
                    )
                    .align_y(Alignment::Center)
                    .spacing(space_xs),
            );
        }
    }

    // ── Layout: calendar left, note + clipboard right ─────────────────────────
    let right_col = widget::column::with_capacity(3)
        .push(note_section)
        .push(widget::Space::new().height(Length::Fixed(space_m as f32)))
        .push(clip_col)
        .spacing(space_s)
        .width(Length::Fill);

    widget::row::with_capacity(3)
        .push(
            widget::container(mini_cal)
                .width(Length::Fixed(290.0))
                .padding(space_s),
        )
        .push(widget::divider::vertical::default())
        .push(
            widget::container(right_col)
                .width(Length::Fill)
                .padding(space_s),
        )
        .spacing(space_s)
        .height(Length::Fill)
        .into()
}

fn mini_calendar<'a>(
    data: &'a AppData,
    cal_year: i32,
    cal_month: u32,
    space_xs: u16,
) -> cosmic::Element<'a, Message> {
    let today = calendar::today_string();
    let days = calendar::days_in_month(cal_year, cal_month);
    let start_weekday = calendar::month_start_weekday(cal_year, cal_month);
    let month_label = format!("{} {}", calendar::month_name(cal_month), cal_year);

    const CELL: f32 = 30.0;

    let nav_row = widget::row::with_capacity(3)
        .push(widget::button::standard("‹").on_press(Message::PrevMonth))
        .push(
            widget::text::body(month_label)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .push(widget::button::standard("›").on_press(Message::NextMonth))
        .align_y(Alignment::Center)
        .spacing(space_xs);

    let dow_labels = ["M", "T", "W", "T", "F", "S", "S"];
    let mut dow_row = widget::row::with_capacity(7).spacing(2);
    for label in &dow_labels {
        dow_row = dow_row.push(
            widget::text(*label)
                .width(Length::Fixed(CELL))
                .align_x(Horizontal::Center),
        );
    }

    let mut grid = widget::column::with_capacity(6).spacing(2);
    let mut day_iter = days.iter().peekable();
    let mut leading = start_weekday;

    while day_iter.peek().is_some() {
        let mut row = widget::row::with_capacity(7).spacing(2);

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

    let today_btn = widget::button::standard("Today")
        .on_press(Message::GoToToday)
        .width(Length::Fill);

    widget::column::with_capacity(5)
        .push(nav_row)
        .push(dow_row)
        .push(grid)
        .push(today_btn)
        .spacing(space_xs)
        .into()
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max_chars)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}…", &s[..end])
    }
}
