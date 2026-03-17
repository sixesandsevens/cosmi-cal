// SPDX-License-Identifier: MPL-2.0

use crate::calendar;
use crate::commands::AppCommand;
use crate::config::Config;
use crate::fl;
use crate::focus::FocusTarget;
use crate::ipc;
use crate::message::Message;
use crate::model::AppData;
use crate::ui;
use chrono::Datelike;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Subscription, clipboard as iced_clipboard};
use cosmic::widget::segmented_button;
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
    shell_mode: ShellMode,
    page_ids: PageIds,
    pub data: AppData,
    pub cal_year: i32,
    pub cal_month: u32,
    /// Last clipboard text seen, used to debounce polling.
    last_clipboard: Option<String>,
    /// True when data has changed since the last save.
    dirty: bool,
    /// Counts down from SAVE_IDLE_TICKS after a change; saves when it hits 0.
    save_countdown: u8,
    /// Set when the last save attempt failed; cleared on next successful save.
    save_error: bool,
    /// The calendar date last seen on the rollover tick, for midnight detection.
    last_seen_date: String,
    /// Current window width in logical pixels, updated via on_window_resize.
    pub window_width: f32,
    /// Multiline editor state for the scratchpad.
    pub scratchpad_content: text_editor::Content,
    /// Multiline editor state for the calendar selected-day note.
    pub day_note_content: text_editor::Content,
    pub day_note_editor_id: widget::Id,
    pub scratchpad_editor_id: widget::Id,
    pending_focus: FocusTarget,
}

#[derive(Clone, Default)]
pub struct LaunchFlags {
    startup_commands: Vec<AppCommand>,
}

impl LaunchFlags {
    pub fn new(startup_commands: Vec<AppCommand>) -> Self {
        Self { startup_commands }
    }
}

#[derive(Clone, Copy)]
struct PageIds {
    dashboard: segmented_button::Entity,
    scratchpad: segmented_button::Entity,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = LaunchFlags;
    type Message = Message;

    const APP_ID: &'static str = "io.github.sixesandsevens.cosmical";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        mut core: cosmic::Core,
        flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let shell_mode = ShellMode::from_env();

        if shell_mode.is_dashboard() {
            core.window.show_headerbar = false;
        }
        if shell_mode == ShellMode::Surface {
            core.window.show_close = false;
            core.window.show_maximize = false;
            core.window.show_minimize = false;
            core.window.show_window_menu = false;
        }

        let mut nav = nav_bar::Model::default();

        let dashboard = nav
            .insert()
            .text(fl!("nav-dashboard"))
            .data::<Page>(Page::Dashboard)
            .icon(icon::from_name("view-grid-symbolic"))
            .activate()
            .id();

        nav.insert()
            .text(fl!("nav-calendar"))
            .data::<Page>(Page::Calendar)
            .icon(icon::from_name("x-office-calendar-symbolic"));

