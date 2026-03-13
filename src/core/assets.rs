use crate::core::preferences::{AppearancePreferences, ThemeSchemeId};
use dark_light::Mode as SystemThemeMode;

const LAUNCHER_LOGO_DARK: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/icons/gamut-full-transparent-dark.svg"
));
const LAUNCHER_LOGO_LIGHT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/icons/gamut-full-transparent-light.svg"
));
const TRAY_ICON_DARK: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/icons/gamut-transparent-dark.svg"
));
const TRAY_ICON_LIGHT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/icons/gamut-transparent-light.svg"
));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AssetTheme {
    Light,
    Dark,
}

pub(crate) fn asset_theme(preferences: &AppearancePreferences) -> AssetTheme {
    asset_theme_for_mode(
        preferences,
        dark_light::detect().unwrap_or(SystemThemeMode::Unspecified),
    )
}

pub(crate) fn asset_theme_for_mode(
    preferences: &AppearancePreferences,
    system_mode: SystemThemeMode,
) -> AssetTheme {
    match preferences.resolved_scheme_for_mode(system_mode) {
        ThemeSchemeId::Light => AssetTheme::Light,
        ThemeSchemeId::Dark => AssetTheme::Dark,
    }
}

pub(crate) const fn launcher_logo_svg(theme: AssetTheme) -> &'static [u8] {
    match theme {
        AssetTheme::Light => LAUNCHER_LOGO_LIGHT,
        AssetTheme::Dark => LAUNCHER_LOGO_DARK,
    }
}

pub(crate) const fn tray_icon_svg(theme: AssetTheme) -> &'static [u8] {
    match theme {
        AssetTheme::Light => TRAY_ICON_LIGHT,
        AssetTheme::Dark => TRAY_ICON_DARK,
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_theme_for_mode, launcher_logo_svg, tray_icon_svg, AssetTheme};
    use crate::core::preferences::{AppearancePreferences, ThemePreference};
    use dark_light::Mode as SystemThemeMode;

    #[test]
    fn explicit_theme_preferences_override_system_mode() {
        let mut preferences = AppearancePreferences::default();
        preferences.theme = ThemePreference::Light;
        assert_eq!(
            asset_theme_for_mode(&preferences, SystemThemeMode::Dark),
            AssetTheme::Light
        );

        preferences.theme = ThemePreference::Dark;
        assert_eq!(
            asset_theme_for_mode(&preferences, SystemThemeMode::Light),
            AssetTheme::Dark
        );
    }

    #[test]
    fn system_theme_preferences_follow_detected_mode() {
        let preferences = AppearancePreferences::default();

        assert_eq!(
            asset_theme_for_mode(&preferences, SystemThemeMode::Light),
            AssetTheme::Light
        );
        assert_eq!(
            asset_theme_for_mode(&preferences, SystemThemeMode::Dark),
            AssetTheme::Dark
        );
        assert_eq!(
            asset_theme_for_mode(&preferences, SystemThemeMode::Unspecified),
            AssetTheme::Dark
        );
    }

    #[test]
    fn asset_payloads_exist_for_each_theme_variant() {
        assert_ne!(
            launcher_logo_svg(AssetTheme::Light),
            launcher_logo_svg(AssetTheme::Dark)
        );
        assert_ne!(
            tray_icon_svg(AssetTheme::Light),
            tray_icon_svg(AssetTheme::Dark)
        );
        assert!(!launcher_logo_svg(AssetTheme::Light).is_empty());
        assert!(!tray_icon_svg(AssetTheme::Dark).is_empty());
    }
}
