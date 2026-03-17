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
    day_note_content: &'a text_editor::Content,
    window_width: f32,
    day_note_editor_id: widget::Id,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let layout = SurfaceLayout::for_width(window_width, spacing.space_xs, spacing.space_s);

    // ── Mini calendar ─────────────────────────────────────────────────────────
    let mini_cal = mini_calendar(data, cal_year, cal_month, &layout);

    // ── Selected day note ─────────────────────────────────────────────────────
    let today = calendar::today_string();
    let selected = &data.selected_date;
    let note_label = if selected == &today {
        format!("Today · {today}")
    } else {
        format!("Note · {selected}")
    };
    let note_section = widget::column::with_capacity(2)
        .push(widget::text::title4(note_label))
        .push(
            text_editor(day_note_content)
                .id(day_note_editor_id)
                .placeholder("No note for this day yet. Write one…")
                .on_action(Message::DayNoteAction)
                .height(Length::Fixed(layout.note_height)),
        )
        .spacing(layout.gap);

    // ── Clipboard preview ─────────────────────────────────────────────────────
    let mut clip_col = widget::column::with_capacity(8).spacing(layout.gap);

    // Pinned first
    if !data.pinned_clipboard.is_empty() {
        clip_col = clip_col.push(widget::text::title4("Pinned"));
        for item in data.pinned_clipboard.iter().take(layout.clip_limit) {
            clip_col = clip_col.push(dash_clip_row(item, layout.gap));
        }
    }

    // Recent (skipping anything already shown in pinned)
    let recent: Vec<&String> = data
        .clipboard_history
        .iter()
        .filter(|item| !data.pinned_clipboard.contains(item))
        .take(layout.clip_limit)
        .collect();

    clip_col = clip_col.push(widget::text::title4("Recent"));
    if recent.is_empty() {
        clip_col = clip_col.push(widget::text(
            "Nothing copied yet.\nCopy something to see it here.",
        ));
    } else {
        for item in recent {
            clip_col = clip_col.push(dash_clip_row(item, layout.gap));
        }
    }

    // ── Layout ────────────────────────────────────────────────────────────────
    let right_col = widget::column::with_capacity(3)
        .push(note_section)
        .push(widget::divider::horizontal::default())
        .push(clip_col)
        .spacing(layout.section_gap)
        .width(Length::Fill);

    let content: cosmic::Element<'a, Message> = if layout.stack {
        widget::column::with_capacity(3)
            .push(
                widget::container(mini_cal)
                    .width(Length::Fill)
                    .padding(layout.panel_padding),
            )
            .push(widget::divider::horizontal::default())
            .push(
                widget::container(right_col)
                    .width(Length::Fill)
                    .padding(layout.panel_padding),
            )
            .spacing(layout.section_gap)
            .height(Length::Fill)
            .into()
    } else {
        widget::row::with_capacity(3)
            .push(
                widget::container(mini_cal)
                    .width(Length::Fixed(layout.calendar_width))
                    .padding(layout.panel_padding),
            )
            .push(widget::divider::vertical::default())
            .push(
                widget::container(right_col)
                    .width(Length::Fill)
                    .padding(layout.panel_padding),
            )
            .spacing(layout.section_gap)
            .height(Length::Fill)
            .into()
    };

    widget::container(content)
        .padding(layout.outer_padding)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn mini_calendar<'a>(
    data: &'a AppData,
    cal_year: i32,
    cal_month: u32,
    layout: &SurfaceLayout,
) -> cosmic::Element<'a, Message> {
    let month_label = format!("{} {}", calendar::month_name(cal_month), cal_year);

    let today_btn = widget::button::standard("Today")
        .on_press(Message::GoToToday)
        .width(Length::Fill);

    widget::column::with_capacity(5)
        .push(calendar_grid::nav_row(month_label, layout.gap))
        .push(calendar_grid::dow_row(
            layout.calendar_cell,
            layout.day_spacing,
        ))
        .push(calendar_grid::day_grid(
            data,
            cal_year,
            cal_month,
            layout.calendar_cell,
            layout.day_spacing,
        ))
        .push(today_btn)
        .spacing(layout.gap)
        .into()
}

fn dash_clip_row(item: &str, space_xs: u16) -> cosmic::Element<'_, Message> {
    let preview = truncate(item, 40);
    let owned = item.to_string();
    widget::row::with_capacity(2)
        .push(widget::text(preview).width(Length::Fill))
        .push(widget::button::text("Restore").on_press(Message::RestoreClipboard(owned)))
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

#[derive(Clone, Copy)]
struct SurfaceLayout {
    stack: bool,
    clip_limit: usize,
    gap: u16,
    section_gap: u16,
    day_spacing: u16,
    outer_padding: u16,
    panel_padding: u16,
    calendar_width: f32,
    calendar_cell: f32,
    note_height: f32,
}

impl SurfaceLayout {
    fn for_width(window_width: f32, space_xs: u16, space_s: u16) -> Self {
        if window_width < 720.0 {
            Self {
                stack: true,
                clip_limit: 2,
                gap: space_xs,
                section_gap: space_s,
                day_spacing: 2,
                outer_padding: space_s,
                panel_padding: space_s,
                calendar_width: 0.0,
                calendar_cell: 30.0,
                note_height: 132.0,
            }
        } else if window_width < 980.0 {
            Self {
                stack: false,
                clip_limit: 3,
                gap: space_xs,
                section_gap: space_s,
                day_spacing: 3,
                outer_padding: space_s + space_xs,
                panel_padding: space_s,
                calendar_width: 276.0,
                calendar_cell: 32.0,
                note_height: 144.0,
            }
        } else {
            Self {
                stack: false,
                clip_limit: 4,
                gap: space_s,
                section_gap: space_s + space_xs,
                day_spacing: 4,
                outer_padding: space_s + space_xs,
                panel_padding: space_s + space_xs,
                calendar_width: 308.0,
                calendar_cell: 34.0,
                note_height: 160.0,
            }
        }
    }
}
