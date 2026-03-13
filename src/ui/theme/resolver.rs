use super::color_scheme::shared_color_scheme;
use super::palette::{ResolvedAppearance, ThemePalette};
use crate::core::assets::{asset_theme, AssetTheme};
use crate::core::preferences::{AppearancePreferences, ThemePreference};
use iced::theme::Palette;
use iced::Theme;

pub(in crate::ui) fn resolve_theme(preferences: &AppearancePreferences) -> Theme {
    let resolved = resolve_palette(preferences);
    Theme::custom(theme_name(preferences.theme), resolved.palette)
}

pub(in crate::ui) fn resolve_appearance(preferences: &AppearancePreferences) -> ResolvedAppearance {
    resolve_palette(preferences).appearance
}

pub(in crate::ui) fn resolve_asset_theme(preferences: &AppearancePreferences) -> AssetTheme {
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

fn theme_name(preference: ThemePreference) -> &'static str {
    match preference {
        ThemePreference::Light => "Gamut Light",
        ThemePreference::Dark => "Gamut Dark",
        ThemePreference::System => "Gamut System",
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_appearance, resolve_theme};
    use crate::core::preferences::{
        AppearancePreferences, ThemeColors, ThemePreference, ThemeSchemeId,
    };

    #[test]
    fn invalid_scheme_falls_back_to_default_dark_palette() {
        let mut preferences = AppearancePreferences::default();
        preferences.theme = ThemePreference::Dark;
        *preferences.scheme_mut(ThemeSchemeId::Dark) =
            ThemeColors::new("invalid", "#FFFFFF", "#3366FF");

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
