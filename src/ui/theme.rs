use crate::core::preferences::{AppearancePreferences, CustomThemeColors, ThemePreference};
use dark_light::Mode as SystemThemeMode;
use iced::theme::Palette;
use iced::{Color, Theme};

#[derive(Debug, Clone, Copy)]
pub(in crate::ui) struct ResolvedAppearance {
    pub panel_background: Color,
    pub panel_border: Color,
    pub primary_text: Color,
    pub secondary_text: Color,
    pub muted_text: Color,
    pub divider: Color,
    pub search_icon: Color,
    pub selection: Color,
    pub first_row_active: Color,
    pub first_row_hover: Color,
    pub first_row_pressed: Color,
    pub row_hover: Color,
    pub row_pressed: Color,
    pub scrollbar_track: Color,
    pub scrollbar_track_border: Color,
    pub scrollbar_scroller: Color,
    pub scrollbar_scroller_border: Color,
    pub scrollbar_scroller_hover: Color,
    pub scrollbar_scroller_hover_border: Color,
    pub scrollbar_scroller_dragged: Color,
    pub scrollbar_scroller_dragged_border: Color,
}

pub(in crate::ui) fn resolve_theme(preferences: &AppearancePreferences) -> Theme {
    match resolve_theme_choice(preferences) {
        ThemeChoice::Dark => Theme::custom("Gamut Dark", dark_palette()),
        ThemeChoice::Light => Theme::custom("Gamut Light", light_palette()),
        ThemeChoice::Custom(palette) => Theme::custom("Gamut Custom", palette),
    }
}

pub(in crate::ui) fn resolve_appearance(preferences: &AppearancePreferences) -> ResolvedAppearance {
    match resolve_theme_choice(preferences) {
        ThemeChoice::Dark => dark_appearance(),
        ThemeChoice::Light => light_appearance(),
        ThemeChoice::Custom(palette) => custom_appearance(palette),
    }
}

enum ThemeChoice {
    Dark,
    Light,
    Custom(Palette),
}

fn resolve_theme_choice(preferences: &AppearancePreferences) -> ThemeChoice {
    match preferences.theme {
        ThemePreference::Dark => ThemeChoice::Dark,
        ThemePreference::Light => ThemeChoice::Light,
        ThemePreference::System => match detect_system_theme() {
            SystemThemeMode::Light => ThemeChoice::Light,
            _ => ThemeChoice::Dark,
        },
        ThemePreference::Custom => parse_custom_palette(&preferences.custom_theme)
            .map(ThemeChoice::Custom)
            .unwrap_or(ThemeChoice::Dark),
    }
}

fn detect_system_theme() -> SystemThemeMode {
    dark_light::detect().unwrap_or(SystemThemeMode::Unspecified)
}

fn parse_custom_palette(colors: &CustomThemeColors) -> Option<Palette> {
    Some(Palette {
        background: parse_hex_color(&colors.background)?,
        text: parse_hex_color(&colors.text)?,
        primary: parse_hex_color(&colors.accent)?,
        success: Color::from_rgb(0.22, 0.69, 0.44),
        warning: Color::from_rgb(0.93, 0.69, 0.20),
        danger: Color::from_rgb(0.86, 0.30, 0.33),
    })
}

fn dark_palette() -> Palette {
    Palette {
        background: rgb8(21, 21, 22),
        text: rgb8(235, 237, 242),
        primary: rgb8(94, 139, 255),
        success: rgb8(71, 176, 112),
        warning: rgb8(226, 178, 71),
        danger: rgb8(212, 94, 94),
    }
}

fn light_palette() -> Palette {
    Palette {
        background: rgb8(248, 249, 251),
        text: rgb8(28, 34, 42),
        primary: rgb8(65, 110, 245),
        success: rgb8(52, 153, 108),
        warning: rgb8(201, 141, 42),
        danger: rgb8(194, 73, 80),
    }
}

