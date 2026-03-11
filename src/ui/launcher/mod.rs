mod actions;
mod preferences;
mod state;
mod subscription;
mod update;

use super::constants::MAX_RESULTS;
use super::layout::{LauncherLayout, LauncherPreferences};
use super::surface::launcher_hidden_surface_settings;
use super::theme::{ResolvedAppearance, resolve_appearance, resolve_theme};
use crate::core::desktop::{
    DesktopApp, IconResolveRequest, load_apps, normalize_query, resolve_icon_requests,
};
use crate::core::ipc::{IpcCommand, start_listener};
use crate::core::preferences::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ThemePreference,
    load_preferences,
};
use iced::Size;
use iced::keyboard::Modifiers;
use iced::widget::{self, Id};
use iced::{Task, window};
use iced_layershell::to_layer_message;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

pub(in crate::ui) use preferences::{PreferencesEditor, ShortcutAction, ThemeColorField};
pub(in crate::ui) use state::{
    expansion_render_range, is_manual_expansion_in_progress, spacer_height_for_rows,
};

const ICON_RESOLVE_BATCH_SIZE: usize = 24;
const IPC_SUBSCRIPTION_ID: u64 = 1;
pub(in crate::ui) const THEME_OPTIONS: [ThemePreference; 4] = [
    ThemePreference::Dark,
    ThemePreference::Light,
    ThemePreference::System,
    ThemePreference::Custom,
];
pub(in crate::ui) const RADIUS_OPTIONS: [RadiusPreference; 4] = [
    RadiusPreference::Small,
    RadiusPreference::Medium,
    RadiusPreference::Large,
    RadiusPreference::Custom,
];
pub(in crate::ui) const SIZE_OPTIONS: [LauncherSize; 4] = [
    LauncherSize::Small,
    LauncherSize::Medium,
    LauncherSize::Large,
    LauncherSize::ExtraLarge,
];
pub(in crate::ui) const PLACEMENT_OPTIONS: [LauncherPlacement; 3] = [
    LauncherPlacement::RaisedCenter,
    LauncherPlacement::Center,
    LauncherPlacement::Custom,
];

#[derive(Clone)]
pub(super) struct IpcReceiverHandle {
    id: u64,
    receiver: Arc<Mutex<Receiver<IpcCommand>>>,
}

