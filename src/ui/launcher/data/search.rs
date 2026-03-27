use super::super::Launcher;
use crate::core::desktop::{DesktopApp, normalize_query};
use crate::core::search::{ApplicationSearchResponse, rank_applications};
use log::error;

impl Launcher {
    pub(in crate::ui::launcher) fn bump_selection_revision(&mut self) {
        self.selection_revision = self.selection_revision.wrapping_add(1);
    }

    pub(in crate::ui::launcher) fn reset_selection_cursor_state(&mut self) {
        self.selected_rank = 0;
        self.highlighted_rank = 0;
        self.results_scroll_offset = 0.0;
        self.scroll_start_rank = 0;
    }

    pub(in crate::ui::launcher) fn update_query(&mut self, query: String) {
        self.bump_selection_revision();
        self.query = query;
        self.normalized_query = normalize_query(&self.query);
        self.search_generation = self.search_generation.wrapping_add(1);
        self.search_in_flight = !self.normalized_query.is_empty();
        self.reset_selection_cursor_state();
        self.sync_results_target_with_query();

        if self.normalized_query.is_empty() {
            self.filtered_indices = self.all_app_indices.clone();
            self.applied_search_generation = self.search_generation;
            self.search_in_flight = false;
        }

        self.submit_search_request();
    }

    pub(in crate::ui::launcher) fn set_apps(&mut self, apps: Vec<DesktopApp>) {
        self.bump_selection_revision();
        self.apps = apps;
        self.all_app_indices = (0..self.apps.len())
            .take(crate::ui::constants::MAX_RESULTS)
            .collect();
        self.search_in_flight = false;
        self.apply_search_results_sync();
        let _ = self.app_search_engine.replace_apps(&self.apps);
    }

    #[cfg(test)]
    pub(in crate::ui::launcher) fn app_count(&self) -> usize {
        self.apps.len()
    }

    pub(in crate::ui::launcher) fn apply_search_results(
        &mut self,
        response: ApplicationSearchResponse,
    ) -> bool {
        if response.generation != self.search_generation {
            return false;
        }

        self.search_in_flight = false;
        self.filtered_indices = response.matches;
        self.applied_search_generation = response.generation;
        self.reconcile_filtered_selection();
        true
    }

    pub(in crate::ui::launcher) fn reset_search_state(&mut self) {
        self.query.clear();
        self.normalized_query.clear();
        self.filtered_indices = self.all_app_indices.clone();
        self.search_generation = self.search_generation.wrapping_add(1);
        self.applied_search_generation = self.search_generation;
        self.search_in_flight = false;
        self.submit_search_request();
    }

    pub(in crate::ui::launcher) fn search_results_handle(
        &self,
    ) -> super::super::SearchResultsReceiverHandle {
        self.search_results_handle.clone()
    }

    fn reconcile_filtered_selection(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected_rank = 0;
            self.highlighted_rank = 0;
            self.results_scroll_offset = 0.0;
            self.scroll_start_rank = 0;
            return;
        }

        self.selected_rank = self.selected_rank.min(self.filtered_indices.len() - 1);
        self.highlighted_rank = self.highlighted_rank.min(self.filtered_indices.len() - 1);
        self.results_scroll_offset = super::super::display::state::clamp_scroll_offset(
            self.results_scroll_offset,
            self.filtered_indices.len(),
            self.layout.results_viewport_height(),
            self.layout.result_row_height,
            self.layout.result_row_gap,
        );
        self.scroll_start_rank = super::super::display::state::scroll_start_for_offset(
            self.results_scroll_offset,
            self.layout.result_row_scroll_step(),
            self.filtered_indices.len().saturating_sub(1),
        );
    }

    fn submit_search_request(&mut self) {
        if self
            .app_search_engine
            .search(self.search_generation, self.normalized_query.clone())
        {
            return;
        }

        error!("application search worker unavailable; falling back to inline ranking");
        self.search_in_flight = false;
        self.apply_search_results_sync();
    }

    fn apply_search_results_sync(&mut self) {
        self.filtered_indices = rank_applications(&self.apps, &self.normalized_query);
        self.applied_search_generation = self.search_generation;
        self.reconcile_filtered_selection();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::app_command::AppCommand;
    use crate::core::desktop::DesktopApp;
    use std::sync::mpsc;

    fn app(name: &str, command: &str, exec_line: &str) -> DesktopApp {
        DesktopApp::new(
            name.to_string(),
            "Application".to_string(),
            exec_line.to_string(),
            command.to_string(),
            Vec::new(),
            None,
            Vec::new(),
            None,
        )
    }

    fn launcher() -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (launcher, _) = Launcher::new(rx, crate::core::tray::TrayController::detached());
        launcher
    }

    #[test]
    fn query_results_arrive_through_generation_guard() {
        let mut launcher = launcher();
        launcher.set_apps(vec![
            app("Firefox", "/usr/bin/firefox", "/usr/bin/firefox %u"),
            app(
                "Files",
                "/usr/bin/nautilus",
                "/usr/bin/nautilus --new-window",
            ),
        ]);

        launcher.update_query("fire".to_string());

        assert_eq!(launcher.filtered_indices(), &[0, 1]);
        assert!(launcher.search_in_flight);
        assert!(launcher.selected_result_index().is_none());

        assert!(launcher.apply_search_results(ApplicationSearchResponse {
            generation: launcher.search_generation,
            matches: vec![0],
        }));

        assert_eq!(launcher.filtered_indices(), &[0]);
        assert_eq!(launcher.selected_result_index(), Some(0));
    }

    #[test]
    fn query_reset_clears_scroll_offset() {
        let mut launcher = launcher();
        launcher.set_apps(vec![app(
            "Firefox",
            "/usr/bin/firefox",
            "/usr/bin/firefox %u",
        )]);
        launcher.results_scroll_offset = 120.0;

        launcher.update_query("fire".to_string());

        assert_eq!(launcher.results_scroll_offset, 0.0);
    }

    #[test]
    fn stale_results_are_ignored() {
        let mut launcher = launcher();
        launcher.set_apps(vec![app(
            "Firefox",
            "/usr/bin/firefox",
            "/usr/bin/firefox %u",
        )]);

        launcher.update_query("fir".to_string());
        let current_generation = launcher.search_generation;
        launcher.update_query("fire".to_string());

        assert!(!launcher.apply_search_results(ApplicationSearchResponse {
            generation: current_generation,
            matches: vec![0],
        }));
        assert_eq!(launcher.filtered_indices(), &[0]);
        assert!(launcher.search_in_flight);
    }

    #[test]
    fn app_refresh_preserves_scroll_offset() {
        let mut launcher = launcher();
        launcher.set_apps(
            (0..20)
                .map(|index| {
                    app(
                        &format!("App {index}"),
                        &format!("/usr/bin/app-{index}"),
                        "app",
                    )
                })
                .collect(),
        );
        launcher.results_scroll_offset = 232.0;

        launcher.set_apps(
            (0..21)
                .map(|index| {
                    app(
                        &format!("App {index}"),
                        &format!("/usr/bin/app-{index}"),
                        "app",
                    )
                })
                .collect(),
        );

        assert_eq!(launcher.results_scroll_offset, 232.0);
        assert_eq!(launcher.scroll_start_rank, 4);
    }
}
