use super::super::super::{Launcher, Message};
use iced::Task;
use iced::widget::{operation, scrollable};

impl Launcher {
    pub(in crate::ui::launcher) fn scroll_to_selected(
        &mut self,
        previous_rank: usize,
        force: bool,
    ) -> Task<Message> {
        if self.selected_result_index().is_none() {
            self.results_scroll_offset = 0.0;
            self.scroll_start_rank = 0;
            self.highlighted_rank = 0;
            return if force {
                self.mark_programmatic_results_scroll(0.0);
                operation::scroll_to(
                    self.results_scroll_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: None,
                        y: Some(0.0),
                    },
                )
            } else {
                Task::none()
            };
        }

        let total_rows = self.filtered_indices().len();
        let previous_offset = self.results_scroll_offset;
        let target_offset = super::super::super::display::state::scroll_offset_for_selection(
            self.selected_rank,
            previous_offset,
            self.layout.results_viewport_height(),
            total_rows,
            self.layout.result_row_height,
            self.layout.result_row_gap,
            self.layout.result_row_inset_y,
        );
        self.results_scroll_offset = target_offset;
        self.scroll_start_rank = super::super::super::display::state::scroll_start_for_offset(
            target_offset,
            self.layout.result_row_scroll_step(),
            total_rows.saturating_sub(1),
        );
        let did_scroll = (target_offset - previous_offset).abs() > f32::EPSILON;

        if did_scroll && self.selected_rank != previous_rank {
            self.highlighted_rank = previous_rank;
            let revision = self.selection_revision;
            self.mark_programmatic_results_scroll(target_offset);
            operation::scroll_to(
                self.results_scroll_id.clone(),
                scrollable::AbsoluteOffset {
                    x: None,
                    y: Some(target_offset),
                },
            )
            .chain(Task::done(Message::SyncHighlightedRank {
                revision,
                rank: self.selected_rank,
            }))
        } else {
            self.highlighted_rank = self.selected_rank;
            if !force && !did_scroll {
                Task::none()
            } else {
                self.mark_programmatic_results_scroll(target_offset);
                operation::scroll_to(
                    self.results_scroll_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: None,
                        y: Some(target_offset),
                    },
                )
            }
        }
    }

    pub(in crate::ui::launcher) fn on_results_scrolled(
        &mut self,
        viewport: scrollable::Viewport,
    ) -> Task<Message> {
        let row_step = self.layout.result_row_scroll_step();

        if row_step <= 0.0 {
            return Task::none();
        }

        let offset = super::super::super::display::state::clamp_scroll_offset(
            viewport.absolute_offset().y,
            self.filtered_indices().len(),
            viewport.bounds().height,
            self.layout.result_row_height,
            self.layout.result_row_gap,
        );

        let start = super::super::super::display::state::scroll_start_for_offset(
            offset,
            row_step,
            self.filtered_indices().len().saturating_sub(1),
        );

        let is_programmatic = self.consume_programmatic_results_scroll_event(offset);
        let did_move = (offset - self.results_scroll_offset).abs() > f32::EPSILON;

        if !is_programmatic && did_move {
            self.reveal_results_scrollbar_for_mouse_scroll();
        }

        self.results_scroll_offset = offset;
        self.scroll_start_rank = start;
        Task::none()
    }
}
