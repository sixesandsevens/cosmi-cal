// SPDX-License-Identifier: MPL-2.0

//! Dashboard — a single-glance summary of calendar, today's note, and
//! recent clipboard items. This is the first sketch of the "desktop surface"
//! end-goal: all the important information at a glance, no tab switching.

use crate::calendar;
use crate::message::Message;
use crate::model::AppData;
use crate::ui::calendar_grid;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, text_editor};

pub fn view<'a>(
    data: &'a AppData,
    cal_year: i32,
    cal_month: u32,
    today_note_content: &'a text_editor::Content,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_xs = spacing.space_xs;

    // ── Mini calendar ─────────────────────────────────────────────────────────
    let mini_cal = mini_calendar(data, cal_year, cal_month, space_xs);

    // ── Today's note ──────────────────────────────────────────────────────────
    let today = calendar::today_string();
    let note_section = widget::column::with_capacity(2)
        .push(widget::text::title4(format!("Today · {today}")))
        .push(
            text_editor(today_note_content)
                .placeholder("No note for today yet. Write one…")
                .on_action(Message::TodayNoteAction)
                .height(Length::Fixed(140.0)),
        )
        .spacing(space_xs);

    // ── Clipboard preview ─────────────────────────────────────────────────────
    let mut clip_col = widget::column::with_capacity(8).spacing(space_xs);

    // Pinned first (up to 3)
    if !data.pinned_clipboard.is_empty() {
        clip_col = clip_col.push(widget::text::title4("Pinned"));
        for item in data.pinned_clipboard.iter().take(3) {
            clip_col = clip_col.push(dash_clip_row(item, space_xs));
        }
    }

    // Recent (up to 3, skipping anything already shown in pinned)
    let recent: Vec<&String> = data
        .clipboard_history
        .iter()
        .filter(|item| !data.pinned_clipboard.contains(item))
        .take(3)
        .collect();

    clip_col = clip_col.push(widget::text::title4("Recent"));
    if recent.is_empty() {
        clip_col = clip_col.push(
            widget::text("Nothing copied yet.\nCopy something to see it here."),
        );
    } else {
        for item in recent {
            clip_col = clip_col.push(dash_clip_row(item, space_xs));
        }
    }

    // ── Layout: calendar left, note + clipboard right ─────────────────────────
    let right_col = widget::column::with_capacity(3)
        .push(note_section)
        .push(widget::divider::horizontal::default())
        .push(clip_col)
        .spacing(space_s)
        .width(Length::Fill);

    widget::row::with_capacity(3)
        .push(
            widget::container(mini_cal)
                .width(Length::Fixed(260.0))
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
    let month_label = format!("{} {}", calendar::month_name(cal_month), cal_year);

    const CELL: f32 = 30.0;

    let today_btn = widget::button::standard("Today")
        .on_press(Message::GoToToday)
        .width(Length::Fill);

    widget::column::with_capacity(5)
        .push(calendar_grid::nav_row(month_label, space_xs))
        .push(calendar_grid::dow_row(CELL, 2))
        .push(calendar_grid::day_grid(data, cal_year, cal_month, CELL, 2))
        .push(today_btn)
        .spacing(space_xs)
        .into()
}

fn dash_clip_row(item: &str, space_xs: u16) -> cosmic::Element<'_, Message> {
    let preview = truncate(item, 40);
    let owned = item.to_string();
    widget::row::with_capacity(2)
        .push(widget::text(preview).width(Length::Fill))
        .push(
            widget::button::text("Restore")
                .on_press(Message::RestoreClipboard(owned)),
        )
        .align_y(Alignment::Center)
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
