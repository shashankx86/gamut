use egui::{Color32, RichText, Stroke, Ui, Vec2};

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

    pub fn icon(self) -> &'static str {
        match self {
            Self::General => "⚙",
            Self::Shortcuts => "⌨",
        }
    }
}

pub fn render_sidebar(ui: &mut Ui, active: &mut PreferencesTab) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        for tab in PreferencesTab::ALL {
            let is_active = *active == tab;

            let bg = if is_active {
                theme::SURFACE_RAISED
            } else {
                Color32::TRANSPARENT
            };
            let text_color = if is_active {
                theme::TEXT_PRIMARY
            } else {
                theme::MUTED
            };

            let button = egui::Button::new(
                RichText::new(format!("{}  {}", tab.icon(), tab.label()))
                    .size(12.0)
                    .color(text_color),
            )
            .fill(bg)
            .stroke(if is_active {
                Stroke::new(1.0, theme::BORDER)
            } else {
                Stroke::NONE
            })
            .corner_radius(4)
            .min_size(Vec2::new(ui.available_width(), 32.0));

            if ui.add(button).clicked() {
                *active = tab;
            }

            ui.add_space(2.0);
        }
    });
}
