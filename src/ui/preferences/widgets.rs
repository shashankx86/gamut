use egui::{
    pos2, Color32, CornerRadius, FontFamily, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use lucide_icons::Icon;

use super::theme;

// ── Lucide Icon Helpers ────────────────────────────────────────────────

fn lucide_font() -> FontFamily {
    FontFamily::Name("lucide".into())
}

pub fn lucide_icon(icon: Icon, size: f32) -> RichText {
    RichText::new(String::from(char::from(icon)))
        .family(lucide_font())
        .size(size)
}

pub fn lucide_font_id(size: f32) -> egui::FontId {
    egui::FontId::new(size, lucide_font())
}

// ── Section Card ───────────────────────────────────────────────────────

pub fn section_card(ui: &mut Ui, icon: Icon, title: &str, content: impl FnOnce(&mut Ui)) {
    let tokens = theme::tokens(ui);

    egui::Frame::new()
        .fill(tokens.surface)
        .corner_radius(10)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin {
            left: 16,
            right: 16,
            top: 14,
            bottom: 14,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(lucide_icon(icon, 15.0).color(tokens.accent));
                ui.add_space(2.0);
                ui.label(
                    RichText::new(title)
                        .size(12.5)
                        .color(tokens.text_primary)
                        .strong(),
                );
            });

            ui.add_space(10.0);
            content(ui);
        });

    ui.add_space(10.0);
}

// ── Setting Row ────────────────────────────────────────────────────────

pub fn setting_row(ui: &mut Ui, label: &str, add_right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);
        ui.label(
            RichText::new(label)
                .size(12.5)
                .color(theme::tokens(ui).text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_right);
    });
}

// ── Segmented Control ──────────────────────────────────────────────────

pub fn segmented_control(ui: &mut Ui, selected: usize, labels: &[&str]) -> Option<usize> {
    let tokens = theme::tokens(ui);
    let mut result = None;

    let bg = if tokens.is_dark {
        mix_c32(tokens.surface, tokens.surface_raised, 0.5)
    } else {
        tokens.surface
    };

    egui::Frame::new()
        .fill(bg)
        .corner_radius(6)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (idx, label) in labels.iter().enumerate() {
                    let is_active = idx == selected;
                    let (btn_bg, text_color) = if is_active {
                        (tokens.accent, theme::contrast_text_for(tokens.accent))
                    } else {
                        (Color32::TRANSPARENT, tokens.text_secondary)
                    };

                    let btn = egui::Button::new(RichText::new(*label).size(11.0).color(text_color))
                        .fill(btn_bg)
                        .stroke(Stroke::NONE)
                        .corner_radius(4)
                        .min_size(Vec2::new(64.0, 24.0));

                    if ui.add(btn).clicked() && !is_active {
                        result = Some(idx);
                    }
                }
            });
        });

    result
}

// ── Theme Preview Card ─────────────────────────────────────────────────

pub fn theme_card(ui: &mut Ui, icon: Icon, label: &str, is_active: bool) -> bool {
    let tokens = theme::tokens(ui);
    let bg = if is_active {
        tokens.accent_dim
    } else {
        tokens.surface_raised
    };
    let border_color = if is_active {
        tokens.accent
    } else {
        tokens.border
    };
    let text_color = if is_active {
        tokens.text_primary
    } else {
        tokens.text_secondary
    };

    let desired_size = Vec2::new(88.0, 58.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let rounding = CornerRadius::same(8);

        let fill = if response.hovered() && !is_active {
            tokens.hover_bg
        } else {
            bg
        };

        painter.rect_filled(rect, rounding, fill);
        painter.rect_stroke(
            rect,
            rounding,
            Stroke::new(if is_active { 1.5 } else { 1.0 }, border_color),
            StrokeKind::Outside,
        );

        painter.text(
            pos2(rect.center().x, rect.top() + 20.0),
            egui::Align2::CENTER_CENTER,
            String::from(char::from(icon)),
            lucide_font_id(17.0),
            text_color,
        );

        painter.text(
            pos2(rect.center().x, rect.bottom() - 14.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            text_color,
        );
    }

    response.clicked()
}

// ── Color Swatch ───────────────────────────────────────────────────────

pub fn color_swatch(ui: &mut Ui, hex_value: &str) {
    let tokens = theme::tokens(ui);
    let swatch_size = Vec2::new(22.0, 22.0);
    let (rect, _) = ui.allocate_exact_size(swatch_size, Sense::hover());

    if ui.is_rect_visible(rect) {
        let color = parse_color32(hex_value).unwrap_or(tokens.muted);
        ui.painter().rect_filled(rect, CornerRadius::same(4), color);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, tokens.border),
            StrokeKind::Outside,
        );
    }
}

fn parse_color32(value: &str) -> Option<Color32> {
    let trimmed = value.trim().trim_start_matches('#');
    match trimmed.len() {
        6 => {
            let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
            let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
            let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
            let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
            let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
            let a = u8::from_str_radix(&trimmed[6..8], 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}

// ── Toggle Switch ──────────────────────────────────────────────────────

pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> bool {
    let desired_size = Vec2::new(38.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    let changed = response.clicked();
    if changed {
        *on = !*on;
    }

    let anim_t = ui.ctx().animate_bool_with_time(response.id, *on, 0.15);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let rounding = CornerRadius::same((rect.height() / 2.0) as u8);
        let tokens = theme::tokens(ui);

        let track_color = Color32::from_rgb(
            lerp_u8(tokens.surface_raised.r(), tokens.accent.r(), anim_t),
            lerp_u8(tokens.surface_raised.g(), tokens.accent.g(), anim_t),
            lerp_u8(tokens.surface_raised.b(), tokens.accent.b(), anim_t),
        );

        painter.rect_filled(rect, rounding, track_color);
        painter.rect_stroke(
            rect,
            rounding,
            Stroke::new(
                1.0,
                if anim_t > 0.5 {
                    tokens.accent
                } else {
                    tokens.border
                },
            ),
            StrokeKind::Outside,
        );

        let thumb_radius = (rect.height() - 6.0) / 2.0;
        let thumb_x = egui::lerp(
            rect.left() + 3.0 + thumb_radius..=rect.right() - 3.0 - thumb_radius,
            anim_t,
        );
        let thumb_center = pos2(thumb_x, rect.center().y);
        let thumb_color = if anim_t > 0.5 {
            Color32::WHITE
        } else {
            tokens.text_primary
        };
        painter.circle_filled(thumb_center, thumb_radius, thumb_color);
    }

    changed
}

// ── Thin Separator ─────────────────────────────────────────────────────

pub fn thin_separator(ui: &mut Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = rect.top();
    let tokens = theme::tokens(ui);
    ui.painter().line_segment(
        [pos2(rect.left(), y), pos2(rect.right(), y)],
        Stroke::new(1.0, tokens.separator),
    );
    ui.add_space(1.0);
}

// ── Color Utilities ────────────────────────────────────────────────────

fn mix_c32(left: Color32, right: Color32, amount: f32) -> Color32 {
    Color32::from_rgb(
        lerp_u8(left.r(), right.r(), amount),
        lerp_u8(left.g(), right.g(), amount),
        lerp_u8(left.b(), right.b(), amount),
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}
