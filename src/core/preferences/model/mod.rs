mod appearance;
mod layout;
mod shortcuts;
mod system;

pub use appearance::{
    AppearancePreferences, ThemeColors, ThemePreference, ThemeSchemeId, normalize_hex_color,
};
pub use layout::{LauncherPlacement, LauncherSize, LayoutPreferences, RadiusPreference};
pub use shortcuts::{
    ShortcutAction, ShortcutBinding, ShortcutPreferences, VK_BACKSPACE, VK_DELETE, VK_DOWN, VK_END,
    VK_ENTER, VK_ESCAPE, VK_F1, VK_HOME, VK_INSERT, VK_LEFT, VK_NUMPAD_0, VK_PAGE_DOWN, VK_PAGE_UP,
    VK_RIGHT, VK_TAB, VK_UP,
};
pub(crate) use shortcuts::{virtual_keycode_from_ascii_char, virtual_keycode_from_token};
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
        ShortcutBinding, ShortcutPreferences, ThemePreference, VK_UP,
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
        let binding = ShortcutBinding::from_str("[17, 16, 40]").unwrap_or_else(|err| {
            panic!("expected shortcut to parse: {err}");
        });

        assert!(binding.ctrl);
        assert!(binding.shift);
        assert_eq!(binding.key_codes, vec![40]);
    }

    #[test]
    fn shortcut_binding_formats_consistently() {
        let binding = ShortcutBinding::from_str("[91, 65]").unwrap_or_else(|err| {
            panic!("expected shortcut to parse: {err}");
        });

        assert_eq!(binding.to_string(), "[91, 65]");
    }

    #[test]
    fn shortcut_binding_rejects_missing_key() {
        assert!(ShortcutBinding::from_str("[17, 18]").is_err());
    }

    #[test]
    fn default_move_up_uses_up_keycode() {
        let preferences = ShortcutPreferences::default();
        assert_eq!(preferences.move_up.key_codes, vec![VK_UP]);
    }

    #[test]
    fn shortcut_action_parses_cli_name() {
        assert_eq!(
            ShortcutAction::from_str("move-down").expect("action should parse"),
            ShortcutAction::MoveDown,
        );
    }
}
