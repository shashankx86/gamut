use super::super::constants::UNFOCUS_GUARD_MS;
use super::{Launcher, Message};
use crate::core::app_command::AppCommand;
use crate::core::ipc::IpcCommand;
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::{operation, scrollable};
use iced::{Task, window};
use log::{error, info};
use std::time::{Duration, Instant};

impl Launcher {
    pub(in crate::ui) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.on_tick(),
            Message::AppsLoaded(apps) => {
                info!("loaded {} desktop applications", apps.len());
                self.set_apps(apps);
                self.request_icon_resolution_for_visible()
            }
            Message::IconsResolved(updates) => {
                self.apply_resolved_icons(updates);
                self.request_icon_resolution_for_visible()
            }
            Message::QueryChanged(query) => {
                self.update_query(query);
                Task::batch(vec![
                    self.scroll_to_selected(self.selected_rank, true),
                    self.request_icon_resolution_for_visible(),
                ])
            }
            Message::LaunchFirstMatch => {
                if let Some(index) = self.selected_result_index() {
                    self.launch(index)
                } else {
                    Task::none()
                }
            }
            Message::LaunchIndex(index) => self.launch(index),
            Message::AppCommand(command) => self.handle_app_command(command),
            Message::IpcCommand(command) => self.handle_ipc_command(command),
            Message::WindowOpened(id) => self.on_window_opened(id),
            Message::WindowClosed(id) => self.on_window_closed(id),
            Message::ResultsScrolled(viewport) => self.on_results_scrolled(viewport),
            Message::KeyboardEvent(id, key_event) => self.handle_key_event(id, key_event),
            Message::WindowEvent(id, window_event) => self.handle_window_event(id, window_event),
            Message::MonitorSizeLoaded(size) => self.update_layout(size),
            Message::SyncHighlightedRank { revision, rank } => {
                self.sync_highlighted_rank(revision, rank);
                Task::none()
            }
            Message::FatalError(error) => {
                error!("{error}");
                iced::exit()
            }
            _ => Task::none(),
        }
    }

    fn on_tick(&mut self) -> Task<Message> {
        self.animate_results()
    }

    fn handle_ipc_command(&mut self, command: IpcCommand) -> Task<Message> {
        match AppCommand::from_ipc(command) {
            Some(command) => self.handle_app_command(command),
            None => Task::none(),
        }
    }

    fn handle_app_command(&mut self, command: AppCommand) -> Task<Message> {
        match command {
            AppCommand::ShowLauncher { target_output } => {
                self.target_output_name = target_output;
                self.show_launcher()
            }
            AppCommand::ToggleLauncher { target_output } => {
                self.target_output_name = target_output;
                if self.is_visible {
                    self.hide_launcher()
                } else {
                    self.show_launcher()
                }
            }
            AppCommand::OpenPreferences => self.open_preferences_window(),
            AppCommand::ReloadPreferences => self.reload_preferences_from_disk(),
            AppCommand::Quit => iced::exit(),
        }
    }

    fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        if self.is_launcher_window(id) {
            let monitor_size = window::monitor_size(id).map(Message::MonitorSizeLoaded);

            if !self.is_visible {
                return monitor_size;
            }

            self.ignore_unfocus_until =
                Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));

            return Task::batch(vec![
                monitor_size,
                operation::focus(self.input_id.clone()),
                operation::move_cursor_to_end(self.input_id.clone()),
                self.request_icon_resolution_for_visible(),
            ]);
        }

        Task::none()
    }

    fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        if self.is_launcher_window(id) {
            self.launcher_window_id = None;
            self.is_visible = false;
            self.clear_window_state();
        }

        Task::none()
    }

    fn handle_key_event(&mut self, id: window::Id, event: keyboard::Event) -> Task<Message> {
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

    fn handle_launcher_key_press(
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

    fn scroll_to_selected(&mut self, previous_rank: usize, force: bool) -> Task<Message> {
        if self.selected_result_index().is_none() {
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

        let visible_rows = self.layout.visible_result_rows();
        let total_rows = self.filtered_indices().len();
        let previous_start = self.scroll_start_rank;
        let start = super::state::scroll_start_for_selection(
            self.selected_rank,
            previous_start,
            total_rows,
            visible_rows,
        );
        self.scroll_start_rank = start;
        let did_scroll = start != previous_start;

        if did_scroll && self.selected_rank != previous_rank {
            self.highlighted_rank = previous_rank;
            let revision = self.selection_revision;
            operation::scroll_to(
                self.results_scroll_id.clone(),
                scrollable::AbsoluteOffset {
                    x: None,
                    y: Some(start as f32 * self.layout.result_row_scroll_step()),
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
                        y: Some(start as f32 * self.layout.result_row_scroll_step()),
                    },
                )
            }
        }
    }

    fn on_results_scrolled(&mut self, viewport: scrollable::Viewport) -> Task<Message> {
        let row_step = self.layout.result_row_scroll_step();

        if row_step <= 0.0 {
            return Task::none();
        }

        let max_start = self
            .filtered_indices()
            .len()
            .saturating_sub(self.layout.visible_result_rows());
        let start = (viewport.absolute_offset().y / row_step).round().max(0.0) as usize;

        self.scroll_start_rank = start.min(max_start);
        Task::none()
    }

    fn handle_window_event(&mut self, id: window::Id, event: window::Event) -> Task<Message> {
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
