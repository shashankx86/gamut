use crate::ui::layout::LauncherLayout;

const SNAP_THRESHOLD: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::ui::launcher) enum SurfaceResize {
    None,
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::ui::launcher) struct AnimationStep {
    pub(in crate::ui::launcher) next_progress: f32,
    pub(in crate::ui::launcher) surface_resize: SurfaceResize,
}

pub(in crate::ui::launcher) fn results_target(
    query_is_empty: bool,
    manually_expanded: bool,
) -> f32 {
    if query_is_empty && !manually_expanded {
        0.0
    } else {
        1.0
    }
}

pub(in crate::ui::launcher) fn animate_results(
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

pub(in crate::ui::launcher) fn scroll_start_for_offset(
    offset_y: f32,
    row_step: f32,
    max_start: usize,
) -> usize {
    if !offset_y.is_finite() || row_step <= 0.0 {
        return 0;
    }

    ((offset_y.max(0.0) / row_step).floor() as usize).min(max_start)
}

pub(in crate::ui::launcher) fn clamp_scroll_offset(
    offset_y: f32,
    total_rows: usize,
    viewport_height: f32,
    row_height: f32,
    row_gap: f32,
) -> f32 {
    let max_offset = max_scroll_offset(total_rows, viewport_height, row_height, row_gap);

    if !offset_y.is_finite() {
        0.0
    } else {
        offset_y.clamp(0.0, max_offset)
    }
}

pub(in crate::ui::launcher) fn max_scroll_offset(
    total_rows: usize,
    viewport_height: f32,
    row_height: f32,
    row_gap: f32,
) -> f32 {
    if total_rows == 0 {
        return 0.0;
    }

    let content_height = (total_rows as f32 * row_height)
        + ((total_rows.saturating_sub(1) as f32) * row_gap.max(0.0));

    (content_height - viewport_height.max(0.0)).max(0.0)
}

pub(in crate::ui::launcher) fn scroll_offset_for_selection(
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

pub(in crate::ui::launcher) fn move_selection(
    selected_rank: usize,
    item_count: usize,
    offset: isize,
) -> usize {
    if item_count == 0 {
        return 0;
    }

    let current = selected_rank.min(item_count.saturating_sub(1)) as isize;
    (current + offset).clamp(0, item_count as isize - 1) as usize
}

#[cfg(test)]
mod tests {
    use super::{
        animate_results, clamp_scroll_offset, is_manual_expansion_in_progress, max_scroll_offset,
        move_selection, results_target, scroll_offset_for_selection, scroll_start_for_offset,
        SurfaceResize,
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
    fn scroll_offset_uses_floor_to_preserve_partial_row_visibility() {
        let first = scroll_start_for_offset(0.0, 58.0, 10);
        let mid_row = scroll_start_for_offset(28.9, 58.0, 10);
        let next_row = scroll_start_for_offset(58.0, 58.0, 10);
        let later_row = scroll_start_for_offset(119.0, 58.0, 10);

        assert!(first <= mid_row);
        assert!(mid_row <= next_row);
        assert!(next_row <= later_row);
        assert_eq!(first, mid_row);
        assert!(next_row > mid_row);
        assert!(later_row <= 10);
    }

    #[test]
    fn selection_scrolling_keeps_rows_fully_visible() {
        let viewport = 288.0;
        let total_rows = 20;
        let row_height = 54.0;
        let row_gap = 4.0;
        let padding = 3.0;
        let step = row_height + row_gap;

        for (selection, current_offset) in [(0usize, 0.0f32), (5, 0.0), (7, 116.0)] {
            let offset = scroll_offset_for_selection(
                selection,
                current_offset,
                viewport,
                total_rows,
                row_height,
                row_gap,
                padding,
            );
            let clamped_current =
                clamp_scroll_offset(current_offset, total_rows, viewport, row_height, row_gap);
            let row_top = (selection as f32 * step) - padding;
            let row_bottom = (selection as f32 * step) + row_height + padding;

            assert!(offset >= 0.0);
            assert!(offset <= max_scroll_offset(total_rows, viewport, row_height, row_gap));
            assert!(row_top >= offset || row_bottom <= offset + viewport);
            if row_top >= clamped_current && row_bottom <= clamped_current + viewport {
                assert_eq!(offset, clamped_current);
            }
        }
    }

    #[test]
    fn scroll_offset_is_clamped_to_content_bounds() {
        let small_max = max_scroll_offset(3, 288.0, 54.0, 4.0);
        let large_max = max_scroll_offset(20, 288.0, 54.0, 4.0);

        assert_eq!(small_max, 0.0);
        assert_eq!(clamp_scroll_offset(500.0, 3, 288.0, 54.0, 4.0), small_max);
        assert_eq!(clamp_scroll_offset(-10.0, 20, 288.0, 54.0, 4.0), 0.0);
        assert_eq!(
            clamp_scroll_offset(1_500.0, 20, 288.0, 54.0, 4.0),
            large_max
        );
    }

    #[test]
    fn max_scroll_offset_reports_zero_for_non_overflowing_content() {
        let non_overflowing = max_scroll_offset(3, 288.0, 54.0, 4.0);
        let overflowing = max_scroll_offset(20, 288.0, 54.0, 4.0);

        assert_eq!(non_overflowing, 0.0);
        assert!(overflowing > 0.0);
        assert!(
            max_scroll_offset(25, 288.0, 54.0, 4.0) > overflowing,
            "adding rows should increase scrollable content"
        );
    }

    #[test]
    fn selection_movement_clamps_to_result_bounds() {
        assert_eq!(move_selection(0, 0, 1), 0);
        assert_eq!(move_selection(0, 3, -1), 0);
        assert_eq!(move_selection(1, 3, 1), 2);
        assert_eq!(move_selection(2, 3, 4), 2);
    }
}
