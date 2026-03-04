use crate::desktop::{DesktopApp, load_apps, trim_label};
use crate::ipc::{IpcCommand, start_listener};
use iced::keyboard::{self, Key, key::Named};
use iced::widget::{self, button, column, container, row, scrollable, text, text_input};
use iced::{Element, Event, Length, Subscription, Task, Theme, event, time, window};
use iced_layershell::daemon;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use std::error::Error;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const SHOWN_HEIGHT: u32 = 760;
const SURFACE_TOP_MARGIN: i32 = 120;
const UNFOCUS_GUARD_MS: u64 = 250;
const MAX_RESULTS: usize = 200;

type DynError = Box<dyn Error>;

pub fn run_daemon() -> Result<(), DynError> {
    daemon(Launcher::new, namespace, Launcher::update, Launcher::view)
        .subscription(Launcher::subscription)
        .theme(launcher_theme)
        .settings(Settings {
            layer_settings: daemon_layer_settings(),
            ..Settings::default()
        })
        .run()
        .map_err(|error| Box::new(error) as DynError)
}

fn launcher_theme(_state: &Launcher, _id: window::Id) -> Theme {
    Theme::Dark
}

struct Launcher {
    apps: Vec<DesktopApp>,
    query: String,
    input_id: widget::Id,
    status: Option<String>,
    window_id: Option<window::Id>,
    had_focus: bool,
    ignore_unfocus_until: Option<Instant>,
    ipc_receiver: Receiver<IpcCommand>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Message {
    Tick,
    AppsLoaded(Vec<DesktopApp>),
    QueryChanged(String),
    LaunchFirstMatch,
    LaunchIndex(usize),
    KeyboardEvent(window::Id, keyboard::Event),
    WindowEvent(window::Id, window::Event),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl Launcher {
    fn new() -> (Self, Task<Message>) {
        let input_id = widget::Id::unique();

        let (ipc_receiver, status) = match start_listener() {
            Ok(receiver) => (receiver, Some("Ready".to_string())),
            Err(error) => {
                let (_tx, receiver) = mpsc::channel();
                (
                    receiver,
                    Some(format!("IPC listener unavailable: {error}. daemon mode unavailable.")),
                )
            }
        };

        (
            Self {
                apps: Vec::new(),
                query: String::new(),
                input_id,
                status,
                window_id: None,
                had_focus: false,
                ignore_unfocus_until: None,
                ipc_receiver,
            },
            Task::perform(async { load_apps() }, Message::AppsLoaded),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            event::listen_with(|event, _status, id| match event {
                Event::Keyboard(key_event) => Some(Message::KeyboardEvent(id, key_event)),
                _ => None,
            }),
            event::listen_with(|event, _status, id| match event {
                Event::Window(window_event) => Some(Message::WindowEvent(id, window_event)),
                _ => None,
            }),
            window::open_events().map(Message::WindowOpened),
            window::close_events().map(Message::WindowClosed),
            time::every(Duration::from_millis(25)).map(|_| Message::Tick),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.handle_ipc(),
            Message::AppsLoaded(apps) => {
                self.apps = apps;
                self.status = None;
                Task::none()
            }
            Message::QueryChanged(query) => {
                self.query = query;
                Task::none()
            }
            Message::LaunchFirstMatch => {
                if let Some(index) = self.filtered_indices().first().copied() {
                    self.launch(index)
                } else {
                    Task::none()
                }
            }
            Message::LaunchIndex(index) => self.launch(index),
            Message::WindowOpened(id) => self.on_window_opened(id),
            Message::WindowClosed(id) => {
                if self.window_id == Some(id) {
                    self.window_id = None;
                    self.query.clear();
                    self.had_focus = false;
                    self.ignore_unfocus_until = None;
                }
                Task::none()
            }
            Message::KeyboardEvent(id, key_event) => self.handle_key_event(id, key_event),
            Message::WindowEvent(id, window_event) => self.handle_window_event(id, window_event),
            _ => Task::none(),
        }
    }

    fn view(&self, _window: window::Id) -> Element<'_, Message> {
        let query_input = text_input("Search applications", &self.query)
            .id(self.input_id.clone())
            .on_input(Message::QueryChanged)
            .on_submit(Message::LaunchFirstMatch)
            .padding([10, 12])
            .size(18);

        let mut results = column![].spacing(6);
        let filtered = self.filtered_indices();

        if filtered.is_empty() {
            results = results.push(text("No applications found"));
        } else {
            for index in filtered.into_iter().take(MAX_RESULTS) {
                results = results.push(self.view_result(index));
            }
        }

        let footer = text(self.status.as_deref().unwrap_or("Open application"));

        let panel = column![
            query_input,
            scrollable(results).height(Length::Fill),
            footer,
        ]
        .spacing(12)
        .width(Length::Fill);

        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .into()
    }

    fn view_result(&self, index: usize) -> Element<'_, Message> {
        let app = &self.apps[index];
        let label = row![
            text(&app.name),
            text(" "),
            text(trim_label(&app.exec_line, 64)).size(13),
        ]
        .spacing(8)
        .width(Length::Fill);

        button(label)
            .width(Length::Fill)
            .on_press(Message::LaunchIndex(index))
            .into()
    }

