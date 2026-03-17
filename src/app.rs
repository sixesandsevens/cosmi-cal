// SPDX-License-Identifier: MPL-2.0

use crate::calendar;
use crate::clipboard as cb;
use crate::config::Config;
use crate::fl;
use crate::model::AppData;
use chrono::Datelike;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::widget::{self, about::About, icon, menu, nav_bar};
use cosmic::{iced_futures, prelude::*};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::time::Duration;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    about: About,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    /// Application data (notes, clipboard history, scratchpad).
    data: AppData,
    /// The year currently shown in the calendar.
    cal_year: i32,
    /// The month currently shown in the calendar (1–12).
    cal_month: u32,
    /// Most recent clipboard text seen by the poller, used to debounce.
    last_clipboard: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Nav / meta
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),

    // Calendar
    PrevMonth,
    NextMonth,
    SelectDate(String),

    // Day note
    DayNoteChanged(String),

    // Scratchpad
    ScratchpadChanged(String),

    // Clipboard
    ClipboardTick,
    RestoreClipboard(String),
    PinClipboard(String),
    UnpinClipboard(String),

    // Persistence (reserved for explicit save triggers)
    #[allow(dead_code)]
    Save,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "dev.sixesandsevens.cosmi-cal";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text(fl!("nav-calendar"))
            .data::<Page>(Page::Calendar)
            .icon(icon::from_name("x-office-calendar-symbolic"))
            .activate();

        nav.insert()
            .text(fl!("nav-scratchpad"))
            .data::<Page>(Page::Scratchpad)
            .icon(icon::from_name("document-edit-symbolic"));

        nav.insert()
            .text(fl!("nav-clipboard"))
            .data::<Page>(Page::Clipboard)
            .icon(icon::from_name("edit-paste-symbolic"));

        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        let data = AppData::load();
        let today = calendar::today_string();
        // Parse stored selected date for the calendar view; fall back to today.
        let (cal_year, cal_month) = parse_ym(&data.selected_date)
            .or_else(|| parse_ym(&today))
            .unwrap_or_else(|| {
                let now = chrono::Local::now();
                (now.year(), now.month())
            });

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|ctx| match Config::get_entry(&ctx) {
                    Ok(cfg) => cfg,
                    Err((_, cfg)) => cfg,
                })
                .unwrap_or_default(),
            data,
            cal_year,
            cal_month,
            last_clipboard: None,
        };

        let command = app.update_title();
        (app, command)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);
        vec![menu_bar.into()]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }
        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<_> = match self.nav.active_data::<Page>() {
            Some(Page::Calendar) => self.view_calendar(),
            Some(Page::Scratchpad) => self.view_scratchpad(),
            Some(Page::Clipboard) => self.view_clipboard(),
            None => widget::text("No page selected").into(),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let config_watch = self
            .core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config));

        // Poll the clipboard every 2 seconds.
        let clipboard_poll = Subscription::run(|| {
            iced_futures::stream::channel(
                1,
                |mut emitter: iced_futures::futures::channel::mpsc::Sender<Message>| async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(2));
                    loop {
                        interval.tick().await;
                        let _ = emitter.send(Message::ClipboardTick).await;
                    }
                },
            )
        });

        Subscription::batch(vec![config_watch, clipboard_poll])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => {
                if let Err(e) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {e}");
                }
            }

            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
            }

            Message::PrevMonth => {
                (self.cal_year, self.cal_month) =
                    calendar::prev_month(self.cal_year, self.cal_month);
            }

            Message::NextMonth => {
                (self.cal_year, self.cal_month) =
                    calendar::next_month(self.cal_year, self.cal_month);
            }

            Message::SelectDate(date) => {
                self.data.selected_date = date;
            }

            Message::DayNoteChanged(text) => {
                self.data.set_selected_day_note(text);
                self.data.save();
            }

            Message::ScratchpadChanged(text) => {
                self.data.scratchpad = text;
                self.data.save();
            }

            Message::ClipboardTick => {
                if let Some(text) = cb::get_text() {
                    if self.last_clipboard.as_deref() != Some(text.as_str()) {
                        self.last_clipboard = Some(text.clone());
                        self.data.push_clipboard(text);
                        self.data.save();
                    }
                }
            }

            Message::RestoreClipboard(text) => {
                if let Err(e) = cb::set_text(&text) {
                    eprintln!("clipboard write failed: {e}");
                }
            }

            Message::PinClipboard(text) => {
                self.data.pin_clipboard(text);
                self.data.save();
            }

            Message::UnpinClipboard(text) => {
                self.data.unpin_clipboard(&text);
                self.data.save();
            }

            Message::Save => {
                self.data.save();
            }
        }
        Task::none()
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        self.update_title()
    }
}

// ── View helpers ─────────────────────────────────────────────────────────────

