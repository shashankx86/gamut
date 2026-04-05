mod events;
mod key_candidates;
mod keyboard;
mod scrolling;

#[cfg(test)]
mod tests {
    use super::key_candidates::normalize_binding_key;
    use crate::core::app_command::AppCommand;
    use crate::core::desktop::DesktopApp;
    use crate::ui::launcher::Launcher;
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
    fn binding_key_normalization_ignores_spacing_and_case() {
        assert_eq!(normalize_binding_key(" Arrow-Down "), "arrowdown");
        assert_eq!(normalize_binding_key("Page_Up"), "pageup");
    }

    #[test]
    fn scrolling_selection_uses_precise_pixel_offset() {
        let mut launcher = launcher_with_results(20);
        launcher.selected_rank = 5;

        let _ = launcher.scroll_to_selected(0, false);

        let row_step = launcher.layout.result_row_scroll_step();
        let row_top = launcher.selected_rank as f32 * row_step;
        let row_bottom = row_top + launcher.layout.result_row_height;
        let viewport_top = launcher.results_scroll_offset;
        let viewport_bottom = viewport_top + launcher.layout.results_viewport_height();

        assert!(row_top >= viewport_top + launcher.layout.result_row_inset_y);
        assert!(row_bottom <= viewport_bottom - launcher.layout.result_row_inset_y);
    }
}
