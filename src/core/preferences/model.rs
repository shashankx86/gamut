use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearancePreferences {
    pub theme: ThemePreference,
    pub custom_theme: CustomThemeColors,
    pub radius: RadiusPreference,
    pub custom_radius: f32,
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            custom_theme: CustomThemeColors::default(),
            radius: RadiusPreference::Medium,
            custom_radius: 10.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
    Custom,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomThemeColors {
    pub background: String,
    pub text: String,
    pub accent: String,
}

impl Default for CustomThemeColors {
    fn default() -> Self {
        Self {
            background: "#151516".to_string(),
            text: "#EBEDF2".to_string(),
            accent: "#5E8BFF".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadiusPreference {
    Small,
    Medium,
    Large,
    Custom,
}

impl Default for RadiusPreference {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutPreferences {
    pub size: LauncherSize,
    pub placement: LauncherPlacement,
    pub custom_top_margin: f32,
}

impl Default for LayoutPreferences {
    fn default() -> Self {
        Self {
            size: LauncherSize::Medium,
            placement: LauncherPlacement::RaisedCenter,
            custom_top_margin: 120.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Default for LauncherSize {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherPlacement {
    Center,
    RaisedCenter,
    Custom,
}

impl Default for LauncherPlacement {
    fn default() -> Self {
        Self::RaisedCenter
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemPreferences {
    pub start_at_login: bool,
}

impl Default for SystemPreferences {
    fn default() -> Self {
        Self {
            start_at_login: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortcutPreferences {
    pub launch_selected: ShortcutBinding,
    pub expand_or_move_down: ShortcutBinding,
    pub move_up: ShortcutBinding,
    pub close_launcher: ShortcutBinding,
}

impl Default for ShortcutPreferences {
    fn default() -> Self {
        Self {
            launch_selected: ShortcutBinding::named("Enter"),
            expand_or_move_down: ShortcutBinding::named("ArrowDown"),
            move_up: ShortcutBinding::named("ArrowUp"),
            close_launcher: ShortcutBinding::named("Escape"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub key: String,
}

impl ShortcutBinding {
    pub fn named(key: &str) -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
            key: key.to_string(),
        }
    }

    pub fn normalized_key(&self) -> String {
        normalize_key_name(&self.key)
    }
}

impl fmt::Display for ShortcutBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.super_key {
            parts.push("Super".to_string());
        }

        parts.push(pretty_key_name(&self.key));
        write!(f, "{}", parts.join("+"))
    }
}

impl FromStr for ShortcutBinding {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut binding = Self::named("");
        let mut key: Option<String> = None;

        for part in value
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => binding.ctrl = true,
                "alt" => binding.alt = true,
                "shift" => binding.shift = true,
                "super" | "meta" | "cmd" | "command" | "win" => binding.super_key = true,
                _ => {
                    if key.is_some() {
                        return Err("shortcut can only contain one key".to_string());
                    }

                    let normalized = normalize_key_name(part);
                    if normalized.is_empty() {
                        return Err("shortcut key cannot be empty".to_string());
                    }

                    key = Some(normalized);
                }
            }
        }

        binding.key = key.ok_or_else(|| "shortcut must include a key".to_string())?;
        Ok(binding)
    }
}

fn pretty_key_name(key: &str) -> String {
    match normalize_key_name(key).as_str() {
        "arrowup" => "ArrowUp".to_string(),
        "arrowdown" => "ArrowDown".to_string(),
        "arrowleft" => "ArrowLeft".to_string(),
        "arrowright" => "ArrowRight".to_string(),
        "escape" => "Escape".to_string(),
        "enter" => "Enter".to_string(),
        "space" => "Space".to_string(),
        normalized if normalized.len() == 1 => normalized.to_ascii_uppercase(),
        normalized => {
            let mut chars = normalized.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn normalize_key_name(key: &str) -> String {
    key.trim().to_ascii_lowercase().replace([' ', '_', '-'], "")
}

#[cfg(test)]
mod tests {
    use super::{AppPreferences, ShortcutBinding, ShortcutPreferences};
    use std::str::FromStr;

    #[test]
    fn default_preferences_preserve_current_behavior() {
        let preferences = AppPreferences::default();

        assert_eq!(
            preferences.appearance.theme as u8,
            super::ThemePreference::System as u8
        );
        assert_eq!(
            preferences.appearance.radius as u8,
            super::RadiusPreference::Medium as u8
        );
        assert_eq!(
            preferences.layout.placement as u8,
            super::LauncherPlacement::RaisedCenter as u8,
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
}
