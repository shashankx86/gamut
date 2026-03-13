use crate::core::preferences::{ThemeColors, ThemeSchemeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ThemeSeedSpec {
    pub background: &'static str,
    pub text: &'static str,
    pub accent: &'static str,
}

pub(crate) const LIGHT_SEED: ThemeSeedSpec = ThemeSeedSpec {
    background: "#F3F4F6",
    text: "#1F2328",
    accent: "#416EF5",
};

pub(crate) const DARK_SEED: ThemeSeedSpec = ThemeSeedSpec {
    background: "#151516",
    text: "#EBEDF2",
    accent: "#5E8BFF",
};

pub(crate) fn default_theme_colors(scheme: ThemeSchemeId) -> ThemeColors {
    let seed = theme_seed(scheme);
    ThemeColors::new(seed.background, seed.text, seed.accent)
}

pub(crate) const fn theme_seed(scheme: ThemeSchemeId) -> ThemeSeedSpec {
    match scheme {
        ThemeSchemeId::Light => LIGHT_SEED,
        ThemeSchemeId::Dark => DARK_SEED,
    }
}
