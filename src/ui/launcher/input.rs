use super::{Launcher, Message};
use iced::keyboard::{self, key::Named, Key, Modifiers};
use iced::widget::{operation, scrollable};
use iced::{window, Task};

impl Launcher {
    pub(super) fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        if self.is_launcher_window(id) {
            let monitor_size = window::monitor_size(id).map(Message::MonitorSizeLoaded);

            if !self.is_visible {
                return monitor_size;
            }

            self.arm_unfocus_guard();

            return Task::batch(vec![
                monitor_size,
                operation::focus(self.input_id.clone()),
                operation::move_cursor_to_end(self.input_id.clone()),
                self.request_icon_resolution_for_visible(),
            ]);
        }

        Task::none()
    }

    pub(super) fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        if self.is_launcher_window(id) {
            self.launcher_window_id = None;
            self.is_visible = false;
            self.clear_window_state();
        }

        Task::none()
    }

    pub(super) fn handle_key_event(
        &mut self,
        id: window::Id,
        event: keyboard::Event,
    ) -> Task<Message> {
        match event {
            keyboard::Event::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            keyboard::Event::KeyPressed {
                key,
                modifiers,
                physical_key,
                ..
            } => {
                self.modifiers = modifiers;

                if self.is_launcher_window(id) && self.is_visible {
                    return self.handle_launcher_key_press(key, modifiers, physical_key);
                }

                Task::none()
            }
            keyboard::Event::KeyReleased { modifiers, .. } => {
                self.modifiers = modifiers;
                Task::none()
            }
        }
    }

    pub(super) fn handle_window_event(
        &mut self,
        id: window::Id,
        event: window::Event,
    ) -> Task<Message> {
        if self.is_launcher_window(id) && self.is_visible {
            return match event {
                window::Event::Focused => {
                    self.ignore_unfocus_until = None;
                    Task::none()
                }
                window::Event::Unfocused if !self.should_ignore_unfocus() => self.hide_launcher(),
                window::Event::CloseRequested => self.hide_launcher(),
                _ => Task::none(),
            };
        }

        Task::none()
    }

    pub(super) fn handle_launcher_key_press(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        physical_key: keyboard::key::Physical,
    ) -> Task<Message> {
        if self.matches_shortcut(
            &self.app_preferences.shortcuts.close_launcher,
            &key,
            modifiers,
            physical_key,
        ) {
            return self.hide_launcher();
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.expand_or_move_down,
            &key,
            modifiers,
            physical_key,
        ) {
            if self.normalized_query.is_empty() && self.results_target == 0.0 {
                self.results_target = 1.0;
                self.manually_expanded = true;
                return Task::none();
            }

            let previous_rank = self.selected_rank;
            self.selection_revision = self.selection_revision.wrapping_add(1);
            self.move_selection(1);
            return self.scroll_to_selected(previous_rank, false);
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.move_up,
            &key,
            modifiers,
            physical_key,
        ) {
            let previous_rank = self.selected_rank;
            self.selection_revision = self.selection_revision.wrapping_add(1);
            self.move_selection(-1);
            return self.scroll_to_selected(previous_rank, false);
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.launch_selected,
            &key,
            modifiers,
            physical_key,
        ) {
            if let Some(index) = self.selected_result_index() {
                return self.launch(index);
            }
        }

        Task::none()
    }

    pub(super) fn scroll_to_selected(
        &mut self,
        previous_rank: usize,
        force: bool,
    ) -> Task<Message> {
        if self.selected_result_index().is_none() {
            self.results_scroll_offset = 0.0;
            self.scroll_start_rank = 0;
            self.highlighted_rank = 0;
            return if force {
                operation::scroll_to(
                    self.results_scroll_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: None,
                        y: Some(0.0),
                    },
                )
            } else {
                Task::none()
            };
        }

        let total_rows = self.filtered_indices().len();
        let previous_offset = self.results_scroll_offset;
        let target_offset = super::state::scroll_offset_for_selection(
            self.selected_rank,
            previous_offset,
            self.layout.results_viewport_height(),
            total_rows,
            self.layout.result_row_height,
            self.layout.result_row_gap,
            self.layout.result_row_inset_y + 1.0,
        );
        self.results_scroll_offset = target_offset;
        self.scroll_start_rank = super::state::scroll_start_for_offset(
            target_offset,
            self.layout.result_row_scroll_step(),
            total_rows.saturating_sub(1),
        );
        let did_scroll = (target_offset - previous_offset).abs() > f32::EPSILON;

        if did_scroll && self.selected_rank != previous_rank {
            self.highlighted_rank = previous_rank;
            let revision = self.selection_revision;
            operation::scroll_to(
                self.results_scroll_id.clone(),
                scrollable::AbsoluteOffset {
                    x: None,
                    y: Some(target_offset),
                },
            )
            .chain(Task::done(Message::SyncHighlightedRank {
                revision,
                rank: self.selected_rank,
            }))
        } else {
            self.highlighted_rank = self.selected_rank;
            if !force && !did_scroll {
                Task::none()
            } else {
                operation::scroll_to(
                    self.results_scroll_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: None,
                        y: Some(target_offset),
                    },
                )
            }
        }
    }

    pub(super) fn on_results_scrolled(&mut self, viewport: scrollable::Viewport) -> Task<Message> {
        let row_step = self.layout.result_row_scroll_step();

        if row_step <= 0.0 {
            return Task::none();
        }

        let offset = super::state::clamp_scroll_offset(
            viewport.absolute_offset().y,
            self.filtered_indices().len(),
            viewport.bounds().height,
            self.layout.result_row_height,
            self.layout.result_row_gap,
        );

        let start = super::state::scroll_start_for_offset(
            offset,
            row_step,
            self.filtered_indices().len().saturating_sub(1),
        );

        self.results_scroll_offset = offset;
        self.scroll_start_rank = start;
        Task::none()
    }

    fn matches_shortcut(
        &self,
        binding: &crate::core::preferences::ShortcutBinding,
        key: &Key,
        modifiers: Modifiers,
        physical_key: keyboard::key::Physical,
    ) -> bool {
        if binding.ctrl != modifiers.control()
            || binding.alt != modifiers.alt()
            || binding.shift != modifiers.shift()
            || binding.super_key != modifiers.logo()
        {
            return false;
        }

        let pressed = match key.as_ref() {
            Key::Named(named) => named_key_name(named),
            Key::Character(_) => key
                .to_latin(physical_key)
                .map(|value| value.to_string())
                .or_else(|| match key.as_ref() {
                    Key::Character(value) => Some(value.to_string()),
                    _ => None,
                }),
            Key::Unidentified => None,
        };

        let Some(pressed) = pressed else {
            return false;
        };

        normalize_binding_key(&pressed) == binding.normalized_key()
    }
}