fn dark_appearance() -> ResolvedAppearance {
    ResolvedAppearance {
        panel_background: rgba8(21, 21, 22, 1.0),
        panel_border: rgba8(48, 49, 52, 0.95),
        primary_text: rgb8(235, 237, 242),
        secondary_text: rgb8(140, 148, 156),
        muted_text: rgb8(131, 135, 143),
        divider: rgba8(46, 46, 46, 0.86),
        search_icon: rgb8(114, 114, 114),
        selection: rgba_f32(0.31, 0.31, 0.34, 0.88),
        first_row_active: rgba8(32, 32, 32, 0.92),
        first_row_hover: rgba8(38, 38, 38, 0.94),
        first_row_pressed: rgba8(45, 45, 45, 0.94),
        row_hover: rgba8(24, 24, 24, 0.28),
        row_pressed: rgba8(32, 32, 32, 0.55),
        scrollbar_track: rgba_f32(0.08, 0.08, 0.08, 0.38),
        scrollbar_track_border: rgba8(40, 40, 40, 0.42),
        scrollbar_scroller: rgba8(92, 92, 92, 0.84),
        scrollbar_scroller_border: rgba8(120, 120, 120, 0.56),
        scrollbar_scroller_hover: rgba8(78, 78, 78, 0.88),
        scrollbar_scroller_hover_border: rgba8(108, 108, 108, 0.60),
        scrollbar_scroller_dragged: rgba8(68, 68, 68, 0.90),
        scrollbar_scroller_dragged_border: rgba8(95, 95, 95, 0.62),
    }
}

fn light_appearance() -> ResolvedAppearance {
    ResolvedAppearance {
        panel_background: rgba8(248, 249, 251, 1.0),
        panel_border: rgba8(214, 220, 227, 0.98),
        primary_text: rgb8(28, 34, 42),
        secondary_text: rgb8(90, 100, 114),
        muted_text: rgb8(111, 120, 133),
        divider: rgba8(217, 222, 228, 0.95),
        search_icon: rgb8(128, 136, 148),
        selection: rgba8(188, 208, 255, 0.82),
        first_row_active: rgba8(233, 237, 243, 0.98),
        first_row_hover: rgba8(225, 231, 239, 1.0),
        first_row_pressed: rgba8(214, 221, 231, 1.0),
        row_hover: rgba8(235, 239, 245, 0.92),
        row_pressed: rgba8(223, 229, 237, 0.96),
        scrollbar_track: rgba8(231, 235, 240, 0.92),
        scrollbar_track_border: rgba8(208, 215, 223, 0.96),
        scrollbar_scroller: rgba8(170, 179, 190, 0.92),
        scrollbar_scroller_border: rgba8(154, 164, 177, 0.98),
        scrollbar_scroller_hover: rgba8(158, 167, 178, 0.94),
        scrollbar_scroller_hover_border: rgba8(142, 153, 166, 0.98),
        scrollbar_scroller_dragged: rgba8(146, 156, 168, 0.96),
        scrollbar_scroller_dragged_border: rgba8(131, 143, 157, 0.98),
    }
}

fn custom_appearance(palette: Palette) -> ResolvedAppearance {
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
            Some(rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
            let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
            let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
            let a = u8::from_str_radix(&trimmed[6..8], 16).ok()?;
            Some(rgba8(r, g, b, a as f32 / 255.0))
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

fn rgb8(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

fn rgba8(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba8(r, g, b, a)
}

fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::from_rgba(r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::{custom_appearance, parse_custom_palette, resolve_theme, resolve_theme_choice};
    use crate::core::preferences::{AppearancePreferences, CustomThemeColors, ThemePreference};

    #[test]
    fn invalid_custom_theme_falls_back_to_dark_choice() {
        let preferences = AppearancePreferences {
            theme: ThemePreference::Custom,
            custom_theme: CustomThemeColors {
                background: "invalid".to_string(),
                text: "#FFFFFF".to_string(),
                accent: "#3366FF".to_string(),
            },
            ..AppearancePreferences::default()
        };

        assert!(matches!(
            resolve_theme_choice(&preferences),
            super::ThemeChoice::Dark
        ));
    }

    #[test]
    fn valid_custom_theme_parses_palette() {
        let palette = parse_custom_palette(&CustomThemeColors {
            background: "#112233".to_string(),
            text: "#EEF0F3".to_string(),
            accent: "#5588FF".to_string(),
        })
        .expect("expected custom palette to parse");

        let appearance = custom_appearance(palette);
        assert!(appearance.primary_text.r > appearance.panel_background.r);
    }

    #[test]
    fn resolve_theme_supports_custom_mode() {
        let preferences = AppearancePreferences {
            theme: ThemePreference::Custom,
            custom_theme: CustomThemeColors {
                background: "#112233".to_string(),
                text: "#EEF0F3".to_string(),
                accent: "#5588FF".to_string(),
            },
            ..AppearancePreferences::default()
        };

        let theme = resolve_theme(&preferences);
        assert_eq!(theme.palette().background, palette_background(&preferences));
    }

    fn palette_background(preferences: &AppearancePreferences) -> iced::Color {
        parse_custom_palette(&preferences.custom_theme)
            .expect("expected custom palette to parse")
            .background
    }
}
