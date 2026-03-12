use super::super::constants::UNFOCUS_GUARD_MS;
use super::super::surface::launcher_visible_surface_settings;
use super::{Launcher, Message};
use iced::{Task, window};
use log::error;
use std::env;
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
                error!("failed to launch {}: {error}", app.name);
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

        if let Some(id) = self.launcher_window_id.take() {
            Task::done(Message::RemoveWindow(id))
        } else {
            Task::none()
        }
    }

    pub(super) fn recreate_launcher_surface(&mut self) -> Task<Message> {
        let previous_window_id = self.launcher_window_id;
        let new_window_id = window::Id::unique();
        self.launcher_window_id = Some(new_window_id);

        let mut tasks = Vec::new();
        if let Some(id) = previous_window_id {
            tasks.push(Task::done(Message::RemoveWindow(id)));
        }
        tasks.push(Task::done(Message::NewLayerShell {
            settings: if self.is_visible {
                launcher_visible_surface_settings(
                    &self.layout,
                    self.results_progress > 0.0 || self.results_target > 0.0,
                    self.target_output_name.as_deref(),
                )
            } else {
                launcher_visible_surface_settings(&self.layout, false, None)
            },
            id: new_window_id,
        }));

        Task::batch(tasks)
    }

    pub(super) fn open_preferences_window(&mut self) -> Task<Message> {
        let current_exe = match env::current_exe() {
            Ok(path) => path,
            Err(error) => {
                error!("failed to locate current executable for preferences window: {error}");
                return Task::none();
            }
        };

        if let Err(error) = Command::new(current_exe)
            .arg("--preferences")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .spawn()
        {
            error!("failed to open preferences window: {error}");
        }

        Task::none()
    }

    pub(super) fn should_ignore_unfocus(&self) -> bool {
        self.ignore_unfocus_until
            .is_some_and(|deadline| Instant::now() < deadline)
    }
}
