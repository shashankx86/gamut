use super::palette::{ResolvedAppearance, ThemeScheme};
use crate::core::preferences::{AppearancePreferences, ThemeColors, ThemeSchemeId};
use crate::core::theme::theme_seed;
use iced::Color;

#[derive(Debug, Clone, Copy)]
struct ThemeSeed {
    background: Color,
    text: Color,
    accent: Color,
}

pub(in crate::ui) fn shared_color_scheme(
    preferences: &AppearancePreferences,
) -> ResolvedAppearance {
    let resolved = preferences.resolved_theme();
    let scheme = match resolved.variant {
        ThemeSchemeId::Light => ThemeScheme::Light,
        ThemeSchemeId::Dark => ThemeScheme::Dark,
    };
    let seed = parse_seed(&resolved.colors).unwrap_or_else(|| default_seed(scheme));

    match scheme {
        ThemeScheme::Light => light_scheme(seed),
        ThemeScheme::Dark => dark_scheme(seed),
    }
}

fn light_scheme(seed: ThemeSeed) -> ResolvedAppearance {
    let panel_background = seed.background;
    let panel_surface = mix(panel_background, seed.text, 0.05);
    let panel_surface_raised = mix(panel_background, seed.text, 0.10);
    let primary_text = seed.text;
    let secondary_text = mix(primary_text, panel_background, 0.30);
    let muted_text = mix(primary_text, panel_background, 0.44);
    let divider = mix(primary_text, panel_background, 0.86);
    let search_icon = mix(primary_text, panel_background, 0.48);
    let accent = seed.accent;
    let accent_soft = mix(accent, panel_surface_raised, 0.84);

    ResolvedAppearance {
        panel_background,
        panel_border: mix(primary_text, panel_background, 0.80),
        primary_text,
        secondary_text,
        muted_text,
        divider,
        search_icon,
        accent,
        accent_soft,
        accent_strong: mix(accent, primary_text, 0.18),
        first_row_active: mix(accent, panel_surface, 0.84),
        first_row_hover: mix(accent, panel_surface, 0.74),
        first_row_pressed: mix(accent, panel_surface, 0.64),
        row_hover: mix(primary_text, panel_surface, 0.96),
        row_pressed: mix(primary_text, panel_surface, 0.91),
        scrollbar_track: mix(primary_text, panel_background, 0.94),
        scrollbar_track_border: mix(primary_text, panel_background, 0.88),
        scrollbar_scroller: mix(primary_text, panel_background, 0.72),
        scrollbar_scroller_border: mix(primary_text, panel_background, 0.66),
        scrollbar_scroller_hover: mix(primary_text, panel_background, 0.64),
        scrollbar_scroller_hover_border: mix(primary_text, panel_background, 0.58),
        scrollbar_scroller_dragged: mix(primary_text, panel_background, 0.56),
        scrollbar_scroller_dragged_border: mix(primary_text, panel_background, 0.50),
    }
}

fn dark_scheme(seed: ThemeSeed) -> ResolvedAppearance {
    let panel_background = seed.background;
    let panel_surface = mix(panel_background, white(), 0.05);
    let panel_surface_raised = mix(panel_background, white(), 0.10);
    let primary_text = seed.text;
    let secondary_text = mix(primary_text, panel_background, 0.24);
    let muted_text = mix(primary_text, panel_background, 0.38);
    let divider = mix(primary_text, panel_background, 0.82);
    let search_icon = mix(primary_text, panel_background, 0.40);
    let accent = seed.accent;
    let accent_soft = mix(accent, panel_surface_raised, 0.70);

    ResolvedAppearance {
        panel_background,
        panel_border: mix(primary_text, panel_background, 0.84),
        primary_text,
        secondary_text,
        muted_text,
        divider,
        search_icon,
        accent,
        accent_soft,
        accent_strong: mix(accent, primary_text, 0.14),
        first_row_active: mix(accent, panel_surface_raised, 0.68),
        first_row_hover: mix(accent, panel_surface_raised, 0.56),
        first_row_pressed: mix(accent, panel_surface_raised, 0.44),
        row_hover: mix(primary_text, panel_surface, 0.94),
        row_pressed: mix(primary_text, panel_surface, 0.88),
        scrollbar_track: mix(primary_text, panel_background, 0.92),
        scrollbar_track_border: mix(primary_text, panel_background, 0.86),
        scrollbar_scroller: mix(primary_text, panel_background, 0.74),
        scrollbar_scroller_border: mix(primary_text, panel_background, 0.68),
        scrollbar_scroller_hover: mix(primary_text, panel_background, 0.66),
        scrollbar_scroller_hover_border: mix(primary_text, panel_background, 0.60),
        scrollbar_scroller_dragged: mix(primary_text, panel_background, 0.58),
        scrollbar_scroller_dragged_border: mix(primary_text, panel_background, 0.52),
    }
}

fn parse_seed(colors: &ThemeColors) -> Option<ThemeSeed> {
    Some(ThemeSeed {
        background: parse_hex_color(&colors.background)?,
        text: parse_hex_color(&colors.text)?,
        accent: parse_hex_color(&colors.accent)?,
    })
}

fn default_seed(scheme: ThemeScheme) -> ThemeSeed {
    let scheme_id = match scheme {
        ThemeScheme::Light => ThemeSchemeId::Light,
        ThemeScheme::Dark => ThemeSchemeId::Dark,
    };
    let seed = theme_seed(scheme_id);
    ThemeSeed {
        background: hex_color(seed.background),
        text: hex_color(seed.text),
        accent: hex_color(seed.accent),
    }
}

fn hex_color(value: &str) -> Color {
    parse_hex_color(value).expect("static theme colors should parse")
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let trimmed = value.trim().trim_start_matches('#');

    match trimmed.len() {
        6 => {
            let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
            let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
            let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
            let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
            let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
            let a = u8::from_str_radix(&trimmed[6..8], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
        }
        _ => None,
    }
}

fn mix(left: Color, right: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: left.r + (right.r - left.r) * amount,
        g: left.g + (right.g - left.g) * amount,
        b: left.b + (right.b - left.b) * amount,
        a: left.a + (right.a - left.a) * amount,
    }
}

fn white() -> Color {
    Color::from_rgb8(255, 255, 255)
}
