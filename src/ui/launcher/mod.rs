mod actions;
mod input;
mod progress;
mod receiver;
mod search;
mod state;
mod subscription;
mod update;

use super::constants::MAX_RESULTS;
use super::layout::{LauncherLayout, LauncherPreferences};
use super::theme::{ResolvedAppearance, resolve_appearance, resolve_asset_theme, resolve_theme};
use crate::core::app_command::AppCommand;
use crate::core::assets::launcher_logo_svg;
use crate::core::desktop::{
    DesktopApp, IconResolveRequest, load_cached_app_catalog, refresh_app_cache,
    resolve_icon_requests, save_cached_apps,
};
use crate::core::ipc::{IpcCommand, start_listener};
use crate::core::preferences::{AppPreferences, load_preferences};
use crate::core::search::{ApplicationSearchEngine, ApplicationSearchResponse};
use crate::core::tray::TrayController;
use iced::Size;
use iced::keyboard::Modifiers;
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{self, scrollable};
use iced::{Task, window};
use iced_layershell::to_layer_message;
use log::warn;
use progress::{
    ProgressConfig, ProgressContext, ProgressIndicator, ProgressIndicatorMode, ProgressSegments,
};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub(in crate::ui) use state::{render_range_for_viewport, spacer_height_for_rows};

const ICON_RESOLVE_BATCH_SIZE: usize = 24;
const IPC_SUBSCRIPTION_ID: u64 = 1;
const APP_SEARCH_SUBSCRIPTION_ID: u64 = IPC_SUBSCRIPTION_ID + 2;
const APP_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const PROGRESS_CONFIG: ProgressConfig = ProgressConfig::manual_expand_indeterminate();

#[derive(Clone)]
pub(super) struct IpcReceiverHandle {
    id: u64,
    receiver: Arc<Mutex<Receiver<IpcCommand>>>,
}

#[derive(Clone)]
pub(super) struct AppCommandReceiverHandle {
    id: u64,
    receiver: Arc<Mutex<Receiver<AppCommand>>>,
}

