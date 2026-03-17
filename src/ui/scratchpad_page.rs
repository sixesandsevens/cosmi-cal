// SPDX-License-Identifier: MPL-2.0

use crate::fl;
use crate::message::Message;
use cosmic::iced::Length;
use cosmic::widget::{self, text_editor};

pub fn view<'a>(
    content: &'a text_editor::Content,
    editor_id: widget::Id,
    dirty: bool,
    save_error: bool,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;

    let status = if save_error {
        " ⚠ save failed"
    } else if dirty {
        " ●"
    } else {
        ""
    };
    let header = widget::text::title3(format!("{}{}", fl!("nav-scratchpad"), status));

    let editor = text_editor(content)
        .id(editor_id)
        .placeholder("Start writing… your thoughts live here.")
        .on_action(Message::ScratchpadAction)
        .height(Length::Fill);

    widget::column::with_capacity(2)
        .push(header)
        .push(editor)
        .spacing(space_s)
        .padding(space_s)
        .height(Length::Fill)
        .into()
}
