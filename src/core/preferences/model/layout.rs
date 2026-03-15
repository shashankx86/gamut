use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadiusPreference {
    Small,
    Medium,
    Large,
    Custom,
}

impl RadiusPreference {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::Custom => "custom",
        }
    }
}

impl Default for RadiusPreference {
    fn default() -> Self {
        Self::Small
    }
}

impl fmt::Display for RadiusPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RadiusPreference {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "small" => Ok(Self::Small),
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            "custom" => Ok(Self::Custom),
            _ => Err("expected one of: small, medium, large, custom".to_string()),
        }
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

impl LauncherSize {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::ExtraLarge => "extra_large",
        }
    }
}

impl Default for LauncherSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl fmt::Display for LauncherSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LauncherSize {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "small" => Ok(Self::Small),
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            "extra_large" | "extralarge" | "extra-large" => Ok(Self::ExtraLarge),
            _ => Err("expected one of: small, medium, large, extra_large".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherPlacement {
    Center,
    RaisedCenter,
    Custom,
}

impl LauncherPlacement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::RaisedCenter => "raised_center",
            Self::Custom => "custom",
        }
    }
}

impl Default for LauncherPlacement {
    fn default() -> Self {
        Self::RaisedCenter
    }
}

impl fmt::Display for LauncherPlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LauncherPlacement {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "center" => Ok(Self::Center),
            "raised_center" | "raisedcenter" | "raised-center" => Ok(Self::RaisedCenter),
            "custom" => Ok(Self::Custom),
            _ => Err("expected one of: center, raised_center, custom".to_string()),
        }
    }
}
