// SPDX-License-Identifier: MPL-2.0

use crate::calendar;
use crate::clipboard as cb;
use crate::config::Config;
use crate::fl;
use crate::message::Message;
use crate::model::AppData;
use crate::ui;
use chrono::Datelike;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::Subscription;
use cosmic::widget::{self, about::About, icon, menu, nav_bar, text_editor};
use cosmic::{iced_futures, prelude::*};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::time::Duration;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// How many save-ticks of idle time before we flush to disk.
/// Each SaveTick fires every 200 ms, so 3 ticks ≈ 600 ms of idle.
const SAVE_IDLE_TICKS: u8 = 3;

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    about: About,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    pub data: AppData,
    pub cal_year: i32,
    pub cal_month: u32,
    /// Last clipboard text seen, used to debounce polling.
    last_clipboard: Option<String>,
    /// True when data has changed since the last save.
    dirty: bool,
    /// Counts down from SAVE_IDLE_TICKS after a change; saves when it hits 0.
    save_countdown: u8,
    /// Multiline editor state for the scratchpad.
    pub scratchpad_content: text_editor::Content,
    /// Multiline editor state for the calendar selected-day note.
    pub day_note_content: text_editor::Content,
    /// Multiline editor state for the dashboard today note.
    pub today_note_content: text_editor::Content,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.sixesandsevens.cosmical";

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
            .text(fl!("nav-dashboard"))
            .data::<Page>(Page::Dashboard)
            .icon(icon::from_name("view-grid-symbolic"))
            .activate();

        nav.insert()
            .text(fl!("nav-calendar"))
            .data::<Page>(Page::Calendar)
            .icon(icon::from_name("x-office-calendar-symbolic"));

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
        let (cal_year, cal_month) = parse_ym(&data.selected_date)
            .or_else(|| parse_ym(&today))
            .unwrap_or_else(|| {
                let now = chrono::Local::now();
                (now.year(), now.month())
            });

        let scratchpad_content = text_editor::Content::with_text(&data.scratchpad);
        let day_note_text = data
            .day_notes
            .get(&data.selected_date)
            .map(String::as_str)
            .unwrap_or("");
        let day_note_content = text_editor::Content::with_text(day_note_text);
        let today_note_text = data
            .day_notes
            .get(&today)
            .map(String::as_str)
            .unwrap_or("");
        let today_note_content = text_editor::Content::with_text(today_note_text);

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
            dirty: false,
            save_countdown: 0,
            scratchpad_content,
            day_note_content,
            today_note_content,
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
            Some(Page::Dashboard) => ui::dashboard_page::view(
                &self.data,
                self.cal_year,
                self.cal_month,
                &self.today_note_content,
            ),
            Some(Page::Calendar) => ui::calendar_page::view(
                &self.data,
                self.cal_year,
                self.cal_month,
                &self.day_note_content,
            ),
            Some(Page::Scratchpad) => {
                ui::scratchpad_page::view(&self.scratchpad_content, self.dirty)
            }
            Some(Page::Clipboard) => ui::clipboard_page::view(&self.data),
            None => widget::text("No page selected").into(),
        };

        widget::container(content)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let config_watch = self
            .core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config));

        // Clipboard polling: every 2 seconds.
        let clipboard_poll = Subscription::run(|| {
            iced_futures::stream::channel(
                1,
                |mut tx: iced_futures::futures::channel::mpsc::Sender<Message>| async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(2));
                    loop {
                        interval.tick().await;
                        let _ = tx.send(Message::ClipboardTick).await;
                    }
                },
            )
        });

        // Save tick: fires every 200 ms to drive debounced autosave.
        let save_tick = Subscription::run(|| {
            iced_futures::stream::channel(
                1,
                |mut tx: iced_futures::futures::channel::mpsc::Sender<Message>| async move {
                    let mut interval = tokio::time::interval(Duration::from_millis(200));
                    loop {
                        interval.tick().await;
                        let _ = tx.send(Message::SaveTick).await;
                    }
                },
            )
        });

        Subscription::batch(vec![config_watch, clipboard_poll, save_tick])
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

            Message::GoToToday => {
                let today = calendar::today_string();
                if let Some((y, m)) = parse_ym(&today) {
                    self.cal_year = y;
                    self.cal_month = m;
                }
                self.data.selected_date = today.clone();
                // Re-initialize day note editor for the newly selected date.
                let text = self
                    .data
                    .day_notes
                    .get(&today)
                    .map(String::as_str)
                    .unwrap_or("");
                self.day_note_content = text_editor::Content::with_text(text);
                self.mark_dirty();
            }

            Message::SelectDate(date) => {
                if let Some((y, m)) = parse_ym(&date) {
                    self.cal_year = y;
                    self.cal_month = m;
                }
                // Re-initialize day note editor for the newly selected date.
                let text = self
                    .data
                    .day_notes
                    .get(&date)
                    .map(String::as_str)
                    .unwrap_or("");
                self.day_note_content = text_editor::Content::with_text(text);
                self.data.selected_date = date;
                self.mark_dirty();
            }

            Message::ScratchpadAction(action) => {
                self.scratchpad_content.perform(action);
                self.data.scratchpad = self.scratchpad_content.text();
                self.mark_dirty();
            }

            Message::DayNoteAction(action) => {
                self.day_note_content.perform(action);
                let text = self.day_note_content.text();
                self.data
                    .set_day_note(self.data.selected_date.clone(), text);
                self.mark_dirty();
            }

            Message::TodayNoteAction(action) => {
                self.today_note_content.perform(action);
                let text = self.today_note_content.text();
                let today = calendar::today_string();
                self.data.set_day_note(today, text);
                self.mark_dirty();
            }

            Message::ClipboardTick => {
                if let Some(text) = cb::get_text() {
                    if self.last_clipboard.as_deref() != Some(text.as_str()) {
                        self.last_clipboard = Some(text.clone());
                        if self.data.push_clipboard(text) {
                            self.mark_dirty();
                        }
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
                self.mark_dirty();
            }

            Message::UnpinClipboard(text) => {
                self.data.unpin_clipboard(&text);
                self.mark_dirty();
            }

            Message::ClearClipboardHistory => {
                self.data.clear_clipboard_history();
                self.last_clipboard = None;
                self.mark_dirty();
            }

            Message::SaveTick => {
                if self.dirty && self.save_countdown > 0 {
                    self.save_countdown -= 1;
                    if self.save_countdown == 0 {
                        self.data.save();
                        self.dirty = false;
                    }
                }
            }
        }
        Task::none()
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Flush unsaved changes immediately when switching pages.
        if self.dirty {
            self.data.save();
            self.dirty = false;
            self.save_countdown = 0;
        }
        self.nav.activate(id);
        self.update_title()
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

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

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.save_countdown = SAVE_IDLE_TICKS;
    }
}

// ── Page / context enums ──────────────────────────────────────────────────────

pub enum Page {
    Dashboard,
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

// ── Utilities ─────────────────────────────────────────────────────────────────

fn parse_ym(date: &str) -> Option<(i32, u32)> {
    let mut parts = date.splitn(3, '-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    if (1..=12).contains(&month) {
        Some((year, month))
    } else {
        None
    }
}
