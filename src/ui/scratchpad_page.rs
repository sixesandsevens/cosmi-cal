// SPDX-License-Identifier: MPL-2.0

use crate::fl;
use crate::message::Message;
use crate::model::AppData;
use cosmic::iced::Length;
use cosmic::widget;

pub fn view<'a>(data: &'a AppData, dirty: bool) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_xs = spacing.space_xs;

    let dirty_indicator = if dirty { " ●" } else { "" };
    let header = widget::text::title3(format!("{}{}", fl!("nav-scratchpad"), dirty_indicator));

    let placeholder = "Start writing… your thoughts live here.";
    let editor = widget::text_input(placeholder, data.scratchpad.as_str())
        .on_input(Message::ScratchpadChanged)
        .width(Length::Fill);

    widget::column::with_capacity(3)
        .push(header)
        .push(widget::Space::new().height(Length::Fixed(space_xs as f32)))
        .push(editor)
        .spacing(space_s)
        .padding(space_s)
        .height(Length::Fill)
        .into()
}
