use super::channel::{new_app_command_handle, new_ipc_receiver_handle, new_search_results_handle};
use super::state::LauncherVisualCache;
use super::{Launcher, Message};
use crate::core::app_command::AppCommand;
use crate::core::desktop::{load_cached_app_catalog, refresh_app_cache};
use crate::core::ipc::start_listener;
use crate::core::preferences::load_preferences;
use crate::core::search::ApplicationSearchEngine;
use crate::core::tray::TrayController;
use crate::ui::constants::MAX_RESULTS;
use crate::ui::layout::{LauncherLayout, LauncherPreferences};
use iced::Task;
use iced::keyboard::Modifiers;
use iced::widget;
use std::sync::mpsc::{self, Receiver};

impl Launcher {
    pub(in crate::ui) fn new(
        command_receiver: Receiver<AppCommand>,
        tray_controller: TrayController,
    ) -> (Self, Task<Message>) {
        let layout_preferences = LauncherPreferences::load_from_env();
        let app_preferences = load_preferences();
        let layout = LauncherLayout::from_monitor_size(None, &layout_preferences, &app_preferences);
        let visual_cache = LauncherVisualCache::build(&app_preferences);
        let input_id = widget::Id::unique();
        let results_scroll_id = widget::Id::unique();
        let cached_catalog = load_cached_app_catalog();
        let (app_search_engine, search_results_receiver) =
            ApplicationSearchEngine::spawn(MAX_RESULTS);
        let (ipc_handle, startup_task) = match start_listener() {
            Ok(receiver) => (
                new_ipc_receiver_handle(receiver),
                if cached_catalog.needs_refresh {
                    Task::perform(async { refresh_app_cache() }, Message::AppsLoaded)
                } else {
                    Task::none()
                },
            ),
            Err(error) => {
                let (_tx, receiver) = mpsc::channel();
                (
                    new_ipc_receiver_handle(receiver),
                    Task::done(Message::FatalError(format!(
                        "IPC listener unavailable: {error}. daemon mode unavailable."
                    ))),
                )
            }
        };

        let mut launcher = Self {
            apps: Vec::new(),
            all_app_indices: Vec::new(),
            layout,
            query: String::new(),
            normalized_query: String::new(),
            input_id,
            results_scroll_id,
            launcher_window_id: None,
            monitor_size: None,
            target_output: None,
            is_visible: false,
            ignore_unfocus_until: None,
            selected_rank: 0,
            highlighted_rank: 0,
            selection_revision: 0,
            search_generation: 0,
            applied_search_generation: 0,
            search_in_flight: false,
            results_scroll_offset: 0.0,
            scroll_start_rank: 0,
            results_scrollbar_visibility: super::scroll::ResultsScrollbarVisibility::default(),
            filtered_indices: Vec::new(),
            results_progress: 0.0,
            results_target: 0.0,
            manually_expanded: false,
            progress_config:
                super::runtime::progress::ProgressConfig::indexing_update_indeterminate(),
            progress_indicator: super::runtime::progress::ProgressIndicator::default(),
            layout_preferences,
            app_preferences,
            visual_cache,
            tray_controller,
            modifiers: Modifiers::default(),
            suppress_alt_actions_until_release: false,
            action_overlay_pinned: false,
            suppress_next_query_change: false,
            icon_resolve_in_flight: false,
            app_refresh_in_flight: false,
            app_refresh_started_at: None,
            last_app_refresh_at: None,
            app_search_engine,
            search_results_handle: new_search_results_handle(search_results_receiver),
            ipc_handle,
            app_command_handle: new_app_command_handle(command_receiver),
        };

        if !cached_catalog.apps.is_empty() {
            launcher.set_apps(cached_catalog.apps);
        }

        (launcher, startup_task)
    }
}