fn named_key_name(named: Named) -> Option<String> {
    match named {
        Named::Enter => Some("Enter".to_string()),
        Named::ArrowDown => Some("ArrowDown".to_string()),
        Named::ArrowUp => Some("ArrowUp".to_string()),
        Named::ArrowLeft => Some("ArrowLeft".to_string()),
        Named::ArrowRight => Some("ArrowRight".to_string()),
        Named::Escape => Some("Escape".to_string()),
        Named::Space => Some("Space".to_string()),
        Named::Tab => Some("Tab".to_string()),
        Named::Backspace => Some("Backspace".to_string()),
        Named::Delete => Some("Delete".to_string()),
        Named::Home => Some("Home".to_string()),
        Named::End => Some("End".to_string()),
        Named::PageUp => Some("PageUp".to_string()),
        Named::PageDown => Some("PageDown".to_string()),
        _ => None,
    }
}

fn normalize_binding_key(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '_', '-'], "")
}

#[cfg(test)]
mod tests {
    use super::normalize_binding_key;
    use crate::core::app_command::AppCommand;
    use crate::core::desktop::DesktopApp;
    use crate::ui::launcher::Launcher;
    use std::sync::mpsc;

    fn app(index: usize) -> DesktopApp {
        DesktopApp::new(
            format!("App {index}"),
            format!("/usr/bin/app-{index} %u"),
            format!("/usr/bin/app-{index}"),
            vec!["%u".to_string()],
            None,
            Vec::new(),
            None,
        )
    }

    fn launcher_with_results(total_results: usize) -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (mut launcher, _) = Launcher::new(rx, crate::core::tray::TrayController::detached());
        launcher.apps = (0..total_results).map(app).collect();
        launcher.all_app_indices = (0..launcher.apps.len()).collect();
        launcher.filtered_indices = launcher.all_app_indices.clone();
        launcher
    }

    #[test]
    fn binding_key_normalization_ignores_spacing_and_case() {
        assert_eq!(normalize_binding_key(" Arrow-Down "), "arrowdown");
        assert_eq!(normalize_binding_key("Page_Up"), "pageup");
    }

    #[test]
    fn scrolling_selection_uses_precise_pixel_offset() {
        let mut launcher = launcher_with_results(20);
        launcher.selected_rank = 5;

        let _ = launcher.scroll_to_selected(0, false);

        let row_step = launcher.layout.result_row_scroll_step();
        let row_top = launcher.selected_rank as f32 * row_step;
        let row_bottom = row_top + launcher.layout.result_row_height;
        let viewport_top = launcher.results_scroll_offset;
        let viewport_bottom = viewport_top + launcher.layout.results_viewport_height();

        assert!(row_top >= viewport_top + launcher.layout.result_row_inset_y);
        assert!(row_bottom <= viewport_bottom - launcher.layout.result_row_inset_y);
    }
}
