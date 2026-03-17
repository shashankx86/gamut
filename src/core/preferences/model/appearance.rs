use crate::core::theme::{
    classify_theme_colors, default_custom_theme_colors, default_theme_colors,
};
use dark_light::Mode as SystemThemeMode;
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use super::RadiusPreference;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AppearancePreferences {
    pub theme: ThemePreference,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub themes: BTreeMap<String, ThemeColors>,
    pub radius: RadiusPreference,
}

impl AppearancePreferences {
    pub fn upsert_custom_theme(&mut self, name: &str) -> Result<&mut ThemeColors, String> {
        let key = normalize_custom_theme_name(name)?;
        Ok(self
            .themes
            .entry(key)
            .or_insert_with(default_custom_theme_colors))
    }

    pub(crate) fn resolved_theme(&self) -> ResolvedTheme {
        self.resolved_theme_for_mode(dark_light::detect().unwrap_or(SystemThemeMode::Unspecified))
    }

    pub(crate) fn resolved_theme_for_mode(&self, system_mode: SystemThemeMode) -> ResolvedTheme {
        match &self.theme {
            ThemePreference::Light => resolve_builtin_theme(ThemeSchemeId::Light),
            ThemePreference::Dark => resolve_builtin_theme(ThemeSchemeId::Dark),
            ThemePreference::System => resolve_builtin_theme(system_theme(system_mode)),
            ThemePreference::Custom(name) => self
                .themes
                .get(name)
                .map(|colors| ResolvedTheme {
                    name: name.clone(),
                    colors: colors.clone(),
                    variant: classify_theme_colors(colors),
                })
                .unwrap_or_else(|| resolve_builtin_theme(system_theme(system_mode))),
        }
    }
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            themes: BTreeMap::new(),
            radius: RadiusPreference::Small,
        }
    }
}

impl<'de> Deserialize<'de> for AppearancePreferences {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(default)]
        struct AppearancePreferencesWire {
            theme: ThemePreference,
            themes: BTreeMap<String, ThemeColors>,
            radius: RadiusPreference,
        }

        impl Default for AppearancePreferencesWire {
            fn default() -> Self {
                let defaults = AppearancePreferences::default();
                Self {
                    theme: defaults.theme,
                    themes: defaults.themes,
                    radius: defaults.radius,
                }
            }
        }

        let wire = AppearancePreferencesWire::deserialize(deserializer)?;

        Ok(Self {
            theme: wire.theme,
            themes: normalize_custom_themes(wire.themes),
            radius: wire.radius,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
    Custom(String),
}

impl ThemePreference {
    pub fn as_str(&self) -> &str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Custom(name) => name,
        }
    }
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self::System
    }
}

impl Serialize for ThemePreference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl fmt::Display for ThemePreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ThemePreference {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "system" => Ok(Self::System),
            _ => Ok(Self::Custom(normalize_custom_theme_name(value)?)),
        }
    }
}

impl<'de> Deserialize<'de> for ThemePreference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Self::from_str(&value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedTheme {
    pub(crate) name: String,
    pub(crate) colors: ThemeColors,
    pub(crate) variant: ThemeSchemeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeSchemeId {
    Light,
    Dark,
}

impl ThemeSchemeId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

impl fmt::Display for ThemeSchemeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub text: String,
    pub accent: String,
}

impl ThemeColors {
    pub fn new(background: &str, text: &str, accent: &str) -> Self {
        Self {
            background: background.to_string(),
            text: text.to_string(),
            accent: accent.to_string(),
        }
    }
}

pub fn normalize_custom_theme_name(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase().replace(' ', "-");

    if normalized.is_empty() {
        return Err("theme name cannot be empty".to_string());
    }

    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'))
    {
        return Err("theme name must use only letters, numbers, '-' or '_'".to_string());
    }

    if matches!(normalized.as_str(), "system" | "light" | "dark") {
        return Err(format!("theme name `{normalized}` is reserved"));
    }

    Ok(normalized)
}

pub fn normalize_hex_color(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('#');

    match trimmed.len() {
        6 | 8 if trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) => {
            Some(format!("#{}", trimmed.to_ascii_uppercase()))
        }
        _ => None,
    }
}

fn normalize_custom_themes(themes: BTreeMap<String, ThemeColors>) -> BTreeMap<String, ThemeColors> {
    themes
        .into_iter()
        .filter_map(|(name, colors)| {
            normalize_custom_theme_name(&name)
                .ok()
                .map(|key| (key, colors))
        })
        .collect()
}

fn resolve_builtin_theme(scheme: ThemeSchemeId) -> ResolvedTheme {
    ResolvedTheme {
        name: scheme.as_str().to_string(),
        colors: default_theme_colors(scheme),
        variant: scheme,
    }
}

fn system_theme(mode: SystemThemeMode) -> ThemeSchemeId {
    match mode {
        SystemThemeMode::Light => ThemeSchemeId::Light,
        _ => ThemeSchemeId::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_custom_theme_name, normalize_hex_color, AppearancePreferences, ThemeColors,
        ThemePreference, ThemeSchemeId,
    };
    use dark_light::Mode as SystemThemeMode;
    use std::str::FromStr;

    #[test]
    fn normalizes_hex_color_values() {
        assert_eq!(normalize_hex_color("a1b2c3"), Some("#A1B2C3".to_string()));
        assert_eq!(
            normalize_hex_color("#abcdef12"),
            Some("#ABCDEF12".to_string())
        );
        assert_eq!(normalize_hex_color("invalid"), None);
    }

    #[test]
    fn custom_theme_names_are_normalized() {
        assert_eq!(
            normalize_custom_theme_name(" Orange Theme ").expect("theme name should normalize"),
            "orange-theme"
        );
        assert!(normalize_custom_theme_name("light").is_err());
    }

    #[test]
    fn custom_theme_preference_parses_named_themes() {
        assert_eq!(
            ThemePreference::from_str("orange").expect("theme should parse"),
            ThemePreference::Custom("orange".to_string())
        );
    }

    #[test]
    fn resolved_custom_theme_uses_custom_colors() {
        let appearance: AppearancePreferences = toml::from_str(
            r##"
theme = "orange"
radius = "small"

[themes.orange]
background = "#FFF4E8"
text = "#2D1606"
accent = "#FF7A00"
"##,
        )
        .expect("custom themes should deserialize");

        let resolved = appearance.resolved_theme_for_mode(SystemThemeMode::Dark);

        assert_eq!(
            resolved.colors,
            ThemeColors::new("#FFF4E8", "#2D1606", "#FF7A00"),
        );
        assert_eq!(resolved.variant, ThemeSchemeId::Light);
    }
}
