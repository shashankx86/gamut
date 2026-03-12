use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadiusPreference {
    Small,
    Medium,
    Large,
    Custom,
}

impl RadiusPreference {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Custom => "Custom",
        }
    }
}

impl Default for RadiusPreference {
    fn default() -> Self {
        Self::Small
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
    pub const ALL: [Self; 4] = [Self::Small, Self::Medium, Self::Large, Self::ExtraLarge];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::ExtraLarge => "Extra large",
        }
    }
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

impl LauncherPlacement {}

impl Default for LauncherPlacement {
    fn default() -> Self {
        Self::RaisedCenter
    }
}
