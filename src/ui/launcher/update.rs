use super::{Launcher, Message};
use crate::core::app_command::AppCommand;
use crate::core::ipc::IpcCommand;
use iced::Task;
use log::{error, info};

impl Launcher {
    pub(in crate::ui) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.on_tick(),
            Message::AppsLoaded(apps) => {
                info!("loaded {} desktop applications", apps.len());
                self.set_apps(apps);
                self.request_icon_resolution_for_visible()
            }
            Message::SearchResultsLoaded(response) => {
                if self.apply_search_results(response) {
                    Task::batch(vec![
                        self.scroll_to_selected(self.selected_rank, true),
                        self.request_icon_resolution_for_visible(),
                    ])
                } else {
                    Task::none()
                }
            }
            Message::IconsResolved(updates) => {
                self.apply_resolved_icons(updates);
                self.request_icon_resolution_for_visible()
            }
            Message::QueryChanged(query) => {
                self.update_query(query);
                self.scroll_to_selected(self.selected_rank, true)
            }
            Message::LaunchFirstMatch => {
                if let Some(index) = self.selected_result_index() {
                    self.launch(index)
                } else {
                    Task::none()
                }
            }
            Message::ExpandResults => self.expand_results(),
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
                self.target_output = target_output;
                self.show_launcher()
            }
            AppCommand::ToggleLauncher { target_output } => {
                self.target_output = target_output;
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
}
