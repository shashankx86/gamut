use crate::ui::layout::LauncherLayout;

const SNAP_THRESHOLD: f32 = 0.01;

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
        SurfaceResize, animate_results, move_selection, results_target, scroll_start_for_selection,
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
}
