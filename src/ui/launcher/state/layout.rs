use super::super::Message;
use super::Launcher;
use crate::core::preferences::load_preferences;
use crate::ui::layout::LauncherLayout;
use iced::{Size, Task, window};

impl Launcher {
    pub(in crate::ui::launcher) fn update_layout(
        &mut self,
        monitor_size: Option<Size>,
    ) -> Task<Message> {
        self.monitor_size = monitor_size;
        let previous_layout = self.layout.clone();
        self.layout = LauncherLayout::from_monitor_size(
            self.monitor_size,
            &self.layout_preferences,
            &self.app_preferences,
        );

        if !self.is_visible {
            return Task::none();
        }

        if self.layout.should_recreate_surface(&previous_layout) {
            return self.recreate_launcher_surface();
        }

        let Some(id) = self.launcher_window_id else {
            return Task::none();
        };

        let size = if self.results_progress > 0.0 || self.results_target > 0.0 {
            self.layout.expanded_surface_size()
        } else {
            self.layout.collapsed_surface_size()
        };

        if size
            != if self.results_progress > 0.0 || self.results_target > 0.0 {
                previous_layout.expanded_surface_size()
            } else {
                previous_layout.collapsed_surface_size()
            }
        {
            self.request_surface_resize(id, size)
        } else {
            Task::none()
        }
    }

    pub(in crate::ui::launcher) fn sync_highlighted_rank(&mut self, revision: u64, rank: usize) {
        if revision != self.selection_revision {
            return;
        }

        self.highlighted_rank = if self.filtered_indices.is_empty() {
            0
        } else {
            rank.min(self.filtered_indices.len().saturating_sub(1))
        };
    }

    pub(in crate::ui::launcher) fn reload_preferences_from_disk(&mut self) -> Task<Message> {
        self.app_preferences = load_preferences();
        self.refresh_visual_cache();
        self.tray_controller
            .update_preferences(self.app_preferences.clone());
        self.update_layout(self.monitor_size)
    }

    pub(in crate::ui) fn window_theme_for(&self, id: window::Id) -> iced::Theme {
        let _ = id;
        self.window_theme()
    }

    pub(in crate::ui) fn window_title(&self, id: window::Id) -> Option<String> {
        let _ = id;
        Some("Gamut".to_string())
    }

    pub(in crate::ui::launcher) fn is_launcher_window(&self, id: window::Id) -> bool {
        self.launcher_window_id == Some(id)
    }
}
