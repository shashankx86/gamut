use crate::ui::layout::LauncherLayout;
use std::ops::Range;

const SNAP_THRESHOLD: f32 = 0.01;
const EXPANSION_RENDER_BUFFER_ROWS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfaceResize {
    None,
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct AnimationStep {
    pub(super) next_progress: f32,
    pub(super) surface_resize: SurfaceResize,
}

pub(super) fn results_target(query_is_empty: bool, manually_expanded: bool) -> f32 {
    if query_is_empty && !manually_expanded {
        0.0
    } else {
        1.0
    }
}

pub(super) fn animate_results(
    progress: f32,
    target: f32,
    layout: &LauncherLayout,
) -> AnimationStep {
    let delta = target - progress;
    let mut surface_resize = SurfaceResize::None;

    if progress == 0.0 && target > 0.0 {
        surface_resize = SurfaceResize::Expanded;
    }

    let next_progress = if delta.abs() < SNAP_THRESHOLD {
        target
    } else {
        (progress + delta * layout.results_animation_speed).clamp(0.0, 1.0)
    };

    if delta.abs() < SNAP_THRESHOLD && next_progress == 0.0 {
        surface_resize = SurfaceResize::Collapsed;
    }

    AnimationStep {
        next_progress,
        surface_resize,
    }
}

#[cfg(test)]
fn is_manual_expansion_in_progress(
    query_is_empty: bool,
    manually_expanded: bool,
    progress: f32,
    target: f32,
) -> bool {
    query_is_empty && manually_expanded && target > progress && progress < 1.0
}

pub(in crate::ui) fn expansion_render_range(
    start: usize,
    total_rows: usize,
    visible_rows: usize,
) -> Range<usize> {
    if total_rows == 0 {
        return 0..0;
    }

    let start = start.min(total_rows.saturating_sub(1));
    let render_rows = visible_rows
        .saturating_add(EXPANSION_RENDER_BUFFER_ROWS)
        .max(1);
    let end = start.saturating_add(render_rows).min(total_rows);

    start..end
}

pub(in crate::ui) fn spacer_height_for_rows(row_count: usize, row_step: f32) -> f32 {
    row_count as f32 * row_step
}

pub(super) fn move_selection(selected_rank: usize, item_count: usize, offset: isize) -> usize {
    if item_count == 0 {
        return 0;
    }

    let current = selected_rank.min(item_count.saturating_sub(1)) as isize;
    (current + offset).clamp(0, item_count as isize - 1) as usize
}

pub(super) fn scroll_start_for_selection(
    selected_rank: usize,
    previous_start: usize,
    total_rows: usize,
    visible_rows: usize,
) -> usize {
    if total_rows == 0 {
        return 0;
    }

    let max_start = total_rows.saturating_sub(visible_rows.max(1));
    let mut start = previous_start.min(max_start);

    if selected_rank < start {
        start = selected_rank;
    } else if selected_rank >= start + visible_rows {
        start = selected_rank.saturating_sub(visible_rows.saturating_sub(1));
    }

    start.min(max_start)
}

#[cfg(test)]
mod tests {
    use super::{
        SurfaceResize, animate_results, expansion_render_range, is_manual_expansion_in_progress,
        move_selection, results_target, scroll_start_for_selection, spacer_height_for_rows,
    };
    use crate::ui::layout::LauncherLayout;

    #[test]
    fn empty_query_stays_collapsed_until_manually_expanded() {
        assert_eq!(results_target(true, false), 0.0);
        assert_eq!(results_target(true, true), 1.0);
        assert_eq!(results_target(false, false), 1.0);
    }

    #[test]
    fn expanding_requests_larger_surface_on_first_animation_tick() {
        let layout = LauncherLayout::fallback();
        let step = animate_results(0.0, 1.0, &layout);

        assert_eq!(step.surface_resize, SurfaceResize::Expanded);
        assert!(step.next_progress > 0.0);
    }

    #[test]
    fn collapsing_waits_until_snap_before_shrinking_surface() {
        let layout = LauncherLayout::fallback();
        let step = animate_results(0.005, 0.0, &layout);

        assert_eq!(step.next_progress, 0.0);
        assert_eq!(step.surface_resize, SurfaceResize::Collapsed);
    }

    #[test]
    fn manual_expansion_detection_is_limited_to_empty_query_expand_animation() {
        assert!(is_manual_expansion_in_progress(true, true, 0.25, 1.0));
        assert!(!is_manual_expansion_in_progress(false, true, 0.25, 1.0));
        assert!(!is_manual_expansion_in_progress(true, false, 0.25, 1.0));
        assert!(!is_manual_expansion_in_progress(true, true, 1.0, 1.0));
    }

    #[test]
    fn expansion_render_range_limits_work_to_visible_rows_plus_buffer() {
        assert_eq!(expansion_render_range(0, 200, 5), 0..6);
        assert_eq!(expansion_render_range(4, 200, 5), 4..10);
        assert_eq!(expansion_render_range(8, 10, 5), 8..10);
    }

    #[test]
    fn spacer_height_scales_with_hidden_rows() {
        assert_eq!(spacer_height_for_rows(0, 58.0), 0.0);
        assert_eq!(spacer_height_for_rows(3, 58.0), 174.0);
    }

    #[test]
    fn selection_movement_clamps_to_result_bounds() {
        assert_eq!(move_selection(0, 0, 1), 0);
        assert_eq!(move_selection(0, 3, -1), 0);
        assert_eq!(move_selection(1, 3, 1), 2);
        assert_eq!(move_selection(2, 3, 4), 2);
    }

    #[test]
    fn scroll_offset_keeps_selected_result_visible() {
        assert_eq!(scroll_start_for_selection(0, 0, 10, 5), 0);
        assert_eq!(scroll_start_for_selection(5, 0, 10, 5), 1);
        assert_eq!(scroll_start_for_selection(8, 5, 10, 5), 5);
        assert_eq!(scroll_start_for_selection(2, 4, 10, 5), 2);
    }

    #[test]
    fn scroll_offset_clamps_when_viewport_fits_fewer_rows() {
        assert_eq!(scroll_start_for_selection(6, 0, 10, 6), 1);
        assert_eq!(scroll_start_for_selection(9, 1, 10, 6), 4);
    }
}
