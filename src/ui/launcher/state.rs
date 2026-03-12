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

pub(in crate::ui) fn render_range_for_viewport(
    offset_y: f32,
    viewport_height: f32,
    total_rows: usize,
    row_height: f32,
    row_gap: f32,
) -> Range<usize> {
    if total_rows == 0 || row_height <= 0.0 {
        return 0..0;
    }

    let step = row_height + row_gap.max(0.0);
    if step <= 0.0 {
        return 0..0;
    }

    let max_start = total_rows.saturating_sub(1);
    let start = scroll_start_for_offset(offset_y, step, max_start)
        .saturating_sub(EXPANSION_RENDER_BUFFER_ROWS);

    if viewport_height <= 0.0 {
        return start..(start + 1).min(total_rows);
    }

    let visible_end = (((offset_y.max(0.0) + viewport_height) / step).ceil() as usize)
        .min(total_rows)
        .max(start.saturating_add(1));
    let end = visible_end
        .saturating_add(EXPANSION_RENDER_BUFFER_ROWS)
        .min(total_rows);

    start..end
}

pub(in crate::ui) fn spacer_height_for_rows(
    row_count: usize,
    row_height: f32,
    row_gap: f32,
) -> f32 {
    if row_count == 0 {
        0.0
    } else {
        (row_count as f32 * row_height) + ((row_count.saturating_sub(1) as f32) * row_gap)
    }
}

pub(super) fn scroll_start_for_offset(offset_y: f32, row_step: f32, max_start: usize) -> usize {
    if !offset_y.is_finite() || row_step <= 0.0 {
        return 0;
    }

    ((offset_y.max(0.0) / row_step).floor() as usize).min(max_start)
}

pub(super) fn clamp_scroll_offset(
    offset_y: f32,
    total_rows: usize,
    viewport_height: f32,
    row_height: f32,
    row_gap: f32,
) -> f32 {
    if total_rows == 0 {
        return 0.0;
    }

    let max_offset =
        (spacer_height_for_rows(total_rows, row_height, row_gap) - viewport_height).max(0.0);

    if !offset_y.is_finite() {
        0.0
    } else {
        offset_y.clamp(0.0, max_offset)
    }
}

pub(super) fn scroll_offset_for_selection(
    selected_rank: usize,
    current_offset: f32,
    viewport_height: f32,
    total_rows: usize,
    row_height: f32,
    row_gap: f32,
    selection_padding: f32,
) -> f32 {
    if total_rows == 0 || row_height <= 0.0 {
        return 0.0;
    }

    let step = row_height + row_gap.max(0.0);
    let rank = selected_rank.min(total_rows.saturating_sub(1));
    let selection_padding = selection_padding.max(0.0);
    let row_top = (rank as f32 * step) - selection_padding;
    let row_bottom = (rank as f32 * step) + row_height + selection_padding;
    let current_offset = clamp_scroll_offset(
        current_offset,
        total_rows,
        viewport_height,
        row_height,
        row_gap,
    );

    let target_offset = if row_top < current_offset {
        row_top
    } else if row_bottom > current_offset + viewport_height {
        row_bottom - viewport_height
    } else {
        current_offset
    };

    clamp_scroll_offset(
        target_offset,
        total_rows,
        viewport_height,
        row_height,
        row_gap,
    )
}

pub(super) fn move_selection(selected_rank: usize, item_count: usize, offset: isize) -> usize {
    if item_count == 0 {
        return 0;
    }

    let current = selected_rank.min(item_count.saturating_sub(1)) as isize;
    (current + offset).clamp(0, item_count as isize - 1) as usize
}

#[cfg(test)]
mod tests {
    use super::{
        SurfaceResize, animate_results, clamp_scroll_offset, is_manual_expansion_in_progress,
        move_selection, render_range_for_viewport, results_target, scroll_offset_for_selection,
        scroll_start_for_offset, spacer_height_for_rows,
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
    fn render_range_limits_work_to_visible_rows_plus_buffer() {
        assert_eq!(render_range_for_viewport(0.0, 288.0, 200, 54.0, 4.0), 0..6);
        assert_eq!(
            render_range_for_viewport(232.0, 288.0, 200, 54.0, 4.0),
            3..10
        );
        assert_eq!(
            render_range_for_viewport(464.0, 120.0, 10, 54.0, 4.0),
            7..10
        );
    }

    #[test]
    fn spacer_height_scales_with_hidden_rows() {
        assert_eq!(spacer_height_for_rows(0, 54.0, 4.0), 0.0);
        assert_eq!(spacer_height_for_rows(1, 54.0, 4.0), 54.0);
        assert_eq!(spacer_height_for_rows(3, 54.0, 4.0), 170.0);
    }

    #[test]
    fn scroll_offset_uses_floor_to_preserve_partial_row_visibility() {
        assert_eq!(scroll_start_for_offset(0.0, 58.0, 10), 0);
        assert_eq!(scroll_start_for_offset(28.9, 58.0, 10), 0);
        assert_eq!(scroll_start_for_offset(58.0, 58.0, 10), 1);
        assert_eq!(scroll_start_for_offset(119.0, 58.0, 10), 2);
    }

    #[test]
    fn selection_scrolling_keeps_rows_fully_visible() {
        assert_eq!(
            scroll_offset_for_selection(0, 0.0, 288.0, 20, 54.0, 4.0, 3.0),
            0.0
        );
        assert_eq!(
            scroll_offset_for_selection(5, 0.0, 288.0, 20, 54.0, 4.0, 3.0),
            59.0
        );
        assert_eq!(
            scroll_offset_for_selection(7, 116.0, 288.0, 20, 54.0, 4.0, 3.0),
            175.0
        );
    }

    #[test]
    fn scroll_offset_is_clamped_to_content_bounds() {
        assert_eq!(clamp_scroll_offset(500.0, 3, 288.0, 54.0, 4.0), 0.0);
        assert_eq!(clamp_scroll_offset(1_500.0, 20, 288.0, 54.0, 4.0), 868.0);
    }

    #[test]
    fn selection_movement_clamps_to_result_bounds() {
        assert_eq!(move_selection(0, 0, 1), 0);
        assert_eq!(move_selection(0, 3, -1), 0);
        assert_eq!(move_selection(1, 3, 1), 2);
        assert_eq!(move_selection(2, 3, 4), 2);
    }
}
