use super::color_scheme::shared_color_scheme;
use super::palette::{ResolvedAppearance, ThemePalette};
use crate::core::assets::{asset_theme, AssetTheme};
use crate::core::preferences::AppearancePreferences;
use iced::theme::Palette;
use iced::Theme;

pub(crate) fn resolve_theme(preferences: &AppearancePreferences) -> Theme {
    let resolved = resolve_palette(preferences);
    Theme::custom(theme_name(preferences), resolved.palette)
}

pub(crate) fn resolve_appearance(preferences: &AppearancePreferences) -> ResolvedAppearance {
    resolve_palette(preferences).appearance
}

pub(crate) fn resolve_asset_theme(preferences: &AppearancePreferences) -> AssetTheme {
    asset_theme(preferences)
}

fn resolve_palette(preferences: &AppearancePreferences) -> ThemePalette {
    let appearance = shared_color_scheme(preferences);
    let palette = Palette {
        background: appearance.panel_background,
        text: appearance.primary_text,
        primary: appearance.accent,
        success: appearance.accent,
        warning: appearance.accent,
        danger: appearance.accent,
    };

    ThemePalette {
        palette,
        appearance,
    }
}

fn theme_name(preferences: &AppearancePreferences) -> String {
    format!("Gamut {}", preferences.resolved_theme().name)
}

#[cfg(test)]
mod tests {
    use super::{resolve_appearance, resolve_theme};
    use crate::core::preferences::{AppearancePreferences, ThemePreference};

    #[test]
    fn invalid_scheme_falls_back_to_default_dark_palette() {
        let mut preferences = AppearancePreferences::default();
        preferences.theme = ThemePreference::Custom("night".to_string());
        preferences
            .upsert_custom_theme("night")
            .expect("theme entry should exist")
            .background = "invalid".to_string();
        preferences
            .upsert_custom_theme("night")
            .expect("theme entry should exist")
            .text = "#FFFFFF".to_string();
        preferences
            .upsert_custom_theme("night")
            .expect("theme entry should exist")
            .accent = "#3366FF".to_string();

        let theme = resolve_theme(&preferences);
        assert_eq!(
            theme.palette().background,
            iced::Color::from_rgb8(21, 21, 22)
        );
    }

    #[test]
    fn light_scheme_uses_accent_tinted_selected_rows() {
        let mut preferences = AppearancePreferences::default();
        preferences.theme = ThemePreference::Light;

        let appearance = resolve_appearance(&preferences);

        // The selected row should have a visible blue/accent tint so it pops
        // against the white panel background.
        assert!(
            appearance.first_row_active.b > appearance.first_row_active.r,
            "selected row should have a blue tint for visibility"
        );
        // It should still be a subtle tint, not a saturated block of color.
        assert!(
            appearance.first_row_active.r > 0.8,
            "selected row should remain a light tint, not a saturated color"
        );
    }
}
