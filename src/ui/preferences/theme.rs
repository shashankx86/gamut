use egui::{Color32, CornerRadius, Stroke, Style, Visuals, style::Selection};

// ── Color palette ──────────────────────────────────────────────────────────

pub const BASE: Color32 = Color32::from_rgb(20, 20, 20);
pub const SURFACE: Color32 = Color32::from_rgb(30, 30, 30);
pub const SURFACE_RAISED: Color32 = Color32::from_rgb(39, 39, 42);
pub const BORDER: Color32 = Color32::from_rgb(45, 45, 48);
pub const MUTED: Color32 = Color32::from_rgb(113, 113, 122);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(161, 161, 170);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(228, 228, 231);
pub const ACCENT: Color32 = Color32::from_rgb(94, 139, 255);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(62, 95, 180);
pub const HOVER_BG: Color32 = Color32::from_rgb(35, 35, 38);
pub const SEPARATOR: Color32 = Color32::from_rgb(38, 38, 42);

// ── Apply to context ───────────────────────────────────────────────────────

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    style.visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(TEXT_PRIMARY),
        panel_fill: BASE,
        window_fill: SURFACE,
        extreme_bg_color: Color32::from_rgb(14, 14, 14),
        faint_bg_color: SURFACE,
        hyperlink_color: ACCENT,

        widgets: style_widgets(),

        selection: Selection {
            bg_fill: Color32::from_rgba_premultiplied(94, 139, 255, 60),
            stroke: Stroke::new(1.0, ACCENT),
        },

        window_shadow: egui::epaint::Shadow::NONE,
        popup_shadow: egui::epaint::Shadow {
            spread: 0,
            blur: 8,
            offset: [0, 2],
            color: Color32::from_black_alpha(80),
        },
        window_corner_radius: CornerRadius::same(4),
        window_stroke: Stroke::new(1.0, BORDER),

        ..Visuals::dark()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(0);
    style.spacing.combo_width = 120.0;

    ctx.set_style(style);
}

fn style_widgets() -> egui::style::Widgets {
    use egui::style::{WidgetVisuals, Widgets};

    let corner_radius = CornerRadius::same(3);

    Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: SURFACE,
            weak_bg_fill: SURFACE,
            bg_stroke: Stroke::new(1.0, BORDER),
            corner_radius,
            fg_stroke: Stroke::new(1.0, TEXT_SECONDARY),
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            bg_fill: SURFACE_RAISED,
            weak_bg_fill: SURFACE,
            bg_stroke: Stroke::new(1.0, BORDER),
            corner_radius,
            fg_stroke: Stroke::new(1.0, TEXT_SECONDARY),
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            bg_fill: HOVER_BG,
            weak_bg_fill: HOVER_BG,
            bg_stroke: Stroke::new(1.0, ACCENT_DIM),
            corner_radius,
            fg_stroke: Stroke::new(1.0, TEXT_PRIMARY),
            expansion: 1.0,
        },
        active: WidgetVisuals {
            bg_fill: ACCENT_DIM,
            weak_bg_fill: ACCENT_DIM,
            bg_stroke: Stroke::new(1.0, ACCENT),
            corner_radius,
            fg_stroke: Stroke::new(1.0, Color32::WHITE),
            expansion: 0.0,
        },
        open: WidgetVisuals {
            bg_fill: SURFACE_RAISED,
            weak_bg_fill: SURFACE_RAISED,
            bg_stroke: Stroke::new(1.0, ACCENT_DIM),
            corner_radius,
            fg_stroke: Stroke::new(1.0, TEXT_PRIMARY),
            expansion: 0.0,
        },
    }
}
