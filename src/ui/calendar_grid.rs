// SPDX-License-Identifier: MPL-2.0

//! Shared calendar grid builder used by both the full calendar page and the
//! mini calendar on the dashboard.

use crate::calendar;
use crate::message::Message;
use crate::model::AppData;
use crate::ui::day_cell::day_cell;
use chrono::Datelike;
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::{Alignment, Length};
use cosmic::widget;

/// Renders a compact month navigation row (‹ body-text label ›).
/// Used by the mini calendar on the dashboard.
pub fn nav_row<'a>(month_label: String, space: u16) -> cosmic::Element<'a, Message> {
    widget::row::with_capacity(3)
        .push(widget::button::standard("‹").on_press(Message::PrevMonth))
        .push(
            widget::text::body(month_label)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .push(widget::button::standard("›").on_press(Message::NextMonth))
        .align_y(Alignment::Center)
        .spacing(space)
        .into()
}

/// Renders a full-size month navigation row (‹ title4 label ›).
/// Used by the main calendar page.
pub fn nav_row_title<'a>(month_label: String, space: u16) -> cosmic::Element<'a, Message> {
    widget::row::with_capacity(3)
        .push(widget::button::standard("‹").on_press(Message::PrevMonth))
        .push(
            widget::text::title4(month_label)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .push(widget::button::standard("›").on_press(Message::NextMonth))
        .align_y(Alignment::Center)
        .spacing(space)
        .into()
}

/// Renders the Mon–Sun day-of-week header row.
pub fn dow_row<'a>(cell_size: f32, spacing: u16) -> cosmic::Element<'a, Message> {
    let labels = ["M", "T", "W", "T", "F", "S", "S"];
    let mut row = widget::row::with_capacity(7).spacing(spacing);
    for label in &labels {
        row = row.push(
            widget::text(*label)
                .width(Length::Fixed(cell_size))
                .align_x(Horizontal::Center),
        );
    }
    row.into()
}

/// Renders the full day grid for a given month.
pub fn day_grid<'a>(
    data: &'a AppData,
    year: i32,
    month: u32,
    cell_size: f32,
    spacing: u16,
) -> cosmic::Element<'a, Message> {
    let today = calendar::today_string();
    let days = calendar::days_in_month(year, month);
    let mut leading = calendar::month_start_weekday(year, month);

    let mut grid = widget::column::with_capacity(6).spacing(spacing);
    let mut day_iter = days.iter().peekable();

    while day_iter.peek().is_some() {
        let mut row = widget::row::with_capacity(7).spacing(spacing);

        for cell in 0..7usize {
            if leading > 0 && cell < leading {
                row = row.push(widget::Space::new().width(Length::Fixed(cell_size)));
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
                        cell_size,
                        is_selected,
                        is_today,
                        Message::SelectDate(ds),
                    ));
                } else {
                    row = row.push(widget::Space::new().width(Length::Fixed(cell_size)));
                }
            }
        }

        grid = grid.push(row);
    }

    grid.into()
}
