use super::palette::{ResolvedAppearance, ThemePalette};
use crate::core::preferences::{
    AppearancePreferences, ThemeColors, ThemePreference, ThemeSchemeId,
};
use dark_light::Mode as SystemThemeMode;
use iced::theme::Palette;
use iced::{Color, Theme};

pub(in crate::ui) fn resolve_theme(preferences: &AppearancePreferences) -> Theme {
    let resolved = resolve_palette(preferences);
    Theme::custom(theme_name(preferences.theme), resolved.palette)
}

pub(in crate::ui) fn resolve_appearance(preferences: &AppearancePreferences) -> ResolvedAppearance {
    resolve_palette(preferences).appearance
}

fn resolve_palette(preferences: &AppearancePreferences) -> ThemePalette {
    let scheme = match preferences.theme {
        ThemePreference::Light => ThemeSchemeId::Light,
        ThemePreference::Dark => ThemeSchemeId::Dark,
        ThemePreference::System => detect_system_scheme(),
    };

    palette_from_colors(preferences.scheme(scheme))
}

fn theme_name(preference: ThemePreference) -> &'static str {
    match preference {
        ThemePreference::Light => "Gamut Light",
        ThemePreference::Dark => "Gamut Dark",
        ThemePreference::System => "Gamut System",
    }
}

fn detect_system_scheme() -> ThemeSchemeId {
    match dark_light::detect().unwrap_or(SystemThemeMode::Unspecified) {
        SystemThemeMode::Light => ThemeSchemeId::Light,
        _ => ThemeSchemeId::Dark,
    }
}

fn palette_from_colors(colors: &ThemeColors) -> ThemePalette {
    let palette = parse_palette(colors).unwrap_or_else(|| {
        parse_palette(&default_dark_colors()).expect("default dark colors should parse")
    });
    let appearance = appearance_from_palette(palette);
    ThemePalette {
        palette,
        appearance,
    }
}

fn parse_palette(colors: &ThemeColors) -> Option<Palette> {
    Some(Palette {
        background: parse_hex_color(&colors.background)?,
        text: parse_hex_color(&colors.text)?,
        primary: parse_hex_color(&colors.accent)?,
        success: Color::from_rgb(0.22, 0.69, 0.44),
        warning: Color::from_rgb(0.93, 0.69, 0.20),
        danger: Color::from_rgb(0.86, 0.30, 0.33),
    })
}

fn default_dark_colors() -> ThemeColors {
    ThemeColors::new("#151516", "#EBEDF2", "#5E8BFF")
}

fn appearance_from_palette(palette: Palette) -> ResolvedAppearance {
    let is_dark = relative_luminance(palette.background) < 0.5;
    let panel_background = palette.background;
    let panel_border = mix(
        palette.background,
        palette.text,
        if is_dark { 0.18 } else { 0.12 },
    );
    let secondary_text = mix(palette.text, palette.background, 0.35);
    let muted_text = mix(palette.text, palette.background, 0.48);
    let divider = mix(palette.text, palette.background, 0.80);
    let search_icon = mix(palette.text, palette.background, 0.55);
    let selection = with_alpha(mix(palette.primary, palette.background, 0.30), 0.84);
    let first_row_active = with_alpha(mix(palette.background, palette.text, 0.08), 0.96);
    let first_row_hover = with_alpha(mix(palette.background, palette.text, 0.12), 0.98);
    let first_row_pressed = with_alpha(mix(palette.background, palette.text, 0.18), 0.98);
    let row_hover = with_alpha(mix(palette.background, palette.text, 0.06), 0.90);
    let row_pressed = with_alpha(mix(palette.background, palette.text, 0.12), 0.94);
    let scrollbar_track = with_alpha(mix(palette.background, palette.text, 0.08), 0.70);
    let scrollbar_track_border = with_alpha(mix(palette.background, palette.text, 0.16), 0.84);
    let scrollbar_scroller = with_alpha(mix(palette.background, palette.text, 0.30), 0.84);
    let scrollbar_scroller_border = with_alpha(mix(palette.background, palette.text, 0.42), 0.88);
    let scrollbar_scroller_hover = with_alpha(mix(palette.background, palette.text, 0.36), 0.90);
    let scrollbar_scroller_hover_border =
        with_alpha(mix(palette.background, palette.text, 0.48), 0.90);
    let scrollbar_scroller_dragged = with_alpha(mix(palette.background, palette.text, 0.46), 0.92);
    let scrollbar_scroller_dragged_border =
        with_alpha(mix(palette.background, palette.text, 0.58), 0.92);

    ResolvedAppearance {
        panel_background,
        panel_border,
        primary_text: palette.text,
        secondary_text,
        muted_text,
        divider,
        search_icon,
        selection,
        first_row_active,
        first_row_hover,
        first_row_pressed,
        row_hover,
        row_pressed,
        scrollbar_track,
        scrollbar_track_border,
        scrollbar_scroller,
        scrollbar_scroller_border,
        scrollbar_scroller_hover,
        scrollbar_scroller_hover_border,
        scrollbar_scroller_dragged,
        scrollbar_scroller_dragged_border,
    }
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

fn relative_luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
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

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

#[cfg(test)]
mod tests {
    use super::{parse_palette, resolve_theme};
    use crate::core::preferences::{
        AppearancePreferences, ThemeColors, ThemePreference, ThemeSchemeId,
    };

    #[test]
    fn invalid_scheme_falls_back_to_default_dark_palette() {
        let mut preferences = AppearancePreferences::default();
        preferences.theme = ThemePreference::Dark;
        *preferences.scheme_mut(ThemeSchemeId::Dark) =
            ThemeColors::new("invalid", "#FFFFFF", "#3366FF");

        let theme = resolve_theme(&preferences);
        assert_eq!(
            theme.palette().background,
            iced::Color::from_rgb8(21, 21, 22)
        );
    }

    #[test]
    fn valid_scheme_parses_palette() {
        let palette = parse_palette(&ThemeColors::new("#112233", "#EEF0F3", "#5588FF"))
            .expect("expected scheme palette to parse");

        assert!(palette.text.r > palette.background.r);
    }
}
