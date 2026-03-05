mod actions;
mod subscription;
mod update;

use super::constants::{MAX_RESULTS, RESULTS_ANIMATION_SPEED};
use crate::core::desktop::{DesktopApp, load_apps, normalize_query};
use crate::core::ipc::{IpcCommand, start_listener};
use iced::widget;
use iced::{Task, window};
use iced_layershell::to_layer_message;
use std::sync::mpsc::{self, Receiver};

pub(super) struct Launcher {
    pub(super) apps: Vec<DesktopApp>,
    pub(super) query: String,
    pub(super) normalized_query: String,
    pub(super) input_id: widget::Id,
    pub(super) results_scroll_id: widget::Id,
    pub(super) window_id: Option<window::Id>,
    pub(super) ignore_unfocus_until: Option<std::time::Instant>,
    pub(super) selected_rank: usize,
    pub(super) scroll_start_rank: usize,
    filtered_indices: Vec<usize>,
    results_progress: f32,
    results_target: f32,
    ipc_receiver: Receiver<IpcCommand>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub(super) enum Message {
    Tick,
    AppsLoaded(Vec<DesktopApp>),
    QueryChanged(String),
    LaunchFirstMatch,
    LaunchIndex(usize),
    KeyboardEvent(window::Id, iced::keyboard::Event),
    WindowEvent(window::Id, window::Event),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    FatalError(String),
}

impl Launcher {
    pub(super) fn new() -> (Self, Task<Message>) {
        let input_id = widget::Id::unique();
        let results_scroll_id = widget::Id::unique();

        let (ipc_receiver, startup_task) = match start_listener() {
            Ok(receiver) => (
                receiver,
                Task::perform(async { load_apps() }, Message::AppsLoaded),
            ),
            Err(error) => {
                let (_tx, receiver) = mpsc::channel();
                (
                    receiver,
                    Task::done(Message::FatalError(format!(
                        "IPC listener unavailable: {error}. daemon mode unavailable."
                    ))),
                )
            }
        };

        (
            Self {
                apps: Vec::new(),
                query: String::new(),
                normalized_query: String::new(),
                input_id,
                results_scroll_id,
                window_id: None,
                ignore_unfocus_until: None,
                selected_rank: 0,
                scroll_start_rank: 0,
                filtered_indices: Vec::new(),
                results_progress: 0.0,
                results_target: 0.0,
                ipc_receiver,
            },
            startup_task,
        )
    }

    pub(super) fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    fn refresh_filtered_indices(&mut self) {
        self.filtered_indices = self
            .apps
            .iter()
            .enumerate()
            .filter_map(|(index, app)| {
                app.matches_normalized_query(&self.normalized_query)
                    .then_some(index)
            })
            .take(MAX_RESULTS)
            .collect();

        if self.filtered_indices.is_empty() {
            self.selected_rank = 0;
        } else {
            self.selected_rank = self.selected_rank.min(self.filtered_indices.len() - 1);
        }

        self.scroll_start_rank = self
            .scroll_start_rank
            .min(self.filtered_indices.len().saturating_sub(1));
    }

    fn clear_window_state(&mut self) {
        self.query.clear();
        self.normalized_query.clear();
        self.filtered_indices.clear();
        self.ignore_unfocus_until = None;
        self.selected_rank = 0;
        self.scroll_start_rank = 0;
        self.results_progress = 0.0;
        self.results_target = 0.0;
    }

    pub(super) fn results_progress(&self) -> f32 {
        self.results_progress
    }

    pub(super) fn sync_results_target_with_query(&mut self) {
        self.results_target = if self.normalized_query.is_empty() {
            0.0
        } else {
            1.0
        };
    }

    pub(super) fn animate_results(&mut self) {
        let delta = self.results_target - self.results_progress;
        if delta.abs() < 0.01 {
            self.results_progress = self.results_target;
            return;
        }

        self.results_progress =
            (self.results_progress + delta * RESULTS_ANIMATION_SPEED).clamp(0.0, 1.0);
    }

    pub(super) fn selected_result_index(&self) -> Option<usize> {
        if self.filtered_indices.is_empty() {
            return None;
        }

        self.filtered_indices
            .get(
                self.selected_rank
                    .min(self.filtered_indices.len().saturating_sub(1)),
            )
            .copied()
    }

    pub(super) fn move_selection(&mut self, offset: isize) {
        let count = self.filtered_indices.len();
        if count == 0 {
            self.selected_rank = 0;
            return;
        }

        let current = self.selected_rank.min(count.saturating_sub(1)) as isize;
        self.selected_rank = (current + offset).clamp(0, count as isize - 1) as usize;
    }

    pub(super) fn update_query(&mut self, query: String) {
        self.query = query;
        self.normalized_query = normalize_query(&self.query);
        self.refresh_filtered_indices();
        self.sync_results_target_with_query();
        self.selected_rank = 0;
        self.scroll_start_rank = 0;
    }

    pub(super) fn set_apps(&mut self, apps: Vec<DesktopApp>) {
        self.apps = apps;
        self.selected_rank = 0;
        self.scroll_start_rank = 0;
        self.refresh_filtered_indices();
    }

    pub(super) fn needs_fast_tick(&self) -> bool {
        self.window_id.is_some() || (self.results_target - self.results_progress).abs() > 0.01
    }
}