impl IpcReceiverHandle {
    fn new(receiver: Receiver<IpcCommand>) -> Self {
        Self {
            id: IPC_SUBSCRIPTION_ID,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

impl Hash for IpcReceiverHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub(super) struct Launcher {
    pub(super) apps: Vec<DesktopApp>,
    pub(super) layout: LauncherLayout,
    pub(super) query: String,
    pub(super) normalized_query: String,
    pub(super) input_id: widget::Id,
    pub(super) results_scroll_id: widget::Id,
    pub(super) launcher_window_id: Option<window::Id>,
    pub(super) preferences_window_id: Option<window::Id>,
    pub(super) monitor_size: Option<Size>,
    pub(super) is_visible: bool,
    pub(super) ignore_unfocus_until: Option<std::time::Instant>,
    pub(super) selected_rank: usize,
    pub(super) highlighted_rank: usize,
    selection_revision: u64,
    pub(super) scroll_start_rank: usize,
    pub(super) filtered_indices: Vec<usize>,
    pub(super) results_progress: f32,
    pub(super) results_target: f32,
    pub(super) manually_expanded: bool,
    layout_preferences: LauncherPreferences,
    pub(super) app_preferences: AppPreferences,
    pub(super) preferences_editor: PreferencesEditor,
    pub(super) preferences_scroll_id: Id,
    pub(super) custom_background_input_id: Id,
    pub(super) custom_text_input_id: Id,
    pub(super) custom_accent_input_id: Id,
    pub(super) shortcut_launch_input_id: Id,
    pub(super) shortcut_down_input_id: Id,
    pub(super) shortcut_up_input_id: Id,
    pub(super) shortcut_close_input_id: Id,
    modifiers: Modifiers,
    icon_resolve_in_flight: bool,
    ipc_handle: IpcReceiverHandle,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub(super) enum Message {
    Tick,
    AppsLoaded(Vec<DesktopApp>),
    IconsResolved(Vec<(usize, Option<PathBuf>)>),
    QueryChanged(String),
    LaunchFirstMatch,
    LaunchIndex(usize),
    IpcCommand(IpcCommand),
    KeyboardEvent(window::Id, iced::keyboard::Event),
    WindowEvent(window::Id, window::Event),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    PreferencesThemeSelected(ThemePreference),
    PreferencesRadiusSelected(RadiusPreference),
    PreferencesSizeSelected(LauncherSize),
    PreferencesPlacementSelected(LauncherPlacement),
    PreferencesCustomRadiusChanged(f32),
    PreferencesCustomTopMarginChanged(f32),
    PreferencesThemeColorChanged(ThemeColorField, String),
    PreferencesShortcutChanged(ShortcutAction, String),
    PreferencesCaptureShortcut(ShortcutAction),
    PreferencesCloseRequested,
    FatalError(String),
    MonitorSizeLoaded(Option<Size>),
    SyncHighlightedRank { revision: u64, rank: usize },
}

impl Launcher {
    pub(super) fn new() -> (Self, Task<Message>) {
        let layout_preferences = LauncherPreferences::load_from_env();
        let app_preferences = load_preferences();
        let layout = LauncherLayout::from_monitor_size(None, &layout_preferences, &app_preferences);
        let preferences_editor = PreferencesEditor::from_preferences(&app_preferences);
        let input_id = widget::Id::unique();
        let results_scroll_id = widget::Id::unique();
        let hidden_window_id = window::Id::unique();

        let (ipc_handle, startup_task) = match start_listener() {
            Ok(receiver) => (
                IpcReceiverHandle::new(receiver),
                Task::batch(vec![
                    Task::done(Message::NewLayerShell {
                        settings: launcher_hidden_surface_settings(&layout),
                        id: hidden_window_id,
                    }),
                    Task::perform(async { load_apps() }, Message::AppsLoaded),
                ]),
            ),
            Err(error) => {
                let (_tx, receiver) = mpsc::channel();
                (
                    IpcReceiverHandle::new(receiver),
                    Task::done(Message::FatalError(format!(
                        "IPC listener unavailable: {error}. daemon mode unavailable."
                    ))),
                )
            }
        };

        (
            Self {
                apps: Vec::new(),
                layout,
                query: String::new(),
                normalized_query: String::new(),
                input_id,
                results_scroll_id,
                launcher_window_id: Some(hidden_window_id),
                preferences_window_id: None,
                monitor_size: None,
                is_visible: false,
                ignore_unfocus_until: None,
                selected_rank: 0,
                highlighted_rank: 0,
                selection_revision: 0,
                scroll_start_rank: 0,
                filtered_indices: Vec::new(),
                results_progress: 0.0,
                results_target: 0.0,
                manually_expanded: false,
                layout_preferences,
                app_preferences,
                preferences_editor,
                preferences_scroll_id: Id::unique(),
                custom_background_input_id: Id::unique(),
                custom_text_input_id: Id::unique(),
                custom_accent_input_id: Id::unique(),
                shortcut_launch_input_id: Id::unique(),
                shortcut_down_input_id: Id::unique(),
                shortcut_up_input_id: Id::unique(),
                shortcut_close_input_id: Id::unique(),
                modifiers: Modifiers::default(),
                icon_resolve_in_flight: false,
                ipc_handle,
            },
            startup_task,
        )
    }

    pub(super) fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    fn refresh_filtered_indices(&mut self) {
        let mut ranked_matches: Vec<(usize, i32)> = if self.normalized_query.is_empty() {
            self.apps
                .iter()
                .enumerate()
                .map(|(index, _)| (index, 0))
                .collect()
        } else {
            self.apps
                .iter()
                .enumerate()
                .filter_map(|(index, app)| {
                    app.query_match_score(&self.normalized_query)
                        .map(|score| (index, score))
                })
                .collect()
        };

        if !self.normalized_query.is_empty() {
            ranked_matches.sort_by(|(left_index, left_score), (right_index, right_score)| {
                right_score
                    .cmp(left_score)
                    .then_with(|| {
                        self.apps[*left_index]
                            .name
                            .cmp(&self.apps[*right_index].name)
                    })
                    .then_with(|| left_index.cmp(right_index))
            });
        }

        self.filtered_indices = ranked_matches
            .into_iter()
            .take(MAX_RESULTS)
            .map(|(index, _)| index)
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
        self.selection_revision = self.selection_revision.wrapping_add(1);
        self.refresh_filtered_indices();
        self.ignore_unfocus_until = None;
        self.selected_rank = 0;
        self.highlighted_rank = 0;
        self.scroll_start_rank = 0;
        self.results_progress = 0.0;
        self.results_target = 0.0;
        self.manually_expanded = false;
        self.icon_resolve_in_flight = false;
    }

    pub(super) fn results_progress(&self) -> f32 {
        self.results_progress
    }

    pub(super) fn sync_results_target_with_query(&mut self) {
        self.results_target =
            state::results_target(self.normalized_query.is_empty(), self.manually_expanded);
    }

    pub(super) fn animate_results(&mut self) -> Task<Message> {
        let step = state::animate_results(self.results_progress, self.results_target, &self.layout);
        self.results_progress = step.next_progress;

        let Some(id) = self.launcher_window_id else {
            return Task::none();
        };

        match step.surface_resize {
            state::SurfaceResize::None => Task::none(),
            state::SurfaceResize::Expanded => Task::done(Message::SizeChange {
                id,
                size: self.layout.expanded_surface_size(),
            }),
            state::SurfaceResize::Collapsed => Task::done(Message::SizeChange {
                id,
                size: self.layout.collapsed_surface_size(),
            }),
        }
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
        self.selected_rank =
            state::move_selection(self.selected_rank, self.filtered_indices.len(), offset);
    }

    pub(super) fn update_query(&mut self, query: String) {
        self.selection_revision = self.selection_revision.wrapping_add(1);
        self.query = query;
        self.normalized_query = normalize_query(&self.query);
        self.refresh_filtered_indices();
        self.sync_results_target_with_query();
        self.selected_rank = 0;
        self.highlighted_rank = 0;
        self.scroll_start_rank = 0;
    }

    pub(super) fn set_apps(&mut self, apps: Vec<DesktopApp>) {
        self.selection_revision = self.selection_revision.wrapping_add(1);
        self.apps = apps;
        self.selected_rank = 0;
        self.highlighted_rank = 0;
        self.scroll_start_rank = 0;
        self.refresh_filtered_indices();
    }

    pub(super) fn needs_fast_tick(&self) -> bool {
        self.is_visible && (self.results_target - self.results_progress).abs() > 0.01
    }

    pub(super) fn request_icon_resolution_for_visible(&mut self) -> Task<Message> {
        if !self.is_visible || self.icon_resolve_in_flight {
            return Task::none();
        }

        let requests: Vec<IconResolveRequest> = self
            .filtered_indices
            .iter()
            .copied()
            .filter_map(|index| {
                self.apps
                    .get(index)
                    .filter(|app| app.icon_path.is_none())
                    .map(|app| app.icon_request(index))
            })
            .take(ICON_RESOLVE_BATCH_SIZE)
            .collect();

        if requests.is_empty() {
            return Task::none();
        }

        self.icon_resolve_in_flight = true;
        Task::perform(
            async move { resolve_icon_requests(requests) },
            Message::IconsResolved,
        )
    }

    pub(super) fn apply_resolved_icons(&mut self, updates: Vec<(usize, Option<PathBuf>)>) {
        for (index, icon_path) in updates {
            if let Some(path) = icon_path
                && let Some(app) = self.apps.get_mut(index)
                && app.icon_path.is_none()
            {
                app.icon_path = Some(path);
            }
        }

        self.icon_resolve_in_flight = false;
    }

    pub(super) fn ipc_handle(&self) -> IpcReceiverHandle {
        self.ipc_handle.clone()
    }

    pub(super) fn update_layout(&mut self, monitor_size: Option<Size>) -> Task<Message> {
        self.monitor_size = monitor_size;
        let previous_layout = self.layout.clone();
        self.layout = LauncherLayout::from_monitor_size(
            self.monitor_size,
            &self.layout_preferences,
            &self.app_preferences,
        );

        if !self.is_visible {
            return Task::none();
        }

        if self.layout.should_recreate_surface(&previous_layout) {
            return self.recreate_launcher_surface();
        }

        let Some(id) = self.launcher_window_id else {
            return Task::none();
        };

        let size = if self.results_progress > 0.0 || self.results_target > 0.0 {
            self.layout.expanded_surface_size()
        } else {
            self.layout.collapsed_surface_size()
        };

        if size
            != if self.results_progress > 0.0 || self.results_target > 0.0 {
                previous_layout.expanded_surface_size()
            } else {
                previous_layout.collapsed_surface_size()
            }
        {
            Task::done(Message::SizeChange { id, size })
        } else {
            Task::none()
        }
    }

    pub(super) fn sync_highlighted_rank(&mut self, revision: u64, rank: usize) {
        if revision != self.selection_revision {
            return;
        }

        self.highlighted_rank = if self.filtered_indices.is_empty() {
            0
        } else {
            rank.min(self.filtered_indices.len().saturating_sub(1))
        };
    }

    pub(super) fn reload_preferences_from_disk(&mut self) -> Task<Message> {
        self.app_preferences = load_preferences();
        self.preferences_editor
            .sync_from_preferences(&self.app_preferences);
        self.update_layout(self.monitor_size)
    }

    pub(super) fn resolved_appearance(&self) -> ResolvedAppearance {
        resolve_appearance(&self.app_preferences.appearance)
    }

    pub(super) fn window_theme(&self) -> iced::Theme {
        resolve_theme(&self.app_preferences.appearance)
    }

    pub(super) fn window_theme_for(&self, id: window::Id) -> iced::Theme {
        let _ = id;
        self.window_theme()
    }

    pub(super) fn window_title(&self, id: window::Id) -> Option<String> {
        if self.preferences_window_id == Some(id) {
            Some("Gamut Preferences".to_string())
        } else {
            Some("Gamut".to_string())
        }
    }

    pub(super) fn is_launcher_window(&self, id: window::Id) -> bool {
        self.launcher_window_id == Some(id)
    }

    pub(super) fn is_preferences_window(&self, id: window::Id) -> bool {
        self.preferences_window_id == Some(id)
    }
}
