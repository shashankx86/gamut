use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProgressIndicatorMode {
    Hidden,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ProgressContext {
    pub(super) manual_expanded: bool,
    pub(super) expanding: bool,
    pub(super) collapsing: bool,
    pub(super) search_in_flight: bool,
    pub(super) app_refresh_in_flight: bool,
    pub(super) icon_resolve_in_flight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ProgressPolicy {
    pub(super) enabled: bool,
    pub(super) during_manual_expanded: bool,
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
                during_manual_expanded: false,
                during_expand: false,
                during_collapse: false,
                during_search: false,
                during_app_refresh: false,
                during_icon_resolve: false,
            },
            animation: ProgressAnimationConfig {
                cycles_per_second: 1.75,
                rows_per_segment: 1.5,
                min_segment_px: 18.0,
                max_segment_ratio: 0.4,
                finish_current_sweep: false,
            },
        }
    }

    pub(super) const fn manual_expand_indeterminate() -> Self {
        Self {
            policy: ProgressPolicy {
                enabled: true,
                during_manual_expanded: true,
                during_expand: false,
                during_collapse: false,
                during_search: false,
                during_app_refresh: false,
                during_icon_resolve: false,
            },
            animation: ProgressAnimationConfig {
                cycles_per_second: 1.75,
                rows_per_segment: 1.5,
                min_segment_px: 18.0,
                max_segment_ratio: 0.4,
                finish_current_sweep: false,
            },
        }
    }

    pub(super) fn mode(self, context: ProgressContext) -> ProgressIndicatorMode {
        if !self.policy.enabled {
            return ProgressIndicatorMode::Hidden;
        }

        if (context.manual_expanded && self.policy.during_manual_expanded)
            || (context.expanding && self.policy.during_expand)
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

#[derive(Debug, Clone)]
pub(super) struct ProgressIndicator {
    phase: f32,
    completed_cycle: bool,
    active_sequence: u64,
    last_completed_sequence: Option<u64>,
    last_tick_at: Option<Instant>,
    was_active: bool,
}

impl Default for ProgressIndicator {
    fn default() -> Self {
        Self {
            phase: 0.0,
            completed_cycle: false,
            active_sequence: 0,
            last_completed_sequence: None,
            last_tick_at: None,
            was_active: false,
        }
    }
}

impl ProgressIndicator {
    pub(super) fn completed_for(&self, activation_sequence: u64) -> bool {
        self.last_completed_sequence == Some(activation_sequence)
    }

    pub(super) fn tick(
        &mut self,
        mode: ProgressIndicatorMode,
        animation: ProgressAnimationConfig,
        activation_sequence: u64,
    ) {
        let now = Instant::now();
        let elapsed_seconds = self
            .last_tick_at
            .map(|previous| now.saturating_duration_since(previous).as_secs_f32())
            .unwrap_or(0.0);
        self.last_tick_at = Some(now);

        let phase_delta = (elapsed_seconds * animation.cycles_per_second.max(0.0)).max(0.0);

        match mode {
            ProgressIndicatorMode::Indeterminate => {
                if !self.was_active || self.active_sequence != activation_sequence {
                    self.phase = 0.0;
                    self.completed_cycle = false;
                    self.active_sequence = activation_sequence;
                }

                if !self.completed_cycle {
                    self.phase = (self.phase + phase_delta).min(1.0);
                    if self.phase >= 1.0 {
                        self.completed_cycle = true;
                        self.last_completed_sequence = Some(activation_sequence);
                    }
                }

                self.was_active = true;
            }
            ProgressIndicatorMode::Hidden => {
                if animation.finish_current_sweep && self.was_active && !self.completed_cycle {
                    self.phase = (self.phase + phase_delta).min(1.0);
                    self.completed_cycle = self.phase >= 1.0;
                    if self.completed_cycle {
                        self.last_completed_sequence = Some(activation_sequence);
                    }
                } else {
                    self.phase = 0.0;
                    self.completed_cycle = false;
                    self.was_active = false;
                    self.last_tick_at = None;
                }
            }
        }
    }

    pub(super) fn needs_animation(
        &self,
        mode: ProgressIndicatorMode,
        animation: ProgressAnimationConfig,
    ) -> bool {
        match mode {
            ProgressIndicatorMode::Indeterminate => !self.completed_cycle,
            ProgressIndicatorMode::Hidden => {
                animation.finish_current_sweep && self.was_active && !self.completed_cycle
            }
        }
    }

    pub(super) fn segments(
        &self,
        mode: ProgressIndicatorMode,
        track_width: f32,
        segment_width: f32,
        finish_current_sweep: bool,
    ) -> ProgressSegments {
        let track_width = track_width.max(0.0);
        let segment_width = segment_width.max(0.0).min(track_width);

        if track_width <= 0.0 || segment_width <= 0.0 {
            return ProgressSegments {
                leading_track: 0.0,
                active: 0.0,
            };
        }

        let show_active = matches!(mode, ProgressIndicatorMode::Indeterminate)
            || (finish_current_sweep && self.was_active && !self.completed_cycle);
        if !show_active {
            return ProgressSegments {
                leading_track: 0.0,
                active: 0.0,
            };
        }

        let start = self.phase.clamp(0.0, 1.0) * track_width;
        let active_start = start.clamp(0.0, track_width);
        let active_end = (start + segment_width).clamp(0.0, track_width);

        ProgressSegments {
            leading_track: active_start,
            active: (active_end - active_start).max(0.0),
        }
    }

    #[cfg(test)]
    pub(super) fn mark_completed_for_testing(&mut self, activation_sequence: u64) {
        self.last_completed_sequence = Some(activation_sequence);
        self.completed_cycle = true;
        self.was_active = false;
        self.phase = 0.0;
        self.last_tick_at = None;
    }

    #[cfg(test)]
    fn tick_by(
        &mut self,
        mode: ProgressIndicatorMode,
        phase_delta: f32,
        finish_current_sweep: bool,
        activation_sequence: u64,
    ) {
        let animation = ProgressAnimationConfig {
            cycles_per_second: 1.0,
            rows_per_segment: 1.5,
            min_segment_px: 18.0,
            max_segment_ratio: 0.4,
            finish_current_sweep,
        };

        match mode {
            ProgressIndicatorMode::Indeterminate => {
                if !self.was_active || self.active_sequence != activation_sequence {
                    self.phase = 0.0;
                    self.completed_cycle = false;
                    self.active_sequence = activation_sequence;
                }
                if !self.completed_cycle {
                    self.phase = (self.phase + phase_delta.max(0.0)).min(1.0);
                    if self.phase >= 1.0 {
                        self.completed_cycle = true;
                        self.last_completed_sequence = Some(activation_sequence);
                    }
                }
                self.was_active = true;
            }
            ProgressIndicatorMode::Hidden => {
                if animation.finish_current_sweep && self.was_active && !self.completed_cycle {
                    self.phase = (self.phase + phase_delta.max(0.0)).min(1.0);
                    self.completed_cycle = self.phase >= 1.0;
                    if self.completed_cycle {
                        self.last_completed_sequence = Some(activation_sequence);
                    }
                } else {
                    self.phase = 0.0;
                    self.completed_cycle = false;
                    self.was_active = false;
                    self.last_tick_at = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProgressConfig, ProgressContext, ProgressIndicator, ProgressIndicatorMode};

    #[test]
    fn disabled_config_always_hides_progress() {
        let config = ProgressConfig::disabled();
        let context = ProgressContext {
            manual_expanded: true,
            expanding: true,
            collapsing: false,
            search_in_flight: true,
            app_refresh_in_flight: true,
            icon_resolve_in_flight: true,
        };

        assert_eq!(config.mode(context), ProgressIndicatorMode::Hidden);
    }

    #[test]
    fn manual_expansion_profile_activates_for_manual_expansion() {
        let config = ProgressConfig::manual_expand_indeterminate();

        assert_eq!(
            config.mode(ProgressContext {
                manual_expanded: true,
                expanding: false,
                collapsing: false,
                search_in_flight: false,
                app_refresh_in_flight: false,
                icon_resolve_in_flight: false,
            }),
            ProgressIndicatorMode::Indeterminate
        );
    }

    #[test]
    fn one_shot_animation_stops_after_reaching_end() {
        let mut indicator = ProgressIndicator::default();

        for _ in 0..40 {
            indicator.tick_by(ProgressIndicatorMode::Indeterminate, 0.05, false, 1);
        }

        assert!(!indicator.needs_animation(
            ProgressIndicatorMode::Indeterminate,
            ProgressConfig::manual_expand_indeterminate().animation()
        ));

        let segments = indicator.segments(ProgressIndicatorMode::Indeterminate, 120.0, 30.0, false);
        assert!((segments.leading_track - 120.0).abs() < f32::EPSILON);
        assert_eq!(segments.active, 0.0);
    }

    #[test]
    fn new_manual_expand_sequence_restarts_one_shot() {
        let mut indicator = ProgressIndicator::default();

        for _ in 0..40 {
            indicator.tick_by(ProgressIndicatorMode::Indeterminate, 0.05, false, 1);
        }

        indicator.tick_by(ProgressIndicatorMode::Indeterminate, 0.0, false, 2);
        let reset = indicator.segments(ProgressIndicatorMode::Indeterminate, 120.0, 30.0, false);
        assert!(reset.leading_track <= f32::EPSILON);
    }

    #[test]
    fn completion_state_is_stable_after_hidden_reset() {
        let mut indicator = ProgressIndicator::default();

        for _ in 0..40 {
            indicator.tick_by(ProgressIndicatorMode::Indeterminate, 0.05, false, 7);
        }

        indicator.tick_by(ProgressIndicatorMode::Hidden, 0.0, false, 7);
        assert!(indicator.completed_for(7));
    }
}
