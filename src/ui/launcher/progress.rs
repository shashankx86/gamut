use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProgressIndicatorMode {
    Hidden,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ProgressContext {
    pub(super) expanding: bool,
    pub(super) collapsing: bool,
    pub(super) search_in_flight: bool,
    pub(super) app_refresh_in_flight: bool,
    pub(super) icon_resolve_in_flight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ProgressPolicy {
    pub(super) enabled: bool,
    pub(super) during_expand: bool,
    pub(super) during_collapse: bool,
    pub(super) during_search: bool,
    pub(super) during_app_refresh: bool,
    pub(super) during_icon_resolve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ProgressAnimationConfig {
    pub(super) cycles_per_second: f32,
    pub(super) rows_per_segment: f32,
    pub(super) min_segment_px: f32,
    pub(super) max_segment_ratio: f32,
    pub(super) finish_current_sweep: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ProgressConfig {
    policy: ProgressPolicy,
    animation: ProgressAnimationConfig,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

impl ProgressConfig {
    pub(super) const fn disabled() -> Self {
        Self {
            policy: ProgressPolicy {
                enabled: false,
                during_expand: false,
                during_collapse: false,
                during_search: false,
                during_app_refresh: false,
                during_icon_resolve: false,
            },
            animation: ProgressAnimationConfig {
                cycles_per_second: 0.2,
                rows_per_segment: 2.0,
                min_segment_px: 24.0,
                max_segment_ratio: 0.6,
                finish_current_sweep: true,
            },
        }
    }

    pub(super) const fn expansion_indeterminate_slow() -> Self {
        Self {
            policy: ProgressPolicy {
                enabled: true,
                during_expand: true,
                during_collapse: false,
                during_search: false,
                during_app_refresh: false,
                during_icon_resolve: false,
            },
            animation: ProgressAnimationConfig {
                cycles_per_second: 0.2,
                rows_per_segment: 2.0,
                min_segment_px: 24.0,
                max_segment_ratio: 0.6,
                finish_current_sweep: true,
            },
        }
    }

    pub(super) fn mode(self, context: ProgressContext) -> ProgressIndicatorMode {
        if !self.policy.enabled {
            return ProgressIndicatorMode::Hidden;
        }

        if (context.expanding && self.policy.during_expand)
            || (context.collapsing && self.policy.during_collapse)
            || (context.search_in_flight && self.policy.during_search)
            || (context.app_refresh_in_flight && self.policy.during_app_refresh)
            || (context.icon_resolve_in_flight && self.policy.during_icon_resolve)
        {
            ProgressIndicatorMode::Indeterminate
        } else {
            ProgressIndicatorMode::Hidden
        }
    }

    pub(super) fn is_enabled(self) -> bool {
        self.policy.enabled
    }

    pub(super) fn animation(self) -> ProgressAnimationConfig {
        self.animation
    }

    pub(super) fn segment_width(
        self,
        track_width: f32,
        row_height: f32,
        row_step: f32,
        viewport_height: f32,
    ) -> f32 {
        let track_width = track_width.max(0.0);
        if track_width <= 0.0 {
            return 0.0;
        }

        let visible_rows = if row_step > 0.0 {
            (viewport_height / row_step).max(1.0)
        } else {
            1.0
        };
        let rows_for_segment = self.animation.rows_per_segment.clamp(1.0, visible_rows);
        let preferred = (rows_for_segment * row_height).max(self.animation.min_segment_px);
        let max_width = (track_width * self.animation.max_segment_ratio).max(1.0);

        preferred.min(max_width).min(track_width)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ProgressSegments {
    pub(super) leading_track: f32,
    pub(super) active: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimationState {
    Idle,
    Running,
    Finishing,
}

#[derive(Debug, Clone)]
pub(super) struct ProgressIndicator {
    phase: f32,
    state: AnimationState,
    last_tick_at: Option<Instant>,
}

impl Default for ProgressIndicator {
    fn default() -> Self {
        Self {
            phase: 0.0,
            state: AnimationState::Idle,
            last_tick_at: None,
        }
    }
}

impl ProgressIndicator {
    pub(super) fn tick(&mut self, mode: ProgressIndicatorMode, animation: ProgressAnimationConfig) {
        let now = Instant::now();
        let elapsed_seconds = self
            .last_tick_at
            .map(|previous| now.saturating_duration_since(previous).as_secs_f32())
            .unwrap_or(0.0);
        self.last_tick_at = Some(now);

        let phase_delta = (elapsed_seconds * animation.cycles_per_second.max(0.0)).max(0.0);
        self.advance(mode, phase_delta, animation.finish_current_sweep);
    }

    pub(super) fn needs_animation(
        &self,
        mode: ProgressIndicatorMode,
        animation: ProgressAnimationConfig,
    ) -> bool {
        matches!(mode, ProgressIndicatorMode::Indeterminate)
            || (animation.finish_current_sweep && matches!(self.state, AnimationState::Finishing))
    }

    pub(super) fn segments(
        &self,
        mode: ProgressIndicatorMode,
        track_width: f32,
        segment_width: f32,
    ) -> ProgressSegments {
        if mode == ProgressIndicatorMode::Hidden && self.state == AnimationState::Idle {
            return ProgressSegments {
                leading_track: 0.0,
                active: 0.0,
            };
        }

        let track_width = track_width.max(0.0);
        let segment_width = segment_width.max(0.0).min(track_width);

        if track_width <= 0.0 || segment_width <= 0.0 {
            return ProgressSegments {
                leading_track: 0.0,
                active: 0.0,
            };
        }

        let max_start = (track_width - segment_width).max(0.0);
        let leading_track = self.phase.clamp(0.0, 1.0) * max_start;

        ProgressSegments {
            leading_track,
            active: segment_width,
        }
    }

    fn advance(
        &mut self,
        mode: ProgressIndicatorMode,
        phase_delta: f32,
        finish_current_sweep: bool,
    ) {
        match mode {
            ProgressIndicatorMode::Indeterminate => {
                if self.state != AnimationState::Running {
                    self.state = AnimationState::Running;
                    if self.phase >= 1.0 {
                        self.phase = 0.0;
                    }
                }

                self.phase = (self.phase + phase_delta).fract();
            }
            ProgressIndicatorMode::Hidden => {
                if self.state == AnimationState::Running {
                    if finish_current_sweep {
                        self.state = AnimationState::Finishing;
                    } else {
                        self.state = AnimationState::Idle;
                        self.phase = 0.0;
                    }
                }

                if self.state != AnimationState::Finishing {
                    self.last_tick_at = None;
                    return;
                }

                if self.phase >= 1.0 {
                    self.phase = 0.0;
                    self.state = AnimationState::Idle;
                    self.last_tick_at = None;
                    return;
                }

                self.phase = (self.phase + phase_delta).min(1.0);
            }
        }
    }

    #[cfg(test)]
    fn tick_by(
        &mut self,
        mode: ProgressIndicatorMode,
        phase_delta: f32,
        finish_current_sweep: bool,
    ) {
        self.advance(mode, phase_delta.max(0.0), finish_current_sweep);
    }
}

#[cfg(test)]
mod tests {
    use super::{ProgressConfig, ProgressContext, ProgressIndicator, ProgressIndicatorMode};

    #[test]
    fn indeterminate_segment_stays_within_track_bounds() {
        let mut indicator = ProgressIndicator::default();

        for _ in 0..40 {
            indicator.tick_by(ProgressIndicatorMode::Indeterminate, 0.05, true);
            let segments = indicator.segments(ProgressIndicatorMode::Indeterminate, 120.0, 36.0);
            assert!(segments.leading_track >= 0.0);
            assert!(segments.active >= 0.0);
            assert!(segments.leading_track + segments.active <= 120.0);
        }
    }

    #[test]
    fn indeterminate_segment_keeps_consistent_width() {
        let mut indicator = ProgressIndicator::default();

        for _ in 0..20 {
            indicator.tick_by(ProgressIndicatorMode::Indeterminate, 0.03, true);
            let segments = indicator.segments(ProgressIndicatorMode::Indeterminate, 120.0, 42.0);
            assert!((segments.active - 42.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn hidden_state_finishes_sweep_before_disappearing() {
        let mut indicator = ProgressIndicator::default();
        indicator.tick_by(ProgressIndicatorMode::Indeterminate, 0.85, true);

        indicator.tick_by(ProgressIndicatorMode::Hidden, 0.10, true);
        let near_end = indicator.segments(ProgressIndicatorMode::Hidden, 100.0, 30.0);
        assert!(near_end.leading_track > 60.0);

        indicator.tick_by(ProgressIndicatorMode::Hidden, 0.20, true);
        let at_end = indicator.segments(ProgressIndicatorMode::Hidden, 100.0, 30.0);
        assert!((at_end.leading_track - 70.0).abs() < f32::EPSILON);

        indicator.tick_by(ProgressIndicatorMode::Hidden, 0.0, true);
        let hidden = indicator.segments(ProgressIndicatorMode::Hidden, 100.0, 30.0);
        assert_eq!(hidden.active, 0.0);
    }

    #[test]
    fn disabled_config_always_hides_progress() {
        let config = ProgressConfig::disabled();
        let context = ProgressContext {
            expanding: true,
            collapsing: false,
            search_in_flight: true,
            app_refresh_in_flight: true,
            icon_resolve_in_flight: true,
        };

        assert_eq!(config.mode(context), ProgressIndicatorMode::Hidden);
    }

    #[test]
    fn expansion_profile_activates_only_while_expanding() {
        let config = ProgressConfig::expansion_indeterminate_slow();

        assert_eq!(
            config.mode(ProgressContext {
                expanding: true,
                collapsing: false,
                search_in_flight: false,
                app_refresh_in_flight: false,
                icon_resolve_in_flight: false,
            }),
            ProgressIndicatorMode::Indeterminate
        );

        assert_eq!(
            config.mode(ProgressContext {
                expanding: false,
                collapsing: true,
                search_in_flight: false,
                app_refresh_in_flight: false,
                icon_resolve_in_flight: false,
            }),
            ProgressIndicatorMode::Hidden
        );
    }
}
