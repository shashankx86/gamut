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
        if self.is_visible {
            return Task::none();
        }

        self.clear_window_state();
        self.is_visible = true;
        self.ignore_unfocus_until = Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));

        self.recreate_launcher_surface()
    }

    pub(super) fn hide_launcher(&mut self) -> Task<Message> {
        if !self.is_visible {
            return Task::none();
        }

        self.is_visible = false;
        self.clear_window_state();

        if let Some(id) = self.window_id.take() {
            Task::done(Message::RemoveWindow(id))
        } else {
            Task::none()
        }
    }

    pub(super) fn recreate_launcher_surface(&mut self) -> Task<Message> {
        let previous_window_id = self.window_id;
        let new_window_id = window::Id::unique();
        self.window_id = Some(new_window_id);

        let mut tasks = Vec::new();
        if let Some(id) = previous_window_id {
            tasks.push(Task::done(Message::RemoveWindow(id)));
        }
        tasks.push(Task::done(Message::NewLayerShell {
            settings: launcher_surface_settings(),
            id: new_window_id,
        }));

        Task::batch(tasks)
    }

    pub(super) fn should_ignore_unfocus(&self) -> bool {
        self.ignore_unfocus_until
            .is_some_and(|deadline| Instant::now() < deadline)
    }
}
