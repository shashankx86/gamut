use super::super::runtime::progress::{ProgressContext, ProgressIndicatorMode, ProgressSegments};
use super::Launcher;

impl Launcher {
    pub(in crate::ui::launcher) fn needs_fast_tick(&self) -> bool {
        let mode = self.progress_indicator_mode();
        self.is_visible
            && ((self.results_target - self.results_progress).abs() > f32::EPSILON
                || (self.should_render_progress_line()
                    && self
                        .progress_indicator
                        .needs_animation(mode, self.progress_config.animation())))
    }

    pub(super) fn progress_indicator_mode(&self) -> ProgressIndicatorMode {
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
                .is_some_and(|started_at| started_at.elapsed() >= super::APP_REFRESH_PROGRESS_DELAY)
    }

    pub(in crate::ui) fn should_render_progress_line(&self) -> bool {
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

    pub(in crate::ui) fn progress_line_widths(&self, width: f32) -> (f32, f32, f32) {
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

    pub(in crate::ui::launcher) fn tick_progress_indicator(&mut self) {
        self.progress_indicator.tick(
            self.progress_indicator_mode(),
            self.progress_config.animation(),
            0,
        );
    }
}
