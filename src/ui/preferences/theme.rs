use crate::core::preferences::AppPreferences;
use crate::ui::theme::resolve_appearance;
use egui::style::{Selection, WidgetVisuals, Widgets};
use egui::{Color32, Context, CornerRadius, Id, Stroke, Style, Ui, Visuals};

#[derive(Debug, Clone)]
pub struct PreferenceThemeTokens {
    pub base: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub muted: Color32,
    pub text_secondary: Color32,
    pub text_primary: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub hover_bg: Color32,
    pub separator: Color32,
    pub is_dark: bool,
}

pub fn apply_theme(ctx: &Context, preferences: &AppPreferences) {
    let tokens = PreferenceThemeTokens::from_preferences(preferences);
    let mut style = Style::default();

    style.visuals = Visuals {
        dark_mode: tokens.is_dark,
        override_text_color: Some(tokens.text_primary),
        panel_fill: tokens.base,
        window_fill: tokens.surface_raised,
        extreme_bg_color: mix(tokens.base, tokens.surface_raised, 0.45),
        faint_bg_color: tokens.surface,
        hyperlink_color: tokens.accent,
        widgets: style_widgets(&tokens),
        selection: Selection {
            bg_fill: tokens.accent_dim,
            stroke: Stroke::new(1.0, tokens.accent),
        },
        window_shadow: egui::epaint::Shadow::NONE,
        popup_shadow: egui::epaint::Shadow {
            spread: 0,
            blur: 8,
            offset: [0, 2],
            color: Color32::from_black_alpha(80),
        },
        window_corner_radius: CornerRadius::same(4),
        window_stroke: Stroke::new(1.0, tokens.border),
        ..if tokens.is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        }
    };

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(0);
    style.spacing.combo_width = 120.0;

    ctx.data_mut(|data| data.insert_temp(tokens_id(), tokens));
    ctx.set_style(style);
}

pub fn tokens(ui: &Ui) -> PreferenceThemeTokens {
    ui.ctx()
        .data(|data| data.get_temp(tokens_id()))
        .unwrap_or_else(PreferenceThemeTokens::fallback)
}

pub fn tokens_from_preferences(preferences: &AppPreferences) -> PreferenceThemeTokens {
    PreferenceThemeTokens::from_preferences(preferences)
}

impl PreferenceThemeTokens {
    fn from_preferences(preferences: &AppPreferences) -> Self {
        let appearance = resolve_appearance(&preferences.appearance);

        Self {
            base: to_color32(appearance.panel_background),
            surface: to_color32(appearance.panel_surface),
            surface_raised: to_color32(appearance.panel_surface_raised),
            border: to_color32(appearance.panel_border),
            muted: to_color32(appearance.muted_text),
            text_secondary: to_color32(appearance.secondary_text),
            text_primary: to_color32(appearance.primary_text),
            accent: to_color32(appearance.accent),
            accent_dim: to_color32(appearance.accent_soft),
            hover_bg: to_color32(appearance.first_row_hover),
            separator: to_color32(appearance.divider),
            is_dark: relative_luminance(to_color32(appearance.panel_background)) < 0.5,
        }
    }

    fn fallback() -> Self {
        Self::from_preferences(&AppPreferences::default())
    }
}

fn style_widgets(tokens: &PreferenceThemeTokens) -> Widgets {
    let corner_radius = CornerRadius::same(3);

    Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: tokens.surface,
            weak_bg_fill: tokens.surface,
            bg_stroke: Stroke::new(1.0, tokens.border),
            corner_radius,
            fg_stroke: Stroke::new(1.0, tokens.text_secondary),
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            bg_fill: tokens.surface_raised,
            weak_bg_fill: tokens.surface_raised,
            bg_stroke: Stroke::new(1.0, tokens.border),
            corner_radius,
            fg_stroke: Stroke::new(1.0, tokens.text_secondary),
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            bg_fill: tokens.hover_bg,
            weak_bg_fill: tokens.hover_bg,
            bg_stroke: Stroke::new(1.0, tokens.accent_dim),
            corner_radius,
            fg_stroke: Stroke::new(1.0, tokens.text_primary),
            expansion: 1.0,
        },
        active: WidgetVisuals {
            bg_fill: tokens.accent_dim,
            weak_bg_fill: tokens.accent_dim,
            bg_stroke: Stroke::new(1.0, tokens.accent),
            corner_radius,
            fg_stroke: Stroke::new(1.0, tokens.text_primary),
            expansion: 0.0,
        },
        open: WidgetVisuals {
            bg_fill: tokens.hover_bg,
            weak_bg_fill: tokens.surface_raised,
            bg_stroke: Stroke::new(1.0, tokens.accent),
            corner_radius,
            fg_stroke: Stroke::new(1.0, tokens.text_primary),
            expansion: 0.0,
        },
    }
}

fn to_color32(color: iced::Color) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
        (color.a * 255.0).round() as u8,
    )
}

fn mix(left: Color32, right: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(
        channel_mix(left.r(), right.r(), amount),
        channel_mix(left.g(), right.g(), amount),
        channel_mix(left.b(), right.b(), amount),
        channel_mix(left.a(), right.a(), amount),
    )
}

fn channel_mix(left: u8, right: u8, amount: f32) -> u8 {
    (left as f32 + (right as f32 - left as f32) * amount)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn relative_luminance(color: Color32) -> f32 {
    0.2126 * (color.r() as f32 / 255.0)
        + 0.7152 * (color.g() as f32 / 255.0)
        + 0.0722 * (color.b() as f32 / 255.0)
}

fn tokens_id() -> Id {
    Id::new("preferences.theme.tokens")
}
