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

/// Returns `true` if the "Reset All" button was clicked.
pub fn render_sidebar(ui: &mut Ui, active: &mut PreferencesTab) -> bool {
    let mut reset_clicked = false;

    ui.vertical(|ui| {
        ui.add_space(8.0);
        let tokens = theme::tokens(ui);

        for tab in PreferencesTab::ALL {
            let is_active = *active == tab;

            let bg = if is_active {
                tokens.accent_dim
            } else {
                Color32::TRANSPARENT
            };
            let text_color = if is_active {
                tokens.text_primary
            } else {
                tokens.muted
            };

            let button = egui::Button::new(
                RichText::new(format!("{}  {}", tab.icon(), tab.label()))
                    .size(12.0)
                    .color(text_color),
            )
            .fill(bg)
            .stroke(if is_active {
                Stroke::new(1.0, tokens.accent)
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

        // Push reset button to the bottom of the sidebar.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(12.0);

            let reset_button = egui::Button::new(
                RichText::new("Reset All Settings")
                    .size(11.0)
                    .color(tokens.muted),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, tokens.border))
            .corner_radius(4)
            .min_size(Vec2::new(ui.available_width(), 28.0));

            if ui.add(reset_button).clicked() {
                reset_clicked = true;
            }
        });
    });

    reset_clicked
}
