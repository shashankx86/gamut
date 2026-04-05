mod channel;
mod data;
mod display;
mod runtime;

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
use runtime::progress::{
    ProgressConfig, ProgressContext, ProgressIndicator, ProgressIndicatorMode, ProgressSegments,
};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use channel::{
    AppCommandReceiverHandle, IpcReceiverHandle, SearchResultsReceiverHandle,
    new_app_command_handle, new_ipc_receiver_handle, new_search_results_handle,
};
use display::calculator;

const ICON_RESOLVE_BATCH_SIZE: usize = 24;
const APP_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const APP_REFRESH_PROGRESS_DELAY: Duration = Duration::from_millis(180);
const PROGRESS_CONFIG: ProgressConfig = ProgressConfig::indexing_update_indeterminate();

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
    progress_config: ProgressConfig,
    progress_indicator: ProgressIndicator,
    layout_preferences: LauncherPreferences,
    pub(super) app_preferences: AppPreferences,
    visual_cache: LauncherVisualCache,
    tray_controller: TrayController,
    modifiers: Modifiers,
    suppress_alt_actions_until_release: bool,
    action_overlay_pinned: bool,
    suppress_next_query_change: bool,
    icon_resolve_in_flight: bool,
    app_refresh_in_flight: bool,
    app_refresh_started_at: Option<Instant>,
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
            filtered_indices: Vec::new(),
            results_progress: 0.0,
            results_target: 0.0,
            manually_expanded: false,
            progress_config: PROGRESS_CONFIG,
            progress_indicator: ProgressIndicator::default(),
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

    pub(super) fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    pub(super) fn calculation_preview(&self) -> Option<calculator::CalculationPreview> {
        calculator::calculation_preview(&self.query)
    }

    fn clear_window_state(&mut self) {
        self.bump_selection_revision();
        self.reset_search_state();
        self.ignore_unfocus_until = None;
        self.reset_selection_cursor_state();
        self.results_progress = 0.0;
        self.results_target = 0.0;
        self.manually_expanded = false;
        self.progress_indicator = ProgressIndicator::default();
        self.icon_resolve_in_flight = false;
        self.app_refresh_started_at = None;
        self.suppress_alt_actions_until_release = false;
        self.action_overlay_pinned = false;
        self.suppress_next_query_change = false;
    }

    pub(super) fn results_progress(&self) -> f32 {
        self.results_progress
    }

    pub(super) fn sync_results_target_with_query(&mut self) {
        self.results_target = display::state::results_target(
            self.normalized_query.is_empty(),
            self.manually_expanded,
        );
    }

    pub(super) fn animate_results(&mut self) -> Task<Message> {
        let step = display::state::animate_results(
            self.results_progress,
            self.results_target,
            &self.layout,
        );
        self.results_progress = step.next_progress;

        let Some(id) = self.launcher_window_id else {
            return Task::none();
        };

        match step.surface_resize {
            display::state::SurfaceResize::None => Task::none(),
            display::state::SurfaceResize::Expanded => Task::done(Message::SizeChange {
                id,
                size: self.layout.expanded_surface_size(),
            }),
            display::state::SurfaceResize::Collapsed => Task::done(Message::SizeChange {
                id,
                size: self.layout.collapsed_surface_size(),
            }),
        }
    }

    pub(super) fn selected_result_index(&self) -> Option<usize> {
        if self.calculation_preview().is_some() {
            return None;
        }

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

    pub(super) fn selected_application_index(&self) -> Option<usize> {
        let index = self.selected_result_index()?;
        let app = self.apps.get(index)?;
        (crate::core::preferences::normalize_identifier(&app.entry_type) == "application")
            .then_some(index)
    }

    pub(super) fn is_expanded(&self) -> bool {
        self.results_progress > 0.0 || self.results_target > 0.0
    }

    pub(super) fn should_show_action_overlay(&self) -> bool {
        self.is_expanded()
            && self.selected_application_index().is_some()
            && ((self.modifiers.alt() && !self.suppress_alt_actions_until_release)
                || self.action_overlay_pinned)
    }

    pub(super) fn suppress_alt_actions_until_release(&mut self) {
        self.suppress_alt_actions_until_release = true;
    }

    pub(super) fn sync_alt_action_state_with_modifiers(&mut self) {
        if !self.modifiers.alt() {
            self.suppress_alt_actions_until_release = false;
            self.suppress_next_query_change = false;
        }
    }

    pub(super) fn suppress_next_query_change(&mut self) {
        self.suppress_next_query_change = true;
    }

    pub(super) fn consume_suppressed_query_change(&mut self) -> bool {
        if !self.suppress_next_query_change {
            return false;
        }

        self.suppress_next_query_change = false;
        true
    }

    pub(super) fn move_selection(&mut self, offset: isize) {
        self.selected_rank =
            display::state::move_selection(self.selected_rank, self.filtered_indices.len(), offset);
    }

    pub(super) fn needs_fast_tick(&self) -> bool {
        let mode = self.progress_indicator_mode();
        self.is_visible
            && ((self.results_target - self.results_progress).abs() > f32::EPSILON
                || (self.should_render_progress_line()
                    && self
                        .progress_indicator
                        .needs_animation(mode, self.progress_config.animation())))
    }

    fn progress_indicator_mode(&self) -> ProgressIndicatorMode {
        self.progress_config.mode(self.progress_context())
    }

    fn progress_context(&self) -> ProgressContext {
        ProgressContext {
            manual_expanded: self.manually_expanded,
            expanding: self.manually_expanded
                && self.results_progress < self.results_target
                && self.results_target > 0.0,
            collapsing: self.results_progress > self.results_target,
            search_in_flight: self.search_in_flight,
            app_refresh_in_flight: self.indexing_progress_ready(),
            icon_resolve_in_flight: self.icon_resolve_in_flight,
        }
    }

    fn indexing_progress_ready(&self) -> bool {
        self.app_refresh_in_flight
            && self
                .app_refresh_started_at
                .is_some_and(|started_at| started_at.elapsed() >= APP_REFRESH_PROGRESS_DELAY)
    }

    pub(super) fn should_render_progress_line(&self) -> bool {
        self.progress_config.is_enabled()
    }

    fn progress_segments(&self, width: f32) -> ProgressSegments {
        self.progress_indicator.segments(
            self.progress_indicator_mode(),
            width,
            self.progress_segment_width(width),
            self.progress_config.animation().finish_current_sweep,
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
        if matches!(
            self.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        ) {
            return (0.0, 0.0, width);
        }

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
            0,
        );
    }

    pub(super) fn request_icon_resolution_for_visible(&mut self) -> Task<Message> {
        if !self.is_visible || self.icon_resolve_in_flight || self.calculation_preview().is_some() {
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
        self.app_refresh_started_at = Some(Instant::now());
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
        self.app_refresh_started_at = None;
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
    use super::runtime::progress::ProgressIndicatorMode;
    use super::*;
    use crate::core::app_command::AppCommand;
    use std::sync::mpsc;

    fn launcher() -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (launcher, _) = Launcher::new(rx, crate::core::tray::TrayController::detached());
        launcher
    }

    #[test]
    fn progress_mode_stays_hidden_for_search_when_indexing_profile_is_used() {
        let mut launcher = launcher();
        launcher.search_in_flight = true;

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }

    #[test]
    fn progress_mode_is_indeterminate_during_indexing_updates() {
        let mut launcher = launcher();
        launcher.app_refresh_in_flight = true;
        launcher.app_refresh_started_at = Some(Instant::now() - APP_REFRESH_PROGRESS_DELAY);

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

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }

    #[test]
    fn progress_mode_hides_when_indexing_finishes() {
        let mut launcher = launcher();
        launcher.app_refresh_in_flight = false;

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }

    #[test]
    fn progress_mode_hides_when_idle() {
        let launcher = launcher();

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }

    #[test]
    fn manual_expand_does_not_force_progress_indicator() {
        let mut launcher = launcher();

        let _ = launcher.expand_results();

        assert!(launcher.manually_expanded);
        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }

    #[test]
    fn progress_config_defaults_to_indexing_update_mode() {
        let launcher = launcher();

        assert!(launcher.should_render_progress_line());

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }

    #[test]
    fn progress_line_renders_as_divider_even_when_collapsed() {
        let mut launcher = launcher();

        assert!(launcher.should_render_progress_line());
        assert_eq!(launcher.progress_line_widths(120.0), (0.0, 0.0, 120.0));

        launcher.app_refresh_in_flight = true;
        launcher.app_refresh_started_at = Some(Instant::now() - APP_REFRESH_PROGRESS_DELAY);
        launcher.results_target = 1.0;
        assert!(launcher.should_render_progress_line());
        let (_leading, active, _trailing) = launcher.progress_line_widths(120.0);
        assert!(active > 0.0);

        launcher.app_refresh_in_flight = false;
        launcher.app_refresh_started_at = None;
        assert!(launcher.should_render_progress_line());
        assert_eq!(launcher.progress_line_widths(120.0), (0.0, 0.0, 120.0));
    }

    #[test]
    fn progress_mode_stays_hidden_for_short_indexing_bursts() {
        let mut launcher = launcher();
        launcher.app_refresh_in_flight = true;
        launcher.app_refresh_started_at = Some(Instant::now());

        assert_eq!(
            launcher.progress_indicator_mode(),
            ProgressIndicatorMode::Hidden
        );
    }
}