impl AppModel {
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");
        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    fn view_calendar(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing();
        let space_s = spacing.space_s;
        let space_xs = spacing.space_xs;

        let today = calendar::today_string();
        let days = calendar::days_in_month(self.cal_year, self.cal_month);
        let start_weekday = calendar::month_start_weekday(self.cal_year, self.cal_month);
        let month_label = format!(
            "{} {}",
            calendar::month_name(self.cal_month),
            self.cal_year
        );

        // Header row: prev / month+year / next
        let nav_row = widget::row::with_capacity(3)
            .push(
                widget::button::text("‹")
                    .on_press(Message::PrevMonth),
            )
            .push(
                widget::text::title4(month_label)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
            )
            .push(
                widget::button::text("›")
                    .on_press(Message::NextMonth),
            )
            .align_y(Alignment::Center)
            .spacing(space_s);

        // Day-of-week header
        let dow_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
        let mut dow_row = widget::row::with_capacity(7).spacing(space_xs);
        for label in &dow_labels {
            dow_row = dow_row.push(
                widget::text(*label)
                    .width(Length::Fixed(36.0))
                    .align_x(Horizontal::Center),
            );
        }

        // Build the grid: 6 rows × 7 cols
        let mut grid = widget::column::with_capacity(6).spacing(space_xs);
        let mut day_iter = days.iter().peekable();
        let mut col = start_weekday;

        while day_iter.peek().is_some() {
            let mut row = widget::row::with_capacity(7).spacing(space_xs);

            for cell in 0..7usize {
                if col > 0 && day_iter.peek().is_none().not() && cell == 0 {
                    // Still in leading blank region — shouldn't happen here, handled below
                }
                if (day_iter.peek().is_none()) || (col > 0 && cell < col) {
                    // Leading blank
                    row = row.push(widget::Space::new().width(Length::Fixed(36.0)));
                } else {
                    col = 0; // leading blanks exhausted
                    if let Some(date) = day_iter.next() {
                        let ds = calendar::date_string(*date);
                        let is_today = ds == today;
                        let is_selected = ds == self.data.selected_date;
                        let has_note = self.data.has_day_note(&ds);

                        let day_num = date.day().to_string();
                        let label = if has_note {
                            format!("{day_num}·")
                        } else {
                            day_num
                        };

                        let btn = if is_selected {
                            widget::button::suggested(label)
                                .on_press(Message::SelectDate(ds))
                                .width(Length::Fixed(36.0))
                        } else if is_today {
                            widget::button::standard(label)
                                .on_press(Message::SelectDate(ds))
                                .width(Length::Fixed(36.0))
                        } else {
                            widget::button::text(label)
                                .on_press(Message::SelectDate(ds))
                                .width(Length::Fixed(36.0))
                        };

                        row = row.push(btn);
                    } else {
                        row = row.push(widget::Space::new().width(Length::Fixed(36.0)));
                    }
                }
            }

            grid = grid.push(row);
        }

        // Day note editor
        let note_header = widget::text::title4(format!("Note — {}", self.data.selected_date));
        let note_editor = widget::text_input("Write a note for this day…", self.data.selected_day_note())
            .on_input(Message::DayNoteChanged)
            .width(Length::Fill);

        widget::column::with_capacity(5)
            .push(nav_row)
            .push(dow_row)
            .push(grid)
            .push(widget::divider::horizontal::default())
            .push(note_header)
            .push(note_editor)
            .spacing(space_s)
            .padding(space_s)
            .into()
    }

    fn view_scratchpad(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing();
        let space_s = spacing.space_s;

        let header = widget::text::title3(fl!("nav-scratchpad"));
        let editor = widget::text_input("Start writing…", self.data.scratchpad.as_str())
            .on_input(Message::ScratchpadChanged)
            .width(Length::Fill);

        widget::column::with_capacity(2)
            .push(header)
            .push(editor)
            .spacing(space_s)
            .padding(space_s)
            .height(Length::Fill)
            .into()
    }

    fn view_clipboard(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing();
        let space_s = spacing.space_s;
        let space_xs = spacing.space_xs;

        let header = widget::text::title3(fl!("nav-clipboard"));

        // Pinned section
        let mut pinned_col = widget::column::with_capacity(self.data.pinned_clipboard.len() + 1)
            .spacing(space_xs);
        if !self.data.pinned_clipboard.is_empty() {
            pinned_col = pinned_col.push(widget::text::title4("Pinned"));
            for item in &self.data.pinned_clipboard {
                let preview = truncate(item, 60);
                let row = widget::row::with_capacity(3)
                    .push(
                        widget::text(preview)
                            .width(Length::Fill),
                    )
                    .push(
                        widget::button::text("Restore")
                            .on_press(Message::RestoreClipboard(item.clone())),
                    )
                    .push(
                        widget::button::text("Unpin")
                            .on_press(Message::UnpinClipboard(item.clone())),
                    )
                    .align_y(Alignment::Center)
                    .spacing(space_xs);
                pinned_col = pinned_col.push(row);
            }
        }

        // History section
        let mut history_col =
            widget::column::with_capacity(self.data.clipboard_history.len() + 1).spacing(space_xs);
        history_col = history_col.push(widget::text::title4("Recent"));
        if self.data.clipboard_history.is_empty() {
            history_col = history_col.push(widget::text("No clipboard history yet."));
        } else {
            for item in &self.data.clipboard_history {
                let preview = truncate(item, 60);
                let row = widget::row::with_capacity(3)
                    .push(widget::text(preview).width(Length::Fill))
                    .push(
                        widget::button::text("Restore")
                            .on_press(Message::RestoreClipboard(item.clone())),
                    )
                    .push(
                        widget::button::text("Pin")
                            .on_press(Message::PinClipboard(item.clone())),
                    )
                    .align_y(Alignment::Center)
                    .spacing(space_xs);
                history_col = history_col.push(row);
            }
        }

        widget::column::with_capacity(4)
            .push(header)
            .push(pinned_col)
            .push(widget::divider::horizontal::default())
            .push(history_col)
            .spacing(space_s)
            .padding(space_s)
            .height(Length::Fill)
            .into()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_ym(date: &str) -> Option<(i32, u32)> {
    let parts: Vec<&str> = date.splitn(3, '-').collect();
    if parts.len() < 2 {
        return None;
    }
    let year = parts[0].parse::<i32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    if month >= 1 && month <= 12 {
        Some((year, month))
    } else {
        None
    }
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

trait BoolExt {
    fn not(self) -> bool;
}
impl BoolExt for bool {
    fn not(self) -> bool {
        !self
    }
}

// ── Page / context enums ──────────────────────────────────────────────────────

pub enum Page {
    Calendar,
    Scratchpad,
    Clipboard,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
