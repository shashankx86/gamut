mod appearance;
mod layout;
mod shortcuts;
mod system;

pub use appearance::{
    AppearancePreferences, ThemeColors, ThemePreference, ThemeSchemeId, normalize_hex_color,
};
pub use layout::{LauncherPlacement, LauncherSize, LayoutPreferences, RadiusPreference};
pub use shortcuts::{ShortcutAction, ShortcutBinding, ShortcutPreferences};
pub use system::SystemPreferences;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppPreferences {
    pub appearance: AppearancePreferences,
    pub layout: LayoutPreferences,
    pub shortcuts: ShortcutPreferences,
    pub system: SystemPreferences,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            appearance: AppearancePreferences::default(),
            layout: LayoutPreferences::default(),
            shortcuts: ShortcutPreferences::default(),
            system: SystemPreferences::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ShortcutAction,
        ShortcutBinding, ShortcutPreferences, ThemePreference,
    };
    use std::str::FromStr;

    #[test]
    fn default_preferences_match_requested_defaults() {
        let preferences = AppPreferences::default();

        assert_eq!(preferences.appearance.theme, ThemePreference::System);
        assert_eq!(preferences.appearance.radius, RadiusPreference::Small);
        assert_eq!(preferences.layout.size, LauncherSize::Medium);
        assert_eq!(
            preferences.layout.placement,
            LauncherPlacement::RaisedCenter
        );
        assert_eq!(preferences.shortcuts, ShortcutPreferences::default());
        assert!(!preferences.system.start_at_login);
    }

    #[test]
    fn shortcut_binding_parses_modifiers_and_named_keys() {
        let binding = ShortcutBinding::from_str("Ctrl+Shift+ArrowDown").unwrap_or_else(|err| {
            panic!("expected shortcut to parse: {err}");
        });

        assert!(binding.ctrl);
        assert!(binding.shift);
        assert_eq!(binding.normalized_key(), "arrowdown");
    }

    #[test]
    fn shortcut_binding_formats_consistently() {
        let binding = ShortcutBinding::from_str("super+a").unwrap_or_else(|err| {
            panic!("expected shortcut to parse: {err}");
        });

        assert_eq!(binding.to_string(), "Super+A");
    }

    #[test]
    fn shortcut_binding_rejects_missing_key() {
        assert!(ShortcutBinding::from_str("Ctrl+Alt").is_err());
    }

    #[test]
    fn shortcut_action_parses_cli_name() {
        assert_eq!(
            ShortcutAction::from_str("expand-or-move-down").expect("action should parse"),
            ShortcutAction::ExpandOrMoveDown,
        );
    }
}
