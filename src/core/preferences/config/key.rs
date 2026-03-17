use super::ShortcutAction;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKey {
    AppearanceTheme,
    AppearanceRadius,
    AppearanceThemeBackground,
    AppearanceThemeText,
    AppearanceThemeAccent,
    LayoutSize,
    LayoutPlacement,
    LayoutCustomTopMargin,
    Shortcut(ShortcutAction),
    SystemStartAtLogin,
}

impl ConfigKey {
    pub const ALL: [Self; 14] = [
        Self::AppearanceTheme,
        Self::AppearanceRadius,
        Self::AppearanceThemeBackground,
        Self::AppearanceThemeText,
        Self::AppearanceThemeAccent,
        Self::LayoutSize,
        Self::LayoutPlacement,
        Self::LayoutCustomTopMargin,
        Self::Shortcut(ShortcutAction::LaunchSelected),
        Self::Shortcut(ShortcutAction::Expand),
        Self::Shortcut(ShortcutAction::MoveDown),
        Self::Shortcut(ShortcutAction::MoveUp),
        Self::Shortcut(ShortcutAction::CloseLauncher),
        Self::SystemStartAtLogin,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AppearanceTheme => "appearance.theme",
            Self::AppearanceRadius => "appearance.radius",
            Self::AppearanceThemeBackground => "appearance.theme.background",
            Self::AppearanceThemeText => "appearance.theme.text",
            Self::AppearanceThemeAccent => "appearance.theme.accent",
            Self::LayoutSize => "layout.size",
            Self::LayoutPlacement => "layout.placement",
            Self::LayoutCustomTopMargin => "layout.custom_top_margin",
            Self::Shortcut(action) => action.config_key(),
            Self::SystemStartAtLogin => "system.start_at_login",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::AppearanceTheme => {
                "Launcher theme: system, light, dark, or any custom theme name"
            }
            Self::AppearanceRadius => "Corner radius preset: none, small, medium, large",
            Self::AppearanceThemeBackground => "Active custom theme background hex color",
            Self::AppearanceThemeText => "Active custom theme text hex color",
            Self::AppearanceThemeAccent => "Active custom theme accent hex color",
            Self::LayoutSize => "Launcher size: small, medium, large",
            Self::LayoutPlacement => "Launcher placement: center, raised_center, custom",
            Self::LayoutCustomTopMargin => "Launcher top margin in pixels",
            Self::Shortcut(action) => action.description(),
            Self::SystemStartAtLogin => "Start the daemon automatically when the session starts",
        }
    }

    pub const fn section(self) -> &'static str {
        match self {
            Self::AppearanceTheme
            | Self::AppearanceRadius
            | Self::AppearanceThemeBackground
            | Self::AppearanceThemeText
            | Self::AppearanceThemeAccent => "appearance",
            Self::LayoutSize | Self::LayoutPlacement | Self::LayoutCustomTopMargin => "layout",
            Self::Shortcut(_) => "shortcuts",
            Self::SystemStartAtLogin => "system",
        }
    }
}

impl fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ConfigKey {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match normalize(value).as_str() {
            "appearancetheme" => Ok(Self::AppearanceTheme),
            "appearanceradius" => Ok(Self::AppearanceRadius),
            "appearancethemebackground" => Ok(Self::AppearanceThemeBackground),
            "appearancethemetext" => Ok(Self::AppearanceThemeText),
            "appearancethemeaccent" => Ok(Self::AppearanceThemeAccent),
            "layoutsize" => Ok(Self::LayoutSize),
            "layoutplacement" => Ok(Self::LayoutPlacement),
            "layoutcustomtopmargin" => Ok(Self::LayoutCustomTopMargin),
            "shortcutslaunchselected" => Ok(Self::Shortcut(ShortcutAction::LaunchSelected)),
            "shortcutsexpand" => Ok(Self::Shortcut(ShortcutAction::Expand)),
            "shortcutsexpandormovedown" => Ok(Self::Shortcut(ShortcutAction::MoveDown)),
            "shortcutsmovedown" => Ok(Self::Shortcut(ShortcutAction::MoveDown)),
            "shortcutsmoveup" => Ok(Self::Shortcut(ShortcutAction::MoveUp)),
            "shortcutscloselauncher" => Ok(Self::Shortcut(ShortcutAction::CloseLauncher)),
            "systemstartatlogin" => Ok(Self::SystemStartAtLogin),
            _ => Err(format!("unknown config key `{value}`")),
        }
    }
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '_', '-', '.'], "")
}

#[cfg(test)]
mod tests {
    use super::ConfigKey;
    use crate::core::preferences::ShortcutAction;
    use std::str::FromStr;

    #[test]
    fn config_keys_accept_dot_notation() {
        assert_eq!(
            ConfigKey::from_str("appearance.theme.accent").expect("key should parse"),
            ConfigKey::AppearanceThemeAccent,
        );
        assert_eq!(
            ConfigKey::from_str("shortcuts.close_launcher").expect("key should parse"),
            ConfigKey::Shortcut(ShortcutAction::CloseLauncher),
        );
    }
}
