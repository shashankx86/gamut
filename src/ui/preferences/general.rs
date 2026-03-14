use egui::{RichText, Ui};
use lucide_icons::Icon;

use crate::core::preferences::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ThemePreference,
    ThemeSchemeId,
};

use super::model::{ThemeColorField, ThemeEditorState};
use super::theme;
use super::widgets::{
    color_swatch, section_card, segmented_control, setting_row, theme_card, thin_separator,
    toggle_switch,
};

pub struct GeneralActions {
    pub changed: bool,
    pub theme_updates: Vec<(ThemeSchemeId, ThemeColorField, String)>,
}

pub fn render_general(
    ui: &mut Ui,
    prefs: &mut AppPreferences,
    theme_editor: &ThemeEditorState,
) -> GeneralActions {
    let mut actions = GeneralActions {
        changed: false,
        theme_updates: Vec::new(),
    };

    // ── Appearance ─────────────────────────────────────────────────────
    section_card(ui, Icon::Palette, "Appearance", |ui| {
        // Theme choice cards
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for option in ThemePreference::ALL {
                let icon = match option {
                    ThemePreference::System => Icon::SunMoon,
                    ThemePreference::Light => Icon::Sun,
                    ThemePreference::Dark => Icon::Moon,
                };
                if theme_card(ui, icon, option.label(), prefs.appearance.theme == option) {
                    prefs.appearance.theme = option;
                    actions.changed = true;
                }
            }
        });

        ui.add_space(8.0);
        thin_separator(ui);
        ui.add_space(4.0);

        // Window Radius
        setting_row(ui, "Window Radius", |ui| {
            let radius_options = [
                RadiusPreference::Small,
                RadiusPreference::Medium,
                RadiusPreference::Large,
            ];
            let radius_index = radius_options
                .iter()
                .position(|&r| r == prefs.appearance.radius)
                .unwrap_or(0);

            if let Some(new_idx) =
                segmented_control(ui, radius_index, &["Small", "Medium", "Large"])
            {
                prefs.appearance.radius = radius_options[new_idx];
                actions.changed = true;
            }
        });
    });

    // ── Color Schemes ──────────────────────────────────────────────────
    section_card(ui, Icon::Paintbrush, "Color Schemes", |ui| {
        for scheme in ThemeSchemeId::ALL {
            let tokens = theme::tokens(ui);
            ui.label(
                RichText::new(scheme.label())
                    .size(11.5)
                    .color(tokens.text_primary)
                    .strong(),
            );
            ui.add_space(4.0);

            for field in ThemeColorField::ALL {
                let tokens = theme::tokens(ui);
                ui.horizontal(|ui| {
                    ui.set_min_height(28.0);
                    ui.label(
                        RichText::new(field.label())
                            .size(12.0)
                            .color(tokens.text_secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut value = theme_editor.theme_value(scheme, field).to_string();
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut value)
                                .desired_width(100.0)
                                .hint_text(field.placeholder()),
                        );
                        color_swatch(ui, theme_editor.theme_value(scheme, field));

                        if response.changed() {
                            actions.theme_updates.push((scheme, field, value));
                        }
                    });
                });
            }

            ui.add_space(8.0);
        }
    });

    // ── Layout ─────────────────────────────────────────────────────────
    section_card(ui, Icon::AppWindow, "Layout", |ui| {
        setting_row(ui, "Window Size", |ui| {
            let size_index = LauncherSize::ALL
                .iter()
                .position(|&s| s == prefs.layout.size)
                .unwrap_or(1);

            let labels: Vec<&str> = LauncherSize::ALL.iter().map(|s| s.label()).collect();
            if let Some(new_idx) = segmented_control(ui, size_index, &labels) {
                prefs.layout.size = LauncherSize::ALL[new_idx];
                actions.changed = true;
            }
        });

        thin_separator(ui);

        setting_row(ui, "Position", |ui| {
            let placement_options = [LauncherPlacement::Center, LauncherPlacement::RaisedCenter];
            let placement_labels = ["Center", "Elevated"];
            let placement_index = placement_options
                .iter()
                .position(|&p| p == prefs.layout.placement)
                .unwrap_or(1);

            if let Some(new_idx) = segmented_control(ui, placement_index, &placement_labels) {
                prefs.layout.placement = placement_options[new_idx];
                actions.changed = true;
            }
        });
    });

    // ── System ─────────────────────────────────────────────────────────
    section_card(ui, Icon::MonitorCog, "System", |ui| {
        setting_row(ui, "Start at Login", |ui| {
            if toggle_switch(ui, &mut prefs.system.start_at_login) {
                actions.changed = true;
            }
        });
    });

    actions
}