impl AppCommandReceiverHandle {
    fn new(receiver: Receiver<AppCommand>) -> Self {
        Self {
            id: IPC_SUBSCRIPTION_ID + 1,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[derive(Clone)]
pub(super) struct SearchResultsReceiverHandle {
    id: u64,
    receiver: Arc<Mutex<Receiver<ApplicationSearchResponse>>>,
}

impl SearchResultsReceiverHandle {
    fn new(receiver: Receiver<ApplicationSearchResponse>) -> Self {
        Self {
            id: APP_SEARCH_SUBSCRIPTION_ID,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

impl Hash for SearchResultsReceiverHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for AppCommandReceiverHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
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
    pub(super) all_app_indices: Vec<usize>,
    pub(super) layout: LauncherLayout,
    pub(super) query: String,
    pub(super) normalized_query: String,
    pub(super) input_id: widget::Id,
    pub(super) results_scroll_id: widget::Id,
    pub(super) launcher_window_id: Option<window::Id>,
    pub(super) monitor_size: Option<Size>,
    pub(super) target_output: Option<String>,
    pub(super) is_visible: bool,
    pub(super) ignore_unfocus_until: Option<std::time::Instant>,
    pub(super) selected_rank: usize,
    pub(super) highlighted_rank: usize,
    selection_revision: u64,
    search_generation: u64,
    applied_search_generation: u64,
    pub(super) search_in_flight: bool,
    pub(super) results_scroll_offset: f32,
    pub(super) scroll_start_rank: usize,
    pub(super) filtered_indices: Vec<usize>,
    pub(super) results_progress: f32,
    pub(super) results_target: f32,
    pub(super) manually_expanded: bool,
    manual_expand_sequence: u64,
    progress_config: ProgressConfig,
    progress_indicator: ProgressIndicator,
    layout_preferences: LauncherPreferences,
    pub(super) app_preferences: AppPreferences,
    visual_cache: LauncherVisualCache,
    tray_controller: TrayController,
    modifiers: Modifiers,
    icon_resolve_in_flight: bool,
    app_refresh_in_flight: bool,
    last_app_refresh_at: Option<Instant>,
    app_search_engine: ApplicationSearchEngine,
    search_results_handle: SearchResultsReceiverHandle,
    ipc_handle: IpcReceiverHandle,
    app_command_handle: AppCommandReceiverHandle,
}

#[derive(Clone)]
struct LauncherVisualCache {
    appearance: ResolvedAppearance,
    window_theme: iced::Theme,
    logo_handle: SvgHandle,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub(super) enum Message {
    Tick,
    AppsLoaded(Vec<DesktopApp>),
    SearchResultsLoaded(ApplicationSearchResponse),
    IconsResolved(Vec<(usize, Option<PathBuf>)>),
    QueryChanged(String),
    LaunchFirstMatch,
    ExpandResults,
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

impl Launcher {
    #[cfg(test)]
    pub(super) fn app_refresh_in_flight(&self) -> bool {
        self.app_refresh_in_flight
    }

    pub(super) fn new(
        command_receiver: Receiver<AppCommand>,
        tray_controller: TrayController,
    ) -> (Self, Task<Message>) {
        let layout_preferences = LauncherPreferences::load_from_env();
        let app_preferences = load_preferences();
        let layout = LauncherLayout::from_monitor_size(None, &layout_preferences, &app_preferences);
        let visual_cache = Self::build_visual_cache(&app_preferences);
        let input_id = widget::Id::unique();
        let results_scroll_id = widget::Id::unique();
        let cached_catalog = load_cached_app_catalog();
        let (app_search_engine, search_results_receiver) =
            ApplicationSearchEngine::spawn(MAX_RESULTS);
        let (ipc_handle, startup_task) = match start_listener() {
            Ok(receiver) => (
                IpcReceiverHandle::new(receiver),
                if cached_catalog.needs_refresh {
                    Task::perform(async { refresh_app_cache() }, Message::AppsLoaded)
                } else {
                    Task::none()
                },
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
            filtered_indices: Vec::new(),
            results_progress: 0.0,
            results_target: 0.0,
            manually_expanded: false,
            manual_expand_sequence: 0,
            progress_config: PROGRESS_CONFIG,
            progress_indicator: ProgressIndicator::default(),
            layout_preferences,
            app_preferences,
            visual_cache,
            tray_controller,
            modifiers: Modifiers::default(),
            icon_resolve_in_flight: false,
            app_refresh_in_flight: false,
            last_app_refresh_at: None,
            app_search_engine,
            search_results_handle: SearchResultsReceiverHandle::new(search_results_receiver),
            ipc_handle,
            app_command_handle: AppCommandReceiverHandle::new(command_receiver),
        };

        if !cached_catalog.apps.is_empty() {
            launcher.set_apps(cached_catalog.apps);
        }

        (launcher, startup_task)
    }

    pub(super) fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    fn clear_window_state(&mut self) {
        self.selection_revision = self.selection_revision.wrapping_add(1);
        self.reset_search_state();
        self.ignore_unfocus_until = None;
        self.selected_rank = 0;
        self.highlighted_rank = 0;
        self.results_scroll_offset = 0.0;
        self.scroll_start_rank = 0;
        self.results_progress = 0.0;
        self.results_target = 0.0;
        self.manually_expanded = false;
        self.manual_expand_sequence = 0;
        self.progress_indicator = ProgressIndicator::default();
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
        if !self.normalized_query.is_empty()
            && self.applied_search_generation != self.search_generation
        {
            return None;
        }

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

    pub(super) fn needs_fast_tick(&self) -> bool {
        let mode = self.progress_indicator_mode();
        self.is_visible
            && ((self.results_target - self.results_progress).abs() > f32::EPSILON
                || self.progress_indicator.needs_animation(
                    mode,
                    self.progress_config.animation(),
                ))
    }

    fn progress_indicator_mode(&self) -> ProgressIndicatorMode {
        let mode = self.progress_config.mode(self.progress_context());
        if matches!(mode, ProgressIndicatorMode::Indeterminate)
            && self
                .progress_indicator
                .completed_for(self.manual_expand_sequence)
        {
            ProgressIndicatorMode::Hidden
        } else {
            mode
        }
    }

    fn progress_context(&self) -> ProgressContext {
        ProgressContext {
            manual_expanded: self.manually_expanded,
            expanding: self.manually_expanded
                && self.results_progress < self.results_target
                && self.results_target > 0.0,
            collapsing: self.results_progress > self.results_target,
            search_in_flight: self.search_in_flight,
            app_refresh_in_flight: self.app_refresh_in_flight,
            icon_resolve_in_flight: self.icon_resolve_in_flight,
        }
    }

    pub(super) fn should_render_progress_line(&self) -> bool {
        self.progress_config.is_enabled()
    }

    fn progress_segments(&self, width: f32) -> ProgressSegments {
        self.progress_indicator.segments(
            self.progress_indicator_mode(),
            width,
            self.progress_segment_width(width),
        )
    }

    fn progress_segment_width(&self, width: f32) -> f32 {
        self.progress_config.segment_width(
            width,
            self.layout.result_row_height,
            self.layout.result_row_scroll_step(),
            self.layout.results_viewport_height(),
        )
    }

    pub(super) fn progress_line_widths(&self, width: f32) -> (f32, f32, f32) {
        let width = width.max(0.0);
        let segments = self.progress_segments(width);
        let leading_track = segments.leading_track.clamp(0.0, width);
        let active = segments.active.clamp(0.0, (width - leading_track).max(0.0));
        let trailing_track = (width - leading_track - active).max(0.0);

        (leading_track, active, trailing_track)
    }

    pub(super) fn tick_progress_indicator(&mut self) {
        self.progress_indicator.tick(
            self.progress_indicator_mode(),
            self.progress_config.animation(),
            self.manual_expand_sequence,
        );
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

    pub(super) fn request_app_refresh(&mut self, force: bool) -> Task<Message> {
        if self.app_refresh_in_flight {
            return Task::none();
        }

        if !force
            && self
                .last_app_refresh_at
                .is_some_and(|at| at.elapsed() < APP_REFRESH_INTERVAL)
        {
            return Task::none();
        }

        self.app_refresh_in_flight = true;
        Task::perform(async { refresh_app_cache() }, Message::AppsLoaded)
    }

    pub(super) fn apply_resolved_icons(&mut self, updates: Vec<(usize, Option<PathBuf>)>) {
        let mut changed = false;

        for (index, icon_path) in updates {
            if let Some(path) = icon_path
                && let Some(app) = self.apps.get_mut(index)
                && app.icon_path.is_none()
            {
                app.icon_path = Some(path);
                changed = true;
            }
        }

        if changed && let Err(error) = save_cached_apps(&self.apps) {
            warn!("failed to persist resolved icon paths: {error}");
        }

        self.icon_resolve_in_flight = false;
    }

    pub(super) fn finish_app_refresh(&mut self) {
        self.app_refresh_in_flight = false;
        self.last_app_refresh_at = Some(Instant::now());
    }

    pub(super) fn ipc_handle(&self) -> IpcReceiverHandle {
        self.ipc_handle.clone()
    }

    pub(super) fn app_command_handle(&self) -> AppCommandReceiverHandle {
        self.app_command_handle.clone()
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
        self.refresh_visual_cache();
        self.tray_controller
            .update_preferences(self.app_preferences.clone());
        self.update_layout(self.monitor_size)
    }

    pub(super) fn resolved_appearance(&self) -> ResolvedAppearance {
        self.visual_cache.appearance
    }

    pub(super) fn window_theme(&self) -> iced::Theme {
        self.visual_cache.window_theme.clone()
    }

    pub(super) fn launcher_logo_handle(&self) -> SvgHandle {
        self.visual_cache.logo_handle.clone()
    }

    pub(super) fn window_theme_for(&self, id: window::Id) -> iced::Theme {
        let _ = id;
        self.window_theme()
    }

    pub(super) fn window_title(&self, id: window::Id) -> Option<String> {
        let _ = id;
        Some("Gamut".to_string())
    }

    pub(super) fn is_launcher_window(&self, id: window::Id) -> bool {
        self.launcher_window_id == Some(id)
    }

    fn refresh_visual_cache(&mut self) {
        self.visual_cache = Self::build_visual_cache(&self.app_preferences);
    }

    fn build_visual_cache(app_preferences: &AppPreferences) -> LauncherVisualCache {
        let appearance = resolve_appearance(&app_preferences.appearance);
        let window_theme = resolve_theme(&app_preferences.appearance);
        let logo_handle = SvgHandle::from_memory(launcher_logo_svg(resolve_asset_theme(
            &app_preferences.appearance,
        )));

        LauncherVisualCache {
            appearance,
            window_theme,
            logo_handle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::progress::ProgressIndicatorMode;
    use super::*;
    use crate::core::app_command::AppCommand;
    use std::sync::mpsc;

    fn launcher() -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (launcher, _) = Launcher::new(rx, crate::core::tray::TrayController::detached());
        launcher
    }

    #[test]
    fn progress_mode_stays_hidden_for_search_when_expansion_only_profile_is_used() {
        let mut launcher = launcher();
        launcher.search_in_flight = true;

        assert_eq!(launcher.progress_indicator_mode(), ProgressIndicatorMode::Hidden);
    }

    #[test]
    fn progress_mode_is_indeterminate_during_results_animation() {
        let mut launcher = launcher();
        launcher.results_target = 1.0;
        launcher.results_progress = 1.0;
        launcher.manually_expanded = true;
        launcher.manual_expand_sequence = 2;

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Indeterminate
        );
    }

    #[test]
    fn progress_mode_stays_hidden_for_query_driven_expansion() {
        let mut launcher = launcher();
        launcher.results_target = 1.0;
        launcher.results_progress = 0.45;
        launcher.manually_expanded = false;

        assert_eq!(launcher.progress_indicator_mode(), ProgressIndicatorMode::Hidden);
    }

    #[test]
    fn progress_mode_hides_after_completing_current_manual_expand_sequence() {
        let mut launcher = launcher();
        launcher.manually_expanded = true;
        launcher.manual_expand_sequence = 3;
        launcher
            .progress_indicator
            .mark_completed_for_testing(launcher.manual_expand_sequence);

        assert_eq!(launcher.progress_indicator_mode(), ProgressIndicatorMode::Hidden);
    }

    #[test]
    fn progress_mode_hides_when_idle() {
        let launcher = launcher();

        assert_eq!(launcher.progress_indicator_mode(), ProgressIndicatorMode::Hidden);
    }

    #[test]
    fn manual_expand_command_increments_progress_sequence() {
        let mut launcher = launcher();
        let initial_sequence = launcher.manual_expand_sequence;

        let _ = launcher.expand_results();

        assert_eq!(launcher.manual_expand_sequence, initial_sequence.wrapping_add(1));
        assert!(launcher.manually_expanded);
    }

    #[test]
    fn progress_config_defaults_to_expansion_only_mode() {
        let launcher = launcher();

        assert!(launcher.should_render_progress_line());

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }
}
