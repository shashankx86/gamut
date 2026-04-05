use super::super::{Launcher, Message};
use crate::ui::constants::UNFOCUS_GUARD_MS;
use crate::ui::surface::launcher_visible_surface_settings;
use iced::{window, Task};
use log::error;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

impl Launcher {
    pub(in crate::ui::launcher) fn expand_results(&mut self) -> Task<Message> {
        if self.normalized_query.is_empty() && self.results_target == 0.0 {
            self.results_target = 1.0;
            self.manually_expanded = true;
        }

        Task::none()
    }

    pub(in crate::ui::launcher) fn launch(&mut self, index: usize) -> Task<Message> {
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

    pub(in crate::ui::launcher) fn open_location(&mut self, index: usize) -> Task<Message> {
        let Some(app) = self.apps.get(index) else {
            return Task::none();
        };

        let path = Path::new(&app.command);
        let Some(parent) = path.parent() else {
            error!("failed to determine parent directory for {}", app.command);
            return Task::none();
        };

        match Command::new("xdg-open").arg(parent).spawn() {
            Ok(_) => self.hide_launcher(),
            Err(open_error) => {
                let fallback = Command::new("gio")
                    .arg("open")
                    .arg(parent)
                    .spawn()
                    .map_err(|gio_error| format!("xdg-open: {open_error}; gio open: {gio_error}"));

                match fallback {
                    Ok(_) => self.hide_launcher(),
                    Err(error) => {
                        error!(
                            "failed to open application location for {}: {error}",
                            app.name
                        );
                        Task::none()
                    }
                }
            }
        }
    }

    pub(in crate::ui::launcher) fn handle_action_button_pressed(&mut self) -> Task<Message> {
        if !self.is_expanded() || self.selected_application_index().is_none() {
            self.action_overlay_pinned = false;
            return Task::none();
        }

        self.action_overlay_pinned = !self.action_overlay_pinned;
        Task::none()
    }

    pub(in crate::ui::launcher) fn show_launcher(&mut self) -> Task<Message> {
        if self.is_visible {
            return Task::none();
        }

        self.clear_window_state();
        self.is_visible = true;
        self.arm_unfocus_guard();

        Task::batch(vec![
            self.request_app_refresh(true),
            self.recreate_launcher_surface(),
        ])
    }

    pub(in crate::ui::launcher) fn hide_launcher(&mut self) -> Task<Message> {
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

    pub(in crate::ui::launcher) fn recreate_launcher_surface(&mut self) -> Task<Message> {
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
                    self.target_output.as_deref(),
                )
            } else {
                launcher_visible_surface_settings(&self.layout, false, None)
            },
            id: new_window_id,
        }));

        Task::batch(tasks)
    }

    pub(in crate::ui::launcher) fn should_ignore_unfocus(&self) -> bool {
        self.ignore_unfocus_until
            .is_some_and(|deadline| Instant::now() < deadline)
    }

    pub(in crate::ui::launcher) fn arm_unfocus_guard(&mut self) {
        self.ignore_unfocus_until = Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));
    }
}
