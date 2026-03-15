use super::ShortcutAction;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKey {
    AppearanceTheme,
    AppearanceRadius,
    AppearanceCustomRadius,
    AppearanceLightBackground,
    AppearanceLightText,
    AppearanceLightAccent,
    AppearanceDarkBackground,
    AppearanceDarkText,
    AppearanceDarkAccent,
    LayoutSize,
    LayoutPlacement,
    LayoutCustomTopMargin,
    Shortcut(ShortcutAction),
    SystemStartAtLogin,
}

impl ConfigKey {
    pub const ALL: [Self; 17] = [
        Self::AppearanceTheme,
        Self::AppearanceRadius,
        Self::AppearanceCustomRadius,
        Self::AppearanceLightBackground,
        Self::AppearanceLightText,
        Self::AppearanceLightAccent,
        Self::AppearanceDarkBackground,
        Self::AppearanceDarkText,
        Self::AppearanceDarkAccent,
        Self::LayoutSize,
        Self::LayoutPlacement,
        Self::LayoutCustomTopMargin,
        Self::Shortcut(ShortcutAction::LaunchSelected),
        Self::Shortcut(ShortcutAction::ExpandOrMoveDown),
        Self::Shortcut(ShortcutAction::MoveUp),
        Self::Shortcut(ShortcutAction::CloseLauncher),
        Self::SystemStartAtLogin,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AppearanceTheme => "appearance.theme",
            Self::AppearanceRadius => "appearance.radius",
            Self::AppearanceCustomRadius => "appearance.custom_radius",
            Self::AppearanceLightBackground => "appearance.schemes.light.background",
            Self::AppearanceLightText => "appearance.schemes.light.text",
            Self::AppearanceLightAccent => "appearance.schemes.light.accent",
            Self::AppearanceDarkBackground => "appearance.schemes.dark.background",
            Self::AppearanceDarkText => "appearance.schemes.dark.text",
            Self::AppearanceDarkAccent => "appearance.schemes.dark.accent",
            Self::LayoutSize => "layout.size",
            Self::LayoutPlacement => "layout.placement",
            Self::LayoutCustomTopMargin => "layout.custom_top_margin",
            Self::Shortcut(action) => action.config_key(),
            Self::SystemStartAtLogin => "system.start_at_login",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::AppearanceTheme => "Launcher theme: system, light, dark",
            Self::AppearanceRadius => "Corner radius preset: small, medium, large, custom",
            Self::AppearanceCustomRadius => "Custom corner radius in pixels",
            Self::AppearanceLightBackground => "Light theme background hex color",
            Self::AppearanceLightText => "Light theme text hex color",
            Self::AppearanceLightAccent => "Light theme accent hex color",
            Self::AppearanceDarkBackground => "Dark theme background hex color",
            Self::AppearanceDarkText => "Dark theme text hex color",
            Self::AppearanceDarkAccent => "Dark theme accent hex color",
            Self::LayoutSize => "Launcher size: small, medium, large, extra_large",
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
            | Self::AppearanceCustomRadius
            | Self::AppearanceLightBackground
            | Self::AppearanceLightText
            | Self::AppearanceLightAccent
            | Self::AppearanceDarkBackground
            | Self::AppearanceDarkText
            | Self::AppearanceDarkAccent => "appearance",
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
            "appearancecustomradius" => Ok(Self::AppearanceCustomRadius),
            "appearanceschemeslightbackground" => Ok(Self::AppearanceLightBackground),
            "appearanceschemeslighttext" => Ok(Self::AppearanceLightText),
            "appearanceschemeslightaccent" => Ok(Self::AppearanceLightAccent),
            "appearanceschemesdarkbackground" => Ok(Self::AppearanceDarkBackground),
            "appearanceschemesdarktext" => Ok(Self::AppearanceDarkText),
            "appearanceschemesdarkaccent" => Ok(Self::AppearanceDarkAccent),
            "layoutsize" => Ok(Self::LayoutSize),
            "layoutplacement" => Ok(Self::LayoutPlacement),
            "layoutcustomtopmargin" => Ok(Self::LayoutCustomTopMargin),
            "shortcutslaunchselected" => Ok(Self::Shortcut(ShortcutAction::LaunchSelected)),
            "shortcutsexpandormovedown" => Ok(Self::Shortcut(ShortcutAction::ExpandOrMoveDown)),
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
            ConfigKey::from_str("appearance.schemes.dark.accent").expect("key should parse"),
            ConfigKey::AppearanceDarkAccent,
        );
        assert_eq!(
            ConfigKey::from_str("shortcuts.close_launcher").expect("key should parse"),
            ConfigKey::Shortcut(ShortcutAction::CloseLauncher),
        );
    }
}