        let scratchpad = nav
            .insert()
            .text(fl!("nav-scratchpad"))
            .data::<Page>(Page::Scratchpad)
            .icon(icon::from_name("document-edit-symbolic"))
            .id();

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
            shell_mode,
            page_ids: PageIds {
                dashboard,
                scratchpad,
            },
            data,
            cal_year,
            cal_month,
            last_clipboard: None,
            dirty: false,
            save_countdown: 0,
            save_error: false,
            last_seen_date: today.clone(),
            window_width: 800.0,
            scratchpad_content,
            day_note_content,
            day_note_editor_id: widget::Id::unique(),
            scratchpad_editor_id: widget::Id::unique(),
            pending_focus: FocusTarget::None,
        };

        let mut commands = vec![app.update_title()];
        for command in flags.startup_commands {
            commands.push(app.handle_app_command(command));
        }
        let command = Task::batch(commands);
        (app, command)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        if self.shell_mode.is_dashboard() {
            return vec![];
        }
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
        match self.shell_mode {
            ShellMode::Full => Some(&self.nav),
            ShellMode::DashboardOnly | ShellMode::Surface => None,
        }
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
        let content: Element<_> = if self.shell_mode.is_dashboard() {
            ui::dashboard_page::view(
                &self.data,
                self.cal_year,
                self.cal_month,
                &self.day_note_content,
                self.window_width,
                self.day_note_editor_id.clone(),
            )
        } else {
            match self.nav.active_data::<Page>() {
                Some(Page::Dashboard) => ui::dashboard_page::view(
                    &self.data,
                    self.cal_year,
                    self.cal_month,
                    &self.day_note_content,
                    self.window_width,
                    self.day_note_editor_id.clone(),
                ),
                Some(Page::Calendar) => ui::calendar_page::view(
                    &self.data,
                    self.cal_year,
                    self.cal_month,
                    &self.day_note_content,
                    self.day_note_editor_id.clone(),
                ),
                Some(Page::Scratchpad) => ui::scratchpad_page::view(
                    &self.scratchpad_content,
                    self.scratchpad_editor_id.clone(),
                    self.dirty,
                    self.save_error,
                ),
                Some(Page::Clipboard) => ui::clipboard_page::view(&self.data),
                None => widget::text("No page selected").into(),
            }
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

        // Date rollover: check every minute so we catch midnight.
        let rollover_tick = Subscription::run(|| {
            iced_futures::stream::channel(
                1,
                |mut tx: iced_futures::futures::channel::mpsc::Sender<Message>| async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(60));
                    loop {
                        interval.tick().await;
                        let _ = tx.send(Message::DateRolloverTick).await;
                    }
                },
            )
        });

        let ipc_commands = ipc::subscription().map(Message::AppCommand);

        Subscription::batch(vec![
            config_watch,
            clipboard_poll,
            save_tick,
            rollover_tick,
            ipc_commands,
        ])
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
                self.select_date_internal(calendar::today_string(), true);
            }

            Message::SelectDate(date) => {
                self.select_date_internal(date, true);
            }

            Message::ScratchpadAction(action) => {
                self.scratchpad_content.perform(action);
                self.data.scratchpad = trim_editor_text(self.scratchpad_content.text());
                self.mark_dirty();
            }

            Message::DayNoteAction(action) => {
                self.day_note_content.perform(action);
                let text = trim_editor_text(self.day_note_content.text());
                self.data
                    .set_day_note(self.data.selected_date.clone(), text);
                self.mark_dirty();
            }

            Message::ClipboardTick => {
                return iced_clipboard::read()
                    .map(|s| cosmic::Action::App(Message::ClipboardRead(s)));
            }

            Message::ClipboardRead(maybe_text) => {
                if let Some(text) = maybe_text {
                    if self.last_clipboard.as_deref() != Some(text.as_str()) {
                        self.last_clipboard = Some(text.clone());
                        if self.data.push_clipboard(text) {
                            self.mark_dirty();
                        }
                    }
                }
            }

            Message::RestoreClipboard(text) => {
                return iced_clipboard::write(text)
                    .map(|()| cosmic::Action::App(Message::SaveTick));
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

            Message::AppCommand(command) => {
                return self.handle_app_command(command);
            }

            Message::SaveTick => {
                if self.dirty && self.save_countdown > 0 {
                    self.save_countdown -= 1;
                    if self.save_countdown == 0 {
                        if self.data.save() {
                            self.dirty = false;
                            self.save_error = false;
                        } else {
                            self.save_error = true;
                            // Retry next tick cycle.
                            self.save_countdown = SAVE_IDLE_TICKS;
                        }
                    }
                }
            }

            Message::DateRolloverTick => {
                let today = calendar::today_string();
                if today != self.last_seen_date {
                    self.last_seen_date = today.clone();
                    // Snap the selected date to the new day and refresh the editor.
                    self.select_date_internal(today, false);
                }
            }
        }
        Task::none()
    }

    fn on_window_resize(&mut self, _id: cosmic::iced::window::Id, width: f32, _height: f32) {
        self.window_width = width;
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Flush unsaved changes immediately when switching pages.
        if self.dirty {
            if self.data.save() {
                self.dirty = false;
                self.save_error = false;
            } else {
                self.save_error = true;
            }
            self.save_countdown = 0;
        }
        self.nav.activate(id);
        self.update_title()
    }

    fn on_app_exit(&mut self) -> Option<Self::Message> {
        // Flush any pending changes synchronously before the process exits.
        if self.dirty {
            self.data.save();
        }
        None
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

    fn handle_app_command(&mut self, command: AppCommand) -> Task<cosmic::Action<Message>> {
        match command {
            AppCommand::ShowSurface => {
                if !self.shell_mode.is_dashboard() {
                    self.nav.activate(self.page_ids.dashboard);
                }

                Task::batch(vec![self.focus_main_window(), self.update_title()])
            }
            AppCommand::FocusTodayNote => {
                if !self.shell_mode.is_dashboard() {
                    self.nav.activate(self.page_ids.dashboard);
                }

                self.select_date_internal(calendar::today_string(), true);
                self.pending_focus = FocusTarget::TodayNote;

                Task::batch(vec![
                    self.focus_main_window(),
                    self.focus_pending_target(),
                    self.update_title(),
                ])
            }
            AppCommand::FocusScratchpad => {
                if !self.shell_mode.is_dashboard() {
                    self.nav.activate(self.page_ids.scratchpad);
                }

                self.pending_focus = FocusTarget::Scratchpad;

                Task::batch(vec![
                    self.focus_main_window(),
                    self.focus_pending_target(),
                    self.update_title(),
                ])
            }
        }
    }

    fn focus_main_window(&self) -> Task<cosmic::Action<Message>> {
        let Some(id) = self.core.main_window_id() else {
            return Task::none();
        };

        cosmic::iced_runtime::window::minimize(id, false)
            .chain(cosmic::iced_runtime::window::gain_focus(id))
    }

    fn focus_pending_target(&mut self) -> Task<cosmic::Action<Message>> {
        let target = self.pending_focus;
        self.pending_focus = FocusTarget::None;

        match target {
            FocusTarget::TodayNote => {
                cosmic::iced_runtime::widget::operation::focus(self.day_note_editor_id.clone())
            }
            FocusTarget::Scratchpad => {
                cosmic::iced_runtime::widget::operation::focus(self.scratchpad_editor_id.clone())
            }
            FocusTarget::None => Task::none(),
        }
    }

    fn select_date_internal(&mut self, date: String, persist_selection: bool) {
        if let Some((y, m)) = parse_ym(&date) {
            self.cal_year = y;
            self.cal_month = m;
        }

        let text = self
            .data
            .day_notes
            .get(&date)
            .map(String::as_str)
            .unwrap_or("");
        self.day_note_content = text_editor::Content::with_text(text);
        self.data.selected_date = date;

        if persist_selection {
            self.mark_dirty();
        }
    }
}

