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

pub(crate) fn default_custom_theme_colors() -> ThemeColors {
    default_theme_colors(ThemeSchemeId::Dark)
}

pub(crate) const fn theme_seed(scheme: ThemeSchemeId) -> ThemeSeedSpec {
    match scheme {
        ThemeSchemeId::Light => LIGHT_SEED,
        ThemeSchemeId::Dark => DARK_SEED,
    }
}

pub(crate) fn classify_theme_colors(colors: &ThemeColors) -> ThemeSchemeId {
    let Some((red, green, blue)) = parse_rgb(&colors.background) else {
        return ThemeSchemeId::Dark;
    };

    let luminance = (0.2126 * red) + (0.7152 * green) + (0.0722 * blue);

    if luminance >= 0.55 {
        ThemeSchemeId::Light
    } else {
        ThemeSchemeId::Dark
    }
}

fn parse_rgb(value: &str) -> Option<(f32, f32, f32)> {
    let trimmed = value.trim().trim_start_matches('#');

    if trimmed.len() != 6 && trimmed.len() != 8 {
        return None;
    }

    let red = u8::from_str_radix(&trimmed[0..2], 16).ok()? as f32 / 255.0;
    let green = u8::from_str_radix(&trimmed[2..4], 16).ok()? as f32 / 255.0;
    let blue = u8::from_str_radix(&trimmed[4..6], 16).ok()? as f32 / 255.0;

    Some((red, green, blue))
}
