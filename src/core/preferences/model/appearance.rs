use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AppearancePreferences {
    pub theme: ThemePreference,
    pub schemes: ThemeSchemes,
    pub radius: RadiusPreference,
    pub custom_radius: f32,
}

impl AppearancePreferences {
    pub fn scheme(&self, scheme: ThemeSchemeId) -> &ThemeColors {
        self.schemes.scheme(scheme)
    }

    pub fn scheme_mut(&mut self, scheme: ThemeSchemeId) -> &mut ThemeColors {
        self.schemes.scheme_mut(scheme)
    }
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            schemes: ThemeSchemes::default(),
            radius: RadiusPreference::Small,
            custom_radius: 10.0,
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
            schemes: ThemeSchemes,
            custom_theme: Option<ThemeColors>,
            radius: RadiusPreference,
            custom_radius: f32,
        }

        impl Default for AppearancePreferencesWire {
            fn default() -> Self {
                let defaults = AppearancePreferences::default();
                Self {
                    theme: defaults.theme,
                    schemes: defaults.schemes,
                    custom_theme: None,
                    radius: defaults.radius,
                    custom_radius: defaults.custom_radius,
                }
            }
        }

        let wire = AppearancePreferencesWire::deserialize(deserializer)?;
        let mut schemes = wire.schemes;

        if let Some(custom_theme) = wire.custom_theme {
            schemes.dark = custom_theme;
        }

        Ok(Self {
            theme: wire.theme,
            schemes,
            radius: wire.radius,
            custom_radius: wire.custom_radius,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
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
        serializer.serialize_str(match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        })
    }
}

impl<'de> Deserialize<'de> for ThemePreference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        match value.as_str() {
            "light" => Ok(Self::Light),
            "dark" | "custom" => Ok(Self::Dark),
            "system" => Ok(Self::System),
            _ => Err(de::Error::unknown_variant(
                &value,
                &["light", "dark", "system", "custom"],
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeSchemeId {
    Light,
    Dark,
}

impl ThemeSchemeId {
    pub const ALL: [Self; 2] = [Self::Light, Self::Dark];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeSchemes {
    pub light: ThemeColors,
    pub dark: ThemeColors,
}

impl ThemeSchemes {
    pub fn scheme(&self, scheme: ThemeSchemeId) -> &ThemeColors {
        match scheme {
            ThemeSchemeId::Light => &self.light,
            ThemeSchemeId::Dark => &self.dark,
        }
    }

    pub fn scheme_mut(&mut self, scheme: ThemeSchemeId) -> &mut ThemeColors {
        match scheme {
            ThemeSchemeId::Light => &mut self.light,
            ThemeSchemeId::Dark => &mut self.dark,
        }
    }
}

impl Default for ThemeSchemes {
    fn default() -> Self {
        Self {
            light: default_light_theme_colors(),
            dark: default_dark_theme_colors(),
        }
    }
}

pub fn default_light_theme_colors() -> ThemeColors {
    ThemeColors::new("#F8F9FB", "#1C222A", "#416EF5")
}

pub fn default_dark_theme_colors() -> ThemeColors {
    ThemeColors::new("#151516", "#EBEDF2", "#5E8BFF")
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

use super::RadiusPreference;

#[cfg(test)]
mod tests {
    use super::{
        AppearancePreferences, ThemeColors, ThemePreference, ThemeSchemeId, normalize_hex_color,
    };

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
    fn legacy_custom_theme_migrates_to_dark_scheme() {
        let appearance: AppearancePreferences = toml::from_str(
            r##"
theme = "custom"
radius = "small"
custom_radius = 10.0

[custom_theme]
background = "#112233"
text = "#EEF0F3"
accent = "#5588FF"
"##,
        )
        .expect("legacy appearance preferences should deserialize");

        assert_eq!(appearance.theme, ThemePreference::Dark);
        assert_eq!(
            appearance.scheme(ThemeSchemeId::Dark),
            &ThemeColors::new("#112233", "#EEF0F3", "#5588FF"),
        );
    }
}
