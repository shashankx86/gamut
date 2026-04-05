use super::Launcher;

impl Launcher {
    pub(in crate::ui) fn resolved_appearance(&self) -> crate::ui::theme::ResolvedAppearance {
        self.visual_cache.appearance
    }

    pub(in crate::ui) fn window_theme(&self) -> iced::Theme {
        self.visual_cache.window_theme.clone()
    }

    pub(in crate::ui) fn launcher_logo_handle(&self) -> iced::widget::svg::Handle {
        self.visual_cache.logo_handle.clone()
    }

    pub(in crate::ui::launcher) fn refresh_visual_cache(&mut self) {
        self.visual_cache = super::LauncherVisualCache::build(&self.app_preferences);
    }
}
