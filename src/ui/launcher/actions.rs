use super::super::constants::UNFOCUS_GUARD_MS;
use super::super::launcher_surface_settings;
use super::{Launcher, Message};
use iced::{Task, window};
use std::process::Command;
use std::time::{Duration, Instant};

impl Launcher {
    pub(super) fn launch(&mut self, index: usize) -> Task<Message> {
        let Some(app) = self.apps.get(index) else {
            return Task::none();
        };

        match Command::new(&app.command).args(&app.args).spawn() {
            Ok(_) => self.hide_launcher(),
            Err(error) => {
                eprintln!("Failed to launch {}: {error}", app.name);
                Task::none()
            }
        }
    }

    pub(super) fn show_launcher(&mut self) -> Task<Message> {
        if self.window_id.is_some() {
            return Task::none();
        }

        let id = window::Id::unique();

        self.clear_window_state();
        self.window_id = Some(id);
        self.ignore_unfocus_until = Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));

        Task::done(Message::NewLayerShell {
            settings: launcher_surface_settings(),
            id,
        })
    }

    pub(super) fn hide_launcher(&mut self) -> Task<Message> {
        let window_id = self.window_id.take();
        self.clear_window_state();

        if let Some(id) = window_id {
            Task::done(Message::RemoveWindow(id))
        } else {
            Task::none()
        }
    }

    pub(super) fn should_ignore_unfocus(&self) -> bool {
        self.ignore_unfocus_until
            .is_some_and(|deadline| Instant::now() < deadline)
    }
}
