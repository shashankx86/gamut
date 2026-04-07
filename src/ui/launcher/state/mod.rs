mod accessors;
mod layout;
mod lifecycle;
mod progress;
mod refresh;

use super::channel::{AppCommandReceiverHandle, IpcReceiverHandle, SearchResultsReceiverHandle};
use super::display::calculator;
use crate::core::assets::launcher_logo_svg;
use crate::core::desktop::DesktopApp;
use crate::core::preferences::AppPreferences;
use crate::core::search::ApplicationSearchEngine;
use crate::core::tray::TrayController;
use crate::ui::layout::{LauncherLayout, LauncherPreferences};
use crate::ui::theme::{
    ResolvedAppearance, resolve_appearance, resolve_asset_theme, resolve_theme,
};
use iced::keyboard::Modifiers;
use iced::widget;
use iced::widget::svg::Handle as SvgHandle;
use iced::{Size, window};
use std::time::{Duration, Instant};

use super::runtime::progress::{ProgressConfig, ProgressIndicator};

pub(super) const APP_REFRESH_PROGRESS_DELAY: Duration = Duration::from_millis(180);

pub(in crate::ui) struct Launcher {
    pub(in crate::ui) apps: Vec<DesktopApp>,
    pub(in crate::ui) all_app_indices: Vec<usize>,
    pub(in crate::ui) layout: LauncherLayout,
    pub(in crate::ui) query: String,
    pub(in crate::ui) normalized_query: String,
    pub(in crate::ui) input_id: widget::Id,
    pub(in crate::ui) results_scroll_id: widget::Id,
    pub(in crate::ui) launcher_window_id: Option<window::Id>,
    pub(in crate::ui) monitor_size: Option<Size>,
    pub(in crate::ui) target_output: Option<String>,
    pub(in crate::ui) is_visible: bool,
    pub(in crate::ui) ignore_unfocus_until: Option<std::time::Instant>,
    pub(in crate::ui) selected_rank: usize,
    pub(in crate::ui) highlighted_rank: usize,
    pub(in crate::ui) selection_revision: u64,
    pub(in crate::ui) search_generation: u64,
    pub(in crate::ui) applied_search_generation: u64,
    pub(in crate::ui) search_in_flight: bool,
    pub(in crate::ui) results_scroll_offset: f32,
    pub(in crate::ui) scroll_start_rank: usize,
    pub(in crate::ui::launcher) results_scrollbar_visibility:
        super::scroll::ResultsScrollbarVisibility,
    pub(in crate::ui) filtered_indices: Vec<usize>,
    pub(in crate::ui) results_progress: f32,
    pub(in crate::ui) results_target: f32,
    pub(in crate::ui) manually_expanded: bool,
    pub(in crate::ui::launcher) progress_config: ProgressConfig,
    pub(in crate::ui::launcher) progress_indicator: ProgressIndicator,
    pub(in crate::ui) layout_preferences: LauncherPreferences,
    pub(in crate::ui) app_preferences: AppPreferences,
    pub(in crate::ui::launcher) visual_cache: LauncherVisualCache,
    pub(in crate::ui) tray_controller: TrayController,
    pub(in crate::ui) modifiers: Modifiers,
    pub(in crate::ui) suppress_alt_actions_until_release: bool,
    pub(in crate::ui) action_overlay_pinned: bool,
    pub(in crate::ui) suppress_next_query_change: bool,
    pub(in crate::ui) icon_resolve_in_flight: bool,
    pub(in crate::ui) app_refresh_in_flight: bool,
    pub(in crate::ui) app_refresh_started_at: Option<Instant>,
    pub(in crate::ui) last_app_refresh_at: Option<Instant>,
    pub(in crate::ui) app_search_engine: ApplicationSearchEngine,
    pub(in crate::ui) search_results_handle: SearchResultsReceiverHandle,
    pub(in crate::ui) ipc_handle: IpcReceiverHandle,
    pub(in crate::ui) app_command_handle: AppCommandReceiverHandle,
}

#[derive(Clone)]
pub(super) struct LauncherVisualCache {
    pub(super) appearance: ResolvedAppearance,
    pub(super) window_theme: iced::Theme,
    pub(super) logo_handle: SvgHandle,
}

impl LauncherVisualCache {
    pub(super) fn build(app_preferences: &AppPreferences) -> Self {
        let appearance = resolve_appearance(&app_preferences.appearance);
        let window_theme = resolve_theme(&app_preferences.appearance);
        let logo_handle = SvgHandle::from_memory(launcher_logo_svg(resolve_asset_theme(
            &app_preferences.appearance,
        )));

        Self {
            appearance,
            window_theme,
            logo_handle,
        }
    }
}

impl Launcher {
    #[cfg(test)]
    pub(super) fn app_refresh_in_flight(&self) -> bool {
        self.app_refresh_in_flight
    }

    pub(in crate::ui) fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    pub(in crate::ui) fn calculation_preview(&self) -> Option<calculator::CalculationPreview> {
        calculator::calculation_preview(&self.query)
    }
}

#[cfg(test)]
mod tests {
    use super::super::runtime::progress::ProgressIndicatorMode;
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
