use super::super::display;
use super::super::Message;
use super::Launcher;
use super::ProgressIndicator;
use iced::widget::operation;
use iced::{window, Task};

impl Launcher {
    pub(in crate::ui::launcher) fn clear_window_state(&mut self) {
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

    pub(in crate::ui) fn results_progress(&self) -> f32 {
        self.results_progress
    }

    pub(in crate::ui::launcher) fn sync_results_target_with_query(&mut self) {
        self.results_target = display::state::results_target(
            self.normalized_query.is_empty(),
            self.manually_expanded,
        );
    }

    pub(in crate::ui::launcher) fn animate_results(&mut self) -> Task<Message> {
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
            display::state::SurfaceResize::Expanded => {
                self.request_surface_resize(id, self.layout.expanded_surface_size())
            }
            display::state::SurfaceResize::Collapsed => {
                self.request_surface_resize(id, self.layout.collapsed_surface_size())
            }
        }
    }

    pub(in crate::ui::launcher) fn focus_search_input(&self) -> Task<Message> {
        operation::focus(self.input_id.clone())
    }

    pub(in crate::ui::launcher) fn with_search_focus_if_visible(
        &self,
        task: Task<Message>,
    ) -> Task<Message> {
        if self.is_visible && self.launcher_window_id.is_some() {
            task.chain(self.focus_search_input())
        } else {
            task
        }
    }

    pub(in crate::ui::launcher) fn request_surface_resize(
        &self,
        id: window::Id,
        size: (u32, u32),
    ) -> Task<Message> {
        self.with_search_focus_if_visible(Task::done(Message::SizeChange { id, size }))
    }

    pub(in crate::ui::launcher) fn selected_result_index(&self) -> Option<usize> {
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

    pub(in crate::ui::launcher) fn selected_application_index(&self) -> Option<usize> {
        let index = self.selected_result_index()?;
        let app = self.apps.get(index)?;
        (crate::core::preferences::normalize_identifier(&app.entry_type) == "application")
            .then_some(index)
    }

    pub(in crate::ui) fn is_expanded(&self) -> bool {
        self.results_progress > 0.0 || self.results_target > 0.0
    }

    pub(in crate::ui) fn should_show_action_overlay(&self) -> bool {
        self.is_expanded()
            && self.selected_application_index().is_some()
            && ((self.modifiers.alt() && !self.suppress_alt_actions_until_release)
                || self.action_overlay_pinned)
    }

    pub(in crate::ui::launcher) fn suppress_alt_actions_until_release(&mut self) {
        self.suppress_alt_actions_until_release = true;
    }

    pub(in crate::ui::launcher) fn sync_alt_action_state_with_modifiers(&mut self) {
        if !self.modifiers.alt() {
            self.suppress_alt_actions_until_release = false;
            self.suppress_next_query_change = false;
        }
    }

    pub(in crate::ui::launcher) fn suppress_next_query_change(&mut self) {
        self.suppress_next_query_change = true;
    }

    pub(in crate::ui::launcher) fn consume_suppressed_query_change(&mut self) -> bool {
        if !self.suppress_next_query_change {
            return false;
        }

        self.suppress_next_query_change = false;
        true
    }

    pub(in crate::ui::launcher) fn move_selection(&mut self, offset: isize) {
        self.selected_rank =
            display::state::move_selection(self.selected_rank, self.filtered_indices.len(), offset);
    }
}