// ── Shell mode ────────────────────────────────────────────────────────────────

/// Controls how much chrome the app shows.
///
/// Set `COSMICAL_MODE` in the environment before launching:
///   (unset)   — surface mode, optimized for everyday always-open use
///   full      — normal full-shell app with nav bar and titlebar
///   dashboard — nav bar hidden, no menu bar, no titlebar; dashboard always shown
///   surface   — everything above, plus all window controls stripped; intended
///               for desktop-embedding experiments
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShellMode {
    Full,
    /// Dashboard always visible, nav/menu bar hidden.
    DashboardOnly,
    /// Like DashboardOnly but with the strongest possible chrome removal —
    /// all window controls off, sharp corners, no window menu.
    /// Use this as the base for desktop-embedding experiments.
    Surface,
}

impl Default for ShellMode {
    fn default() -> Self {
        Self::Surface
    }
}

impl ShellMode {
    fn from_env() -> Self {
        match std::env::var("COSMICAL_MODE").as_deref() {
            Ok("full") => Self::Full,
            Ok("dashboard") => Self::DashboardOnly,
            Ok("surface") => Self::Surface,
            _ => Self::Surface,
        }
    }

    fn is_dashboard(&self) -> bool {
        matches!(self, Self::DashboardOnly | Self::Surface)
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

/// `text_editor::Content::text()` always appends a trailing newline.
/// Strip it before storing so saved content round-trips cleanly.
fn trim_editor_text(mut s: String) -> String {
    if s.ends_with('\n') {
        s.pop();
    }
    s
}
