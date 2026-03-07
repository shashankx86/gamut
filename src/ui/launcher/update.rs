use super::super::constants::UNFOCUS_GUARD_MS;
use super::preferences::{normalize_hex_color, shortcut_preferences_from_editor};
use super::{Launcher, Message, ShortcutAction, ThemeColorField};
use crate::core::ipc::IpcCommand;
use crate::core::preferences::{
    LauncherPlacement, LauncherSize, RadiusPreference, ThemePreference,
};
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget;
use iced::widget::scrollable;
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
            Message::IpcCommand(command) => self.handle_ipc_command(command),
            Message::WindowOpened(id) => self.on_window_opened(id),
            Message::WindowClosed(id) => self.on_window_closed(id),
            Message::KeyboardEvent(id, key_event) => self.handle_key_event(id, key_event),
            Message::WindowEvent(id, window_event) => self.handle_window_event(id, window_event),
            Message::MonitorSizeLoaded(size) => self.update_layout(size),
            Message::PreferencesThemeSelected(theme) => self.update_theme_preference(theme),
            Message::PreferencesRadiusSelected(radius) => self.update_radius_preference(radius),
            Message::PreferencesSizeSelected(size) => self.update_size_preference(size),
            Message::PreferencesPlacementSelected(placement) => {
                self.update_placement_preference(placement)
            }
            Message::PreferencesCustomRadiusChanged(radius) => self.update_custom_radius(radius),
            Message::PreferencesCustomTopMarginChanged(top_margin) => {
                self.update_custom_top_margin(top_margin)
            }
            Message::PreferencesThemeColorChanged(field, value) => {
                self.update_custom_theme_color(field, value)
            }
            Message::PreferencesShortcutChanged(action, value) => {
                self.update_shortcut_binding(action, value)
            }
            Message::PreferencesCaptureShortcut(action) => self.capture_shortcut(action),
            Message::PreferencesCloseRequested => self.close_preferences_window(),
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
        match command {
            IpcCommand::Show => self.show_launcher(),
            IpcCommand::Toggle => {
                if self.is_visible {
                    self.hide_launcher()
                } else {
                    self.show_launcher()
                }
            }
            IpcCommand::OpenPreferences => self.open_preferences_window(),
            IpcCommand::ReloadPreferences => self.reload_preferences_from_disk(),
            IpcCommand::Quit => iced::exit(),
            IpcCommand::Ping => Task::none(),
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
                widget::operation::focus(self.input_id.clone()),
                widget::operation::move_cursor_to_end(self.input_id.clone()),
                self.request_icon_resolution_for_visible(),
            ]);
        }

        if self.is_preferences_window(id) {
            return widget::operation::focus(self.preferences_scroll_id.clone());
        }

        Task::none()
    }

    fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        if self.is_launcher_window(id) {
            self.launcher_window_id = None;
            self.is_visible = false;
            self.clear_window_state();
        }

        if self.is_preferences_window(id) {
            self.preferences_window_id = None;
            self.preferences_editor.set_theme_error(None);
            self.preferences_editor.set_shortcut_error(None);
            self.preferences_editor.set_save_error(None);
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

                if self.is_preferences_window(id) {
                    return self.handle_preferences_key_press(key, modifiers, physical_key);
                }

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

    fn handle_preferences_key_press(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        physical_key: keyboard::key::Physical,
    ) -> Task<Message> {
        if matches!(key.as_ref(), Key::Named(Named::Escape)) {
            return self.close_preferences_window();
        }

        if modifiers.control() && matches!(key.as_ref(), Key::Character("w" | "W")) {
            return self.close_preferences_window();
        }

        if modifiers.control() && matches!(key.as_ref(), Key::Character("r" | "R")) {
            return self.reload_preferences_from_disk();
        }

        if modifiers.control() && matches!(key.as_ref(), Key::Character("l" | "L")) {
            return self.show_launcher();
        }

        let _ = physical_key;
        Task::none()
    }

    fn scroll_to_selected(&mut self, previous_rank: usize, force: bool) -> Task<Message> {
        if self.selected_result_index().is_none() {
            self.scroll_start_rank = 0;
            self.highlighted_rank = 0;
            return if force {
                widget::operation::scroll_to(
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
            widget::operation::scroll_to(
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
                widget::operation::scroll_to(
                    self.results_scroll_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: None,
                        y: Some(start as f32 * self.layout.result_row_scroll_step()),
                    },
                )
            }
        }
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

        if self.is_preferences_window(id) {
            return match event {
                window::Event::CloseRequested => self.close_preferences_window(),
                _ => Task::none(),
            };
        }

        Task::none()
    }

    fn update_theme_preference(&mut self, theme: ThemePreference) -> Task<Message> {
        self.app_preferences.appearance.theme = theme;
        self.preferences_editor.set_theme_error(None);
        self.persist_preferences()
    }

    fn update_radius_preference(&mut self, radius: RadiusPreference) -> Task<Message> {
        self.app_preferences.appearance.radius = radius;
        self.persist_preferences()
    }

    fn update_size_preference(&mut self, size: LauncherSize) -> Task<Message> {
        self.app_preferences.layout.size = size;
        self.persist_preferences()
    }

    fn update_placement_preference(&mut self, placement: LauncherPlacement) -> Task<Message> {
        self.app_preferences.layout.placement = placement;
        self.persist_preferences()
    }

    fn update_custom_radius(&mut self, radius: f32) -> Task<Message> {
        self.app_preferences.appearance.custom_radius = radius;
        self.persist_preferences()
    }

    fn update_custom_top_margin(&mut self, top_margin: f32) -> Task<Message> {
        self.app_preferences.layout.custom_top_margin = top_margin;
        self.persist_preferences()
    }

    fn update_custom_theme_color(
        &mut self,
        field: ThemeColorField,
        value: String,
    ) -> Task<Message> {
        self.preferences_editor
            .set_theme_value(field, value.clone());

        let Some(normalized) = normalize_hex_color(&value) else {
            self.preferences_editor.set_theme_error(Some(
                "Custom colors must use 6 or 8 digit hexadecimal values like #151516 or #151516FF."
                    .to_string(),
            ));
            return Task::none();
        };

        self.preferences_editor.set_theme_error(None);
        match field {
            ThemeColorField::Background => {
                self.app_preferences.appearance.custom_theme.background = normalized;
            }
            ThemeColorField::Text => {
                self.app_preferences.appearance.custom_theme.text = normalized;
            }
            ThemeColorField::Accent => {
                self.app_preferences.appearance.custom_theme.accent = normalized;
            }
        }

        self.preferences_editor
            .sync_from_preferences(&self.app_preferences);
        self.persist_preferences()
    }

    fn update_shortcut_binding(&mut self, action: ShortcutAction, value: String) -> Task<Message> {
        self.preferences_editor.set_shortcut_value(action, value);
        self.apply_shortcut_editor()
    }

    fn capture_shortcut(&mut self, action: ShortcutAction) -> Task<Message> {
        let mut parts = Vec::new();
        if self.modifiers.control() {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.alt() {
            parts.push("Alt".to_string());
        }
        if self.modifiers.shift() {
            parts.push("Shift".to_string());
        }
        if self.modifiers.logo() {
            parts.push("Super".to_string());
        }

        if parts.is_empty() {
            self.preferences_editor.set_shortcut_error(Some(
                "Hold the modifiers you want and press the capture button again, or type the shortcut manually."
                    .to_string(),
            ));
            return Task::none();
        }

        parts.push("Enter".to_string());
        self.preferences_editor
            .set_shortcut_value(action, parts.join("+"));
        self.apply_shortcut_editor()
    }

    fn apply_shortcut_editor(&mut self) -> Task<Message> {
        match shortcut_preferences_from_editor(&self.preferences_editor) {
            Ok(shortcuts) => {
                self.preferences_editor.set_shortcut_error(None);
                self.app_preferences.shortcuts = shortcuts;
                self.persist_preferences()
            }
            Err(error) => {
                self.preferences_editor.set_shortcut_error(Some(error));
                Task::none()
            }
        }
    }

    fn persist_preferences(&mut self) -> Task<Message> {
        match crate::core::preferences::save_preferences(&self.app_preferences) {
            Ok(()) => {
                self.preferences_editor.set_save_error(None);
                self.preferences_editor
                    .sync_from_preferences(&self.app_preferences);
                self.update_layout(self.monitor_size)
            }
            Err(error) => {
                self.preferences_editor
                    .set_save_error(Some(format!("Failed to save preferences: {error}")));
                Task::none()
            }
        }
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
