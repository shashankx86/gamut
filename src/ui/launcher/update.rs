use super::super::constants::UNFOCUS_GUARD_MS;
use super::{Launcher, Message};
use crate::core::ipc::IpcCommand;
use iced::keyboard::{self, Key, key::Named};
use iced::widget;
use iced::{Task, window};
use std::time::{Duration, Instant};

impl Launcher {
    pub(in crate::ui) fn update(&mut self, message: Message) -> Task<Message> {
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
            Message::WindowClosed(id) => self.on_window_closed(id),
            Message::KeyboardEvent(id, key_event) => self.handle_key_event(id, key_event),
            Message::WindowEvent(id, window_event) => self.handle_window_event(id, window_event),
            _ => Task::none(),
        }
    }

    fn handle_ipc(&mut self) -> Task<Message> {
        let mut latest_command = None;

        while let Ok(command) = self.ipc_receiver.try_recv() {
            latest_command = Some(command);
        }

        match latest_command {
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

    fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        if self.window_id == Some(id) {
            self.window_id = None;
            self.clear_window_state();
        }

        Task::none()
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
}
