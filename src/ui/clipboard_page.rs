// SPDX-License-Identifier: MPL-2.0

use crate::fl;
use crate::message::Message;
use crate::model::AppData;
use cosmic::iced::{Alignment, Length};
use cosmic::widget;

pub fn view<'a>(data: &'a AppData) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_xs = spacing.space_xs;

    let header = widget::text::title3(fl!("nav-clipboard"));

    // ── Pinned ────────────────────────────────────────────────────────────────
    let mut pinned_col =
        widget::column::with_capacity(data.pinned_clipboard.len() + 1).spacing(space_xs);

    if !data.pinned_clipboard.is_empty() {
        pinned_col = pinned_col.push(widget::text::title4("Pinned — always here"));
        for item in &data.pinned_clipboard {
            pinned_col = pinned_col.push(clipboard_row(item, true, space_xs));
        }
        pinned_col = pinned_col.push(widget::divider::horizontal::default());
    }

    // ── Recent ────────────────────────────────────────────────────────────────
    let mut history_col =
        widget::column::with_capacity(data.clipboard_history.len() + 2).spacing(space_xs);

    let recent_header = widget::row::with_capacity(2)
        .push(widget::text::title4("Recent").width(Length::Fill))
        .push(widget::button::text("Clear").on_press(Message::ClearClipboardHistory))
        .align_y(Alignment::Center);

    history_col = history_col.push(recent_header);

    if data.clipboard_history.is_empty() {
        history_col = history_col.push(widget::text("No clipboard history yet. Copy something!"));
    } else {
        for item in &data.clipboard_history {
            history_col = history_col.push(clipboard_row(item, false, space_xs));
        }
    }

    widget::column::with_capacity(3)
        .push(header)
        .push(pinned_col)
        .push(history_col)
        .spacing(space_s)
        .padding(space_s)
        .height(Length::Fill)
        .into()
}

fn clipboard_row(item: &str, pinned: bool, space_xs: u16) -> cosmic::Element<'_, Message> {
    let preview = truncate(item, 60);
    let owned = item.to_string();

    let action_btn = if pinned {
        widget::button::text("Unpin").on_press(Message::UnpinClipboard(owned.clone()))
    } else {
        widget::button::text("Pin").on_press(Message::PinClipboard(owned.clone()))
    };

    widget::row::with_capacity(3)
        .push(widget::text(preview).width(Length::Fill))
        .push(widget::button::text("Restore").on_press(Message::RestoreClipboard(owned)))
        .push(action_btn)
        .align_y(cosmic::iced::Alignment::Center)
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