    fn filtered_indices(&self) -> Vec<usize> {
        self.apps
            .iter()
            .enumerate()
            .filter_map(|(index, app)| app.matches_query(&self.query).then_some(index))
            .collect()
    }

    fn launch(&mut self, index: usize) -> Task<Message> {
        let Some(app) = self.apps.get(index) else {
            return Task::none();
        };

        match Command::new(&app.command).args(&app.args).spawn() {
            Ok(_) => self.hide_launcher(),
            Err(error) => {
                self.status = Some(format!("Failed to launch {}: {error}", app.name));
                Task::none()
            }
        }
    }

    fn handle_ipc(&mut self) -> Task<Message> {
        let mut latest = None;
        while let Ok(command) = self.ipc_receiver.try_recv() {
            latest = Some(command);
        }

        match latest {
            Some(IpcCommand::Toggle) => {
                if self.window_id.is_some() {
                    self.hide_launcher()
                } else {
                    self.show_launcher()
                }
            }
            Some(IpcCommand::Quit) => iced::exit(),
            Some(IpcCommand::Ping) | None => Task::none(),
        }
    }

    fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        if self.window_id != Some(id) {
            return Task::none();
        }

        self.ignore_unfocus_until = Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));
        self.had_focus = false;

        Task::batch(vec![
            widget::operation::focus(self.input_id.clone()),
            widget::operation::move_cursor_to_end(self.input_id.clone()),
        ])
    }

    fn handle_key_event(&mut self, id: window::Id, event: keyboard::Event) -> Task<Message> {
        if self.window_id != Some(id) {
            return Task::none();
        }

        match event {
            keyboard::Event::KeyPressed { key, .. }
                if matches!(key.as_ref(), Key::Named(Named::Escape)) =>
            {
                self.hide_launcher()
            }
            _ => Task::none(),
        }
    }

    fn handle_window_event(&mut self, id: window::Id, event: window::Event) -> Task<Message> {
        if self.window_id != Some(id) {
            return Task::none();
        }

        match event {
            window::Event::Focused => {
                self.had_focus = true;
                self.ignore_unfocus_until = None;
                Task::none()
            }
            window::Event::Unfocused if self.had_focus && !self.should_ignore_unfocus() => {
                self.hide_launcher()
            }
            window::Event::CloseRequested => self.hide_launcher(),
            _ => Task::none(),
        }
    }

    fn should_ignore_unfocus(&self) -> bool {
        match self.ignore_unfocus_until {
            Some(deadline) => Instant::now() < deadline,
            None => false,
        }
    }

    fn show_launcher(&mut self) -> Task<Message> {
        if self.window_id.is_some() {
            return Task::none();
        }

        let id = window::Id::unique();

        self.query.clear();
        self.window_id = Some(id);
        self.had_focus = false;
        self.ignore_unfocus_until = Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));

        Task::done(Message::NewLayerShell {
            settings: launcher_surface_settings(),
            id,
        })
    }

    fn hide_launcher(&mut self) -> Task<Message> {
        self.query.clear();
        self.had_focus = false;
        self.ignore_unfocus_until = None;

        if let Some(id) = self.window_id.take() {
            Task::done(Message::RemoveWindow(id))
        } else {
            Task::none()
        }
    }
}

fn namespace() -> String {
    "gamut-launcher".to_string()
}

fn daemon_layer_settings() -> LayerShellSettings {
    LayerShellSettings {
        start_mode: StartMode::Background,
        ..LayerShellSettings::default()
    }
}

fn launcher_surface_settings() -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some((0, SHOWN_HEIGHT)),
        layer: Layer::Overlay,
        anchor: Anchor::Top | Anchor::Left | Anchor::Right,
        exclusive_zone: None,
        margin: Some((SURFACE_TOP_MARGIN, 0, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        events_transparent: false,
        namespace: Some(namespace()),
        ..NewLayerShellSettings::default()
    }
}
