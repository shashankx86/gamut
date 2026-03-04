mod actions;
mod subscription;
mod update;

use super::constants::MAX_RESULTS;
use crate::core::desktop::{DesktopApp, load_apps};
use crate::core::ipc::{IpcCommand, start_listener};
use iced::widget;
use iced::{Task, window};
use iced_layershell::to_layer_message;
use std::sync::mpsc::{self, Receiver};

pub(super) struct Launcher {
    pub(super) apps: Vec<DesktopApp>,
    pub(super) query: String,
    pub(super) input_id: widget::Id,
    pub(super) status: Option<String>,
    pub(super) window_id: Option<window::Id>,
    pub(super) had_focus: bool,
    pub(super) ignore_unfocus_until: Option<std::time::Instant>,
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
}

impl Launcher {
    pub(super) fn new() -> (Self, Task<Message>) {
        let input_id = widget::Id::unique();

        let (ipc_receiver, status) = match start_listener() {
            Ok(receiver) => (receiver, Some("Ready".to_string())),
            Err(error) => {
                let (_tx, receiver) = mpsc::channel();
                (
                    receiver,
                    Some(format!(
                        "IPC listener unavailable: {error}. daemon mode unavailable."
                    )),
                )
            }
        };

        (
            Self {
                apps: Vec::new(),
                query: String::new(),
                input_id,
                status,
                window_id: None,
                had_focus: false,
                ignore_unfocus_until: None,
                ipc_receiver,
            },
            Task::perform(async { load_apps() }, Message::AppsLoaded),
        )
    }

    pub(super) fn filtered_indices(&self) -> Vec<usize> {
        self.apps
            .iter()
            .enumerate()
            .filter_map(|(index, app)| app.matches_query(&self.query).then_some(index))
            .take(MAX_RESULTS)
            .collect()
    }

    fn clear_window_state(&mut self) {
        self.query.clear();
        self.had_focus = false;
        self.ignore_unfocus_until = None;
    }
}
