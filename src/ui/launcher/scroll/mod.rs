use super::Launcher;
use std::time::{Duration, Instant};

const PROGRAMMATIC_SCROLL_MATCH_EPSILON: f32 = 0.5;
const SCROLLBAR_VISIBLE_FOR: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(in crate::ui::launcher) struct ResultsScrollbarVisibility {
    reveal_until: Option<Instant>,
    was_visible_last_tick: bool,
    programmatic_scroll_target: Option<f32>,
}

impl ResultsScrollbarVisibility {
    fn reveal_temporarily(&mut self) {
        self.reveal_until = Some(Instant::now() + SCROLLBAR_VISIBLE_FOR);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn is_revealed(self) -> bool {
        self.is_revealed_at(Instant::now())
    }

    fn is_revealed_at(self, now: Instant) -> bool {
        self.reveal_until.is_some_and(|deadline| now <= deadline)
    }

    fn mark_programmatic_scroll(&mut self, target_offset: f32) {
        if target_offset.is_finite() {
            self.programmatic_scroll_target = Some(target_offset.max(0.0));
        }
    }

    fn consume_programmatic_scroll_match(&mut self, observed_offset: f32) -> bool {
        let Some(expected_offset) = self.programmatic_scroll_target.take() else {
            return false;
        };

        observed_offset.is_finite()
            && (observed_offset - expected_offset).abs() <= PROGRAMMATIC_SCROLL_MATCH_EPSILON
    }
}

impl Launcher {
    pub(in crate::ui::launcher) fn reset_results_scrollbar_visibility(&mut self) {
        self.results_scrollbar_visibility.reset();
    }

    pub(in crate::ui::launcher) fn reveal_results_scrollbar_for_mouse_scroll(&mut self) {
        self.results_scrollbar_visibility.reveal_temporarily();
    }

    pub(in crate::ui::launcher) fn on_scrollbar_visibility_tick(
        &mut self,
    ) -> iced::Task<super::Message> {
        let is_visible_now = self.should_show_results_scrollbar();
        let was_visible = self.results_scrollbar_visibility.was_visible_last_tick;
        self.results_scrollbar_visibility.was_visible_last_tick = is_visible_now;

        if is_visible_now == was_visible {
            return iced::Task::none();
        }

        self.mark_programmatic_results_scroll(self.results_scroll_offset);

        iced::widget::operation::scroll_to(
            self.results_scroll_id.clone(),
            iced::widget::scrollable::AbsoluteOffset {
                x: None,
                y: Some(self.results_scroll_offset),
            },
        )
    }

    pub(in crate::ui::launcher) fn mark_programmatic_results_scroll(&mut self, target_offset: f32) {
        self.results_scrollbar_visibility
            .mark_programmatic_scroll(target_offset);
    }

    pub(in crate::ui::launcher) fn consume_programmatic_results_scroll_event(
        &mut self,
        observed_offset: f32,
    ) -> bool {
        self.results_scrollbar_visibility
            .consume_programmatic_scroll_match(observed_offset)
    }

    pub(in crate::ui::launcher) fn reveal_results_scrollbar_for_keyboard_end(
        &mut self,
        previous_rank: usize,
    ) {
        let total = self.filtered_indices.len();

        if total == 0 {
            return;
        }

        let last_rank = total.saturating_sub(1);

        if self.selected_rank == last_rank && previous_rank != last_rank {
            self.results_scrollbar_visibility.reveal_temporarily();
        }
    }

    pub(in crate::ui) fn should_show_results_scrollbar(&self) -> bool {
        self.results_scrollbar_visibility.is_revealed() && self.results_are_scrollable()
    }

    fn results_are_scrollable(&self) -> bool {
        super::display::state::max_scroll_offset(
            self.filtered_indices.len(),
            self.layout.results_viewport_height(),
            self.layout.result_row_height,
            self.layout.result_row_gap,
        ) > f32::EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::app_command::AppCommand;
    use crate::core::desktop::DesktopApp;
    use std::sync::mpsc;

    fn app(index: usize) -> DesktopApp {
        DesktopApp::new(
            format!("App {index}"),
            "Application".to_string(),
            format!("/usr/bin/app-{index} %u"),
            format!("/usr/bin/app-{index}"),
            vec!["%u".to_string()],
            None,
            Vec::new(),
            None,
        )
    }

    fn launcher_with_results(total_results: usize) -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (mut launcher, _) = Launcher::new(rx, crate::core::tray::TrayController::detached());
        launcher.apps = (0..total_results).map(app).collect();
        launcher.all_app_indices = (0..launcher.apps.len()).collect();
        launcher.filtered_indices = launcher.all_app_indices.clone();
        launcher
    }

    #[test]
    fn scrollbar_stays_hidden_until_a_reveal_trigger_occurs() {
        let launcher = launcher_with_results(20);

        assert!(!launcher.should_show_results_scrollbar());
    }

    #[test]
    fn mouse_scroll_reveals_results_scrollbar() {
        let mut launcher = launcher_with_results(20);

        launcher.reveal_results_scrollbar_for_mouse_scroll();

        assert!(launcher.should_show_results_scrollbar());
    }

    #[test]
    fn keyboard_reaching_last_result_reveals_results_scrollbar() {
        let mut launcher = launcher_with_results(20);
        launcher.selected_rank = 19;

        launcher.reveal_results_scrollbar_for_keyboard_end(18);

        assert!(launcher.should_show_results_scrollbar());
    }

    #[test]
    fn keyboard_navigation_does_not_reveal_when_not_at_last_result() {
        let mut launcher = launcher_with_results(20);
        launcher.selected_rank = 5;

        launcher.reveal_results_scrollbar_for_keyboard_end(4);

        assert!(!launcher.should_show_results_scrollbar());
    }

    #[test]
    fn reset_hides_scrollbar_after_any_reveal() {
        let mut launcher = launcher_with_results(20);
        launcher.reveal_results_scrollbar_for_mouse_scroll();

        launcher.reset_results_scrollbar_visibility();

        assert!(!launcher.should_show_results_scrollbar());
    }

    #[test]
    fn scrollbar_auto_hides_after_deadline_expires() {
        let mut launcher = launcher_with_results(20);
        launcher.reveal_results_scrollbar_for_mouse_scroll();

        launcher.results_scrollbar_visibility.reveal_until =
            Some(Instant::now() - Duration::from_secs(1));

        assert!(!launcher.should_show_results_scrollbar());
    }

    #[test]
    fn programmatic_scroll_event_is_consumed_once() {
        let mut launcher = launcher_with_results(20);

        launcher.mark_programmatic_results_scroll(120.0);

        assert!(launcher.consume_programmatic_results_scroll_event(120.0));
        assert!(!launcher.consume_programmatic_results_scroll_event(120.0));
    }
}
