use super::ConfigKey;
use super::display::trim_float;
use super::parsing::{parse_bool, parse_non_negative_f32};
use crate::core::preferences::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ShortcutBinding,
    ThemeColors, ThemePreference,
};
use crate::core::theme::default_custom_theme_colors;

impl ConfigKey {
    pub fn value(self, preferences: &AppPreferences) -> String {
        match self {
            Self::AppearanceTheme => preferences.appearance.theme.to_string(),
            Self::AppearanceRadius => preferences.appearance.radius.to_string(),
            Self::AppearanceThemeBackground => active_theme_colors(preferences).background,
            Self::AppearanceThemeText => active_theme_colors(preferences).text,
            Self::AppearanceThemeAccent => active_theme_colors(preferences).accent,
            Self::LayoutSize => preferences.layout.size.to_string(),
            Self::LayoutPlacement => preferences.layout.placement.to_string(),
            Self::LayoutCustomTopMargin => trim_float(preferences.layout.custom_top_margin),
            Self::Shortcut(action) => preferences.shortcuts.binding(action).to_string(),
            Self::SystemStartAtLogin => preferences.system.start_at_login.to_string(),
        }
    }

    pub fn set_value(self, preferences: &mut AppPreferences, value: &str) -> Result<(), String> {
        match self {
            Self::AppearanceTheme => {
                preferences.appearance.theme = value.parse::<ThemePreference>()?;

                if let ThemePreference::Custom(name) = preferences.appearance.theme.clone() {
                    preferences.appearance.upsert_custom_theme(&name)?;
                }
            }
            Self::AppearanceRadius => {
                preferences.appearance.radius = value.parse::<RadiusPreference>()?;
            }
            Self::AppearanceThemeBackground => {
                selected_custom_theme_mut(preferences)?.background = parse_color(value)?;
            }
            Self::AppearanceThemeText => {
                selected_custom_theme_mut(preferences)?.text = parse_color(value)?;
            }
            Self::AppearanceThemeAccent => {
                selected_custom_theme_mut(preferences)?.accent = parse_color(value)?;
            }
            Self::LayoutSize => {
                preferences.layout.size = value.parse::<LauncherSize>()?;
            }
            Self::LayoutPlacement => {
                preferences.layout.placement = value.parse::<LauncherPlacement>()?;
            }
            Self::LayoutCustomTopMargin => {
                preferences.layout.custom_top_margin =
                    parse_non_negative_f32(value, "layout.custom_top_margin")?;
            }
            Self::Shortcut(action) => {
                preferences
                    .shortcuts
                    .update_binding(action, value.parse::<ShortcutBinding>()?)?;
            }
            Self::SystemStartAtLogin => {
                preferences.system.start_at_login = parse_bool(value)?;
            }
        }

        Ok(())
    }

    pub fn reset_value(self, preferences: &mut AppPreferences) {
        let defaults = AppPreferences::default();

        match self {
            Self::AppearanceTheme => preferences.appearance.theme = defaults.appearance.theme,
            Self::AppearanceRadius => preferences.appearance.radius = defaults.appearance.radius,
            Self::AppearanceThemeBackground => {
                if let Ok(theme) = selected_custom_theme_mut(preferences) {
                    theme.background = default_custom_theme_colors().background;
                }
            }
            Self::AppearanceThemeText => {
                if let Ok(theme) = selected_custom_theme_mut(preferences) {
                    theme.text = default_custom_theme_colors().text;
                }
            }
            Self::AppearanceThemeAccent => {
                if let Ok(theme) = selected_custom_theme_mut(preferences) {
                    theme.accent = default_custom_theme_colors().accent;
                }
            }
            Self::LayoutSize => preferences.layout.size = defaults.layout.size,
            Self::LayoutPlacement => preferences.layout.placement = defaults.layout.placement,
            Self::LayoutCustomTopMargin => {
                preferences.layout.custom_top_margin = defaults.layout.custom_top_margin;
            }
            Self::Shortcut(action) => {
                let binding = defaults.shortcuts.binding(action).clone();
                preferences
                    .shortcuts
                    .update_binding(action, binding)
                    .expect("default shortcut binding should apply");
            }
            Self::SystemStartAtLogin => {
                preferences.system.start_at_login = defaults.system.start_at_login;
            }
        }
    }
}

fn active_theme_colors(preferences: &AppPreferences) -> ThemeColors {
    preferences.appearance.resolved_theme().colors
}

fn selected_custom_theme_mut(preferences: &mut AppPreferences) -> Result<&mut ThemeColors, String> {
    let ThemePreference::Custom(name) = preferences.appearance.theme.clone() else {
        return Err(
            "appearance.theme colors can only be changed for a custom theme; set appearance.theme to a name like `orange` first"
                .to_string(),
        );
    };

    preferences.appearance.upsert_custom_theme(&name)
}

fn parse_color(value: &str) -> Result<String, String> {
    crate::core::preferences::normalize_hex_color(value)
        .ok_or_else(|| "expected a 6 or 8 digit hex color like #151516 or #151516FF".to_string())
}

#[cfg(test)]
mod tests {
    use super::ConfigKey;
    use crate::core::preferences::AppPreferences;
    use std::str::FromStr;

    #[test]
    fn config_key_updates_shortcuts() {
        let mut preferences = AppPreferences::default();

        ConfigKey::from_str("shortcuts.move_up")
            .expect("key should parse")
            .set_value(&mut preferences, "[17, 75]")
            .expect("shortcut should update");

        assert_eq!(preferences.shortcuts.move_up.to_string(), "[17, 75]");
    }

    #[test]
    fn setting_custom_theme_creates_theme_entry() {
        let mut preferences = AppPreferences::default();

        ConfigKey::AppearanceTheme
            .set_value(&mut preferences, "orange")
            .expect("theme should update");

        assert!(preferences.appearance.themes.contains_key("orange"));
    }
}
