mod commands;
mod events;

#[cfg(test)]
mod tests {
    use super::super::super::{Launcher, Message};
    use crate::core::app_command::AppCommand;
    use crate::core::desktop::DesktopApp;
    use crate::core::search::ApplicationSearchResponse;
    use std::sync::mpsc;

    fn launcher() -> Launcher {
        let (_tx, rx) = mpsc::channel::<AppCommand>();
        let (launcher, _) = Launcher::new(rx, crate::core::tray::TrayController::detached());
        launcher
    }

    fn app(name: &str) -> DesktopApp {
        DesktopApp::new(
            name.to_string(),
            "Application".to_string(),
            format!("/usr/bin/{name}"),
            format!("/usr/bin/{name}"),
            Vec::new(),
            None,
            Vec::new(),
            None,
        )
    }

    #[test]
    fn refresh_command_replaces_apps_and_clears_inflight_state() {
        let mut launcher = launcher();

        let _ = launcher.request_app_refresh(true);
        assert!(launcher.app_refresh_in_flight());

        let _ = launcher.update(Message::AppsLoaded(vec![app("new-app")]));

        assert!(!launcher.app_refresh_in_flight());
        assert_eq!(launcher.app_count(), 1);
    }

    #[test]
    fn search_results_do_not_force_scroll_back_to_selection() {
        let mut launcher = launcher();
        launcher.set_apps((0..20).map(|index| app(&format!("app-{index}"))).collect());
        launcher.results_scroll_offset = 232.0;

        let _ = launcher.update(Message::SearchResultsLoaded(ApplicationSearchResponse {
            generation: 0,
            matches: (0..20).collect(),
        }));

        assert_eq!(launcher.results_scroll_offset, 232.0);
    }

    #[test]
    fn suppressed_query_change_does_not_update_search_text() {
        let mut launcher = launcher();
        launcher.update_query("fire".to_string());
        launcher.suppress_next_query_change();

        let _ = launcher.update(Message::QueryChanged("fire1".to_string()));

        assert_eq!(launcher.query, "fire");
    }
}
