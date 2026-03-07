use super::super::constants::UNFOCUS_GUARD_MS;
use super::{Launcher, Message};
use crate::core::ipc::IpcCommand;
use iced::keyboard::{self, key::Named, Key};
use iced::widget;
use iced::widget::scrollable;
use iced::{window, Task};
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
            IpcCommand::ReloadPreferences => Task::none(),
            IpcCommand::Quit => iced::exit(),
            IpcCommand::Ping => Task::none(),
        }
    }

    fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        if self.window_id != Some(id) {
            return Task::none();
        }

        let monitor_size = window::monitor_size(id).map(Message::MonitorSizeLoaded);

        if !self.is_visible {
            return monitor_size;
        }

        self.ignore_unfocus_until = Some(Instant::now() + Duration::from_millis(UNFOCUS_GUARD_MS));

        Task::batch(vec![
            monitor_size,
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
                    let previous_rank = self.selected_rank;
                    self.selection_revision = self.selection_revision.wrapping_add(1);
                    self.move_selection(1);
                    self.scroll_to_selected(previous_rank, false)
                }
            }
            keyboard::Event::KeyPressed { key, .. }
                if matches!(key.as_ref(), Key::Named(Named::ArrowUp)) =>
            {
                let previous_rank = self.selected_rank;
                self.selection_revision = self.selection_revision.wrapping_add(1);
                self.move_selection(-1);
                self.scroll_to_selected(previous_rank, false)
            }
            _ => Task::none(),
        }
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
