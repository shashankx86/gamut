mod bootstrap;
mod channel;
mod data;
mod display;
mod runtime;
mod scroll;
mod state;

use crate::core::app_command::AppCommand;
use crate::core::desktop::DesktopApp;
use crate::core::ipc::IpcCommand;
use iced::Size;
use iced::widget::scrollable;
use iced::window;
use iced_layershell::to_layer_message;
use std::path::PathBuf;

pub(in crate::ui) use state::Launcher;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub(super) enum Message {
    Tick,
    ScrollbarVisibilityTick,
    AppsLoaded(Vec<DesktopApp>),
    SearchResultsLoaded(crate::core::search::ApplicationSearchResponse),
    IconsResolved(Vec<(usize, Option<PathBuf>)>),
    QueryChanged(String),
    LaunchFirstMatch,
    ExpandResults,
    ActionButtonPressed,
    LaunchIndex(usize),
    AppCommand(AppCommand),
    IpcCommand(IpcCommand),
    KeyboardEvent(window::Id, iced::keyboard::Event),
    WindowEvent(window::Id, window::Event),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    ResultsScrolled(scrollable::Viewport),
    FatalError(String),
    MonitorSizeLoaded(Option<Size>),
    SyncHighlightedRank { revision: u64, rank: usize },
}
