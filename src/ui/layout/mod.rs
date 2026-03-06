mod metrics;
mod preferences;

pub(in crate::ui) use metrics::LauncherLayout;
pub(in crate::ui) use preferences::LauncherPreferences;

#[cfg(test)]
mod tests {
    use super::{LauncherLayout, LauncherPreferences};
    use iced::Size;

    fn approx_eq(left: f32, right: f32) {
        assert!((left - right).abs() < 0.001, "{left} != {right}");
    }

    #[test]
    fn reference_monitor_preserves_current_layout() {
        let layout = LauncherLayout::from_monitor_size(
            Some(Size::new(1920.0, 1080.0)),
            &LauncherPreferences::default(),
        );

        approx_eq(layout.panel_width, 825.0);
        approx_eq(layout.results_height, 300.0);
        approx_eq(layout.results_animation_speed, 0.25);
        assert_eq!(layout.top_margin, 120);
        assert_eq!(layout.collapsed_surface_height(), 103);
        assert_eq!(layout.expanded_surface_height(), 403);
    }

    #[test]
    fn smaller_monitors_scale_down_with_safe_floor() {
        let layout = LauncherLayout::from_monitor_size(
            Some(Size::new(1280.0, 720.0)),
            &LauncherPreferences::default(),
        );

        assert!(layout.panel_width < 825.0);
        assert!(layout.results_height < 300.0);
        assert!(layout.top_margin < 120);
        assert!(layout.panel_width >= 700.0);
    }

    #[test]
    fn preferences_override_scaled_defaults() {
        let preferences = LauncherPreferences {
            panel_width: Some(910.0),
            top_margin: Some(144.0),
            results_height: Some(360.0),
            animation_speed: Some(0.33),
        };

        let layout =
            LauncherLayout::from_monitor_size(Some(Size::new(1280.0, 720.0)), &preferences);

        approx_eq(layout.panel_width, 910.0);
        approx_eq(layout.results_height, 360.0);
        approx_eq(layout.results_animation_speed, 0.33);
        assert_eq!(layout.top_margin, 144);
    }
}
