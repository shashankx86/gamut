use super::super::constants::{RESULT_ROW_SCROLL_STEP, RESULTS_HEIGHT, UNFOCUS_GUARD_MS};
use super::{Launcher, Message};
use crate::core::ipc::IpcCommand;
use iced::keyboard::{self, Key, key::Named};
use iced::widget;
use iced::widget::scrollable;
use iced::{Task, window};
use std::time::{Duration, Instant};

impl Launcher {
    pub(in crate::ui) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.on_tick(),
            Message::AppsLoaded(apps) => {
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
                    self.scroll_to_selected(true),
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
            Message::FatalError(error) => {
                eprintln!("{error}");
                iced::exit()
            }
            _ => Task::none(),
        }
    }

    fn on_tick(&mut self) -> Task<Message> {
        self.animate_results();
        Task::none()
    }

    fn handle_ipc_command(&mut self, command: IpcCommand) -> Task<Message> {
        match command {
            IpcCommand::Toggle => {
                if self.is_visible {
                    self.hide_launcher()
                } else {
                    self.show_launcher()
                }
            }
            IpcCommand::Quit => iced::exit(),
            IpcCommand::Ping => Task::none(),
        }
    }

    fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        if self.window_id != Some(id) || !self.is_visible {
            return Task::none();
        }

        self.ignore_unfocus_until = Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));

        Task::batch(vec![
            widget::operation::focus(self.input_id.clone()),
            widget::operation::move_cursor_to_end(self.input_id.clone()),
            self.request_icon_resolution_for_visible(),
        ])
    }

    fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        if self.window_id == Some(id) {
            self.window_id = None;
            self.is_visible = false;
            self.clear_window_state();
        }

        Task::none()
    }

    fn handle_key_event(&mut self, id: window::Id, event: keyboard::Event) -> Task<Message> {
        if self.window_id != Some(id) || !self.is_visible {
            return Task::none();
        }

        match event {
            keyboard::Event::KeyPressed { key, .. }
                if matches!(key.as_ref(), Key::Named(Named::Escape)) =>
            {
                self.hide_launcher()
            }
            keyboard::Event::KeyPressed { key, .. }
                if matches!(key.as_ref(), Key::Named(Named::ArrowDown)) =>
            {
                if self.normalized_query.is_empty() && self.results_target == 0.0 {
                    self.results_target = 1.0;
                    self.manually_expanded = true;
                    Task::none()
                } else {
                    self.move_selection(1);
                    self.scroll_to_selected(false)
                }
            }
            keyboard::Event::KeyPressed { key, .. }
                if matches!(key.as_ref(), Key::Named(Named::ArrowUp)) =>
            {
                self.move_selection(-1);
                self.scroll_to_selected(false)
            }
            _ => Task::none(),
        }
    }

    fn scroll_to_selected(&mut self, force: bool) -> Task<Message> {
        if self.selected_result_index().is_none() {
            self.scroll_start_rank = 0;
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

        let visible_rows = ((RESULTS_HEIGHT / RESULT_ROW_SCROLL_STEP).floor() as usize).max(1);
        let total_rows = self.filtered_indices().len();
        let max_start = total_rows.saturating_sub(visible_rows);
        let previous_start = self.scroll_start_rank.min(max_start);
        let mut start = previous_start;

        if self.selected_rank < start {
            start = self.selected_rank;
        } else if self.selected_rank >= start + visible_rows {
            start = self
                .selected_rank
                .saturating_sub(visible_rows.saturating_sub(1));
        }

        start = start.min(max_start);
        self.scroll_start_rank = start;

        if !force && start == previous_start {
            Task::none()
        } else {
            widget::operation::scroll_to(
                self.results_scroll_id.clone(),
                scrollable::AbsoluteOffset {
                    x: None,
                    y: Some(start as f32 * RESULT_ROW_SCROLL_STEP),
                },
            )
        }
    }

    fn handle_window_event(&mut self, id: window::Id, event: window::Event) -> Task<Message> {
        if self.window_id != Some(id) || !self.is_visible {
            return Task::none();
        }

        match event {
            window::Event::Focused => {
                self.ignore_unfocus_until = None;
                Task::none()
            }
            window::Event::Unfocused if !self.should_ignore_unfocus() => self.hide_launcher(),
            window::Event::CloseRequested => self.hide_launcher(),
            _ => Task::none(),
        }
    }
}
