use egui::{Color32, CornerRadius, FontFamily, Sense, Stroke, StrokeKind, Ui, Vec2};
use lucide_icons::Icon;

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesTab {
    General,
    Shortcuts,
}

impl PreferencesTab {
    pub const ALL: [Self; 2] = [Self::General, Self::Shortcuts];

    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Shortcuts => "Shortcuts",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Self::General => Icon::Settings,
            Self::Shortcuts => Icon::Keyboard,
        }
    }
}

fn lucide_font() -> FontFamily {
    FontFamily::Name("lucide".into())
}

/// Returns `true` if the "Reset All" button was clicked.
pub fn render_sidebar(ui: &mut Ui, active: &mut PreferencesTab) -> bool {
    let mut reset_clicked = false;

    ui.vertical(|ui| {
        let tokens = theme::tokens(ui);

        // ── App Header ────────────────────────────────────────────
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("Gamut")
                .size(14.0)
                .color(tokens.text_primary)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Preferences")
                .size(11.0)
                .color(tokens.muted),
        );
        ui.add_space(12.0);

        // Separator
        let avail = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [
                egui::pos2(avail.left(), avail.top()),
                egui::pos2(avail.right(), avail.top()),
            ],
            Stroke::new(1.0, tokens.separator),
        );
        ui.add_space(12.0);

        // ── Tab Buttons ───────────────────────────────────────────
        for tab in PreferencesTab::ALL {
            let is_active = *active == tab;
            let text_color = if is_active {
                tokens.text_primary
            } else {
                tokens.muted
            };
            let icon_color = if is_active {
                tokens.accent
            } else {
                tokens.muted
            };

            let desired_size = Vec2::new(ui.available_width(), 34.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                let rounding = CornerRadius::same(6);

                let bg = if is_active {
                    tokens.accent_dim
                } else if response.hovered() {
                    tokens.hover_bg
                } else {
                    Color32::TRANSPARENT
                };

                painter.rect_filled(rect, rounding, bg);

                if is_active {
                    painter.rect_stroke(
                        rect,
                        rounding,
                        Stroke::new(1.0, tokens.accent),
                        StrokeKind::Outside,
                    );
                }

                // Icon
                painter.text(
                    egui::pos2(rect.left() + 12.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    String::from(char::from(tab.icon())),
                    egui::FontId::new(14.0, lucide_font()),
                    icon_color,
                );

                // Label
                painter.text(
                    egui::pos2(rect.left() + 34.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    tab.label(),
                    egui::FontId::proportional(12.0),
                    text_color,
                );
            }

            if response.clicked() {
                *active = tab;
            }

            ui.add_space(2.0);
        }

        // ── Reset Button at Bottom ────────────────────────────────
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(12.0);

            let desired_size = Vec2::new(ui.available_width(), 30.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                let rounding = CornerRadius::same(6);

                let bg = if response.hovered() {
                    tokens.hover_bg
                } else {
                    Color32::TRANSPARENT
                };

                painter.rect_filled(rect, rounding, bg);
                painter.rect_stroke(
                    rect,
                    rounding,
                    Stroke::new(1.0, tokens.border),
                    StrokeKind::Outside,
                );

                // Icon
                painter.text(
                    egui::pos2(rect.left() + 10.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    String::from(char::from(Icon::RotateCcw)),
                    egui::FontId::new(12.0, lucide_font()),
                    tokens.muted,
                );

                // Label
                painter.text(
                    egui::pos2(rect.left() + 30.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Reset All",
                    egui::FontId::proportional(11.0),
                    tokens.muted,
                );
            }

            if response.clicked() {
                reset_clicked = true;
            }
        });
    });

    reset_clicked
}
