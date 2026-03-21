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
                self.finish_app_refresh();
                info!("loaded {} desktop applications", apps.len());
                self.set_apps(apps);
                self.request_icon_resolution_for_visible()
            }
            Message::SearchResultsLoaded(response) => {
                if self.apply_search_results(response) {
                    self.request_icon_resolution_for_visible()
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
        let mut tasks = vec![self.animate_results()];

        if self.is_visible {
            tasks.push(self.request_app_refresh(false));
        }

        Task::batch(tasks)
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
            AppCommand::ReloadPreferences => self.reload_preferences_from_disk(),
            AppCommand::Quit => iced::exit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::app_command::AppCommand;
    use crate::core::desktop::DesktopApp;
    use crate::core::search::ApplicationSearchResponse;
    use std::sync::mpsc;

    fn launcher() -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (launcher, _) = Launcher::new(rx, crate::core::tray::TrayController::detached());
        launcher
    }

    fn app(name: &str) -> DesktopApp {
        DesktopApp::new(
            name.to_string(),
            "Application".to_string(),
            format!("/usr/bin/{name}"),
            format!("/usr/bin/{name}"),
            Vec::new(),
            None,
            Vec::new(),
            None,
        )
    }

    #[test]
    fn refresh_command_replaces_apps_and_clears_inflight_state() {
        let mut launcher = launcher();

        let _ = launcher.request_app_refresh(true);
        assert!(launcher.app_refresh_in_flight());

        let _ = launcher.update(Message::AppsLoaded(vec![app("new-app")]));

        assert!(!launcher.app_refresh_in_flight());
        assert_eq!(launcher.app_count(), 1);
    }

    #[test]
    fn search_results_do_not_force_scroll_back_to_selection() {
        let mut launcher = launcher();
        launcher.set_apps((0..20).map(|index| app(&format!("app-{index}"))).collect());
        launcher.results_scroll_offset = 232.0;

        let _ = launcher.update(Message::SearchResultsLoaded(ApplicationSearchResponse {
            generation: 0,
            matches: (0..20).collect(),
        }));

        assert_eq!(launcher.results_scroll_offset, 232.0);
    }
}
