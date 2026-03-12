use egui::{Color32, CornerRadius, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, pos2};

use super::theme;

/// Renders a section heading — uppercase, small, muted.
pub fn section_heading(ui: &mut Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(title.to_ascii_uppercase())
            .size(10.5)
            .color(theme::tokens(ui).muted)
            .strong(),
    );
    ui.add_space(2.0);
}

/// A horizontal setting row: label on the left, control callback on the right.
pub fn setting_row(ui: &mut Ui, label: &str, add_right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(28.0);
        ui.label(
            RichText::new(label)
                .size(13.0)
                .color(theme::tokens(ui).text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_right);
    });
}

/// A custom toggle-switch widget (pill shape).
/// Returns `true` if the value changed.
pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> bool {
    let desired_size = Vec2::new(36.0, 20.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    let changed = response.clicked();
    if changed {
        *on = !*on;
    }

    let anim_t = ui.ctx().animate_bool_with_time(response.id, *on, 0.15);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = (rect.height() / 2.0) as u8;
        let rounding = CornerRadius::same(radius);
        let tokens = theme::tokens(ui);

        // Track
        let track_color = Color32::from_rgb(
            lerp_u8(tokens.surface_raised.r(), tokens.accent.r(), anim_t),
            lerp_u8(tokens.surface_raised.g(), tokens.accent.g(), anim_t),
            lerp_u8(tokens.surface_raised.b(), tokens.accent.b(), anim_t),
        );
        painter.rect_filled(rect, rounding, track_color);
        painter.rect_stroke(
            rect,
            rounding,
            Stroke::new(1.0, tokens.border),
            StrokeKind::Outside,
        );

        // Thumb
        let thumb_radius = (rect.height() - 6.0) / 2.0;
        let thumb_x = egui::lerp(
            rect.left() + 3.0 + thumb_radius..=rect.right() - 3.0 - thumb_radius,
            anim_t,
        );
        let thumb_center = pos2(thumb_x, rect.center().y);
        painter.circle_filled(thumb_center, thumb_radius, Color32::from_gray(250));
    }

    changed
}

/// Renders a thin separator line.
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

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let result = a as f32 + (b as f32 - a as f32) * t;
    result.round().clamp(0.0, 255.0) as u8
}
