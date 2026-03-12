use egui::{ComboBox, RichText, Ui};

use crate::core::preferences::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ThemePreference,
    ThemeSchemeId,
};

use super::model::{ThemeColorField, ThemeEditorState};
use super::widgets::{section_heading, setting_row, thin_separator, toggle_switch};

/// Returns `true` if any preference was changed (needs persist).
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

    // ── Appearance ──────────────────────────────────────────────────────
    section_heading(ui, "Appearance");

    setting_row(ui, "Theme", |ui| {
        let current = theme_label(prefs.appearance.theme);
        ComboBox::from_id_salt("theme_combo")
            .selected_text(RichText::new(current).size(12.0))
            .width(110.0)
            .show_ui(ui, |ui| {
                for option in ThemePreference::ALL {
                    let label = theme_label(option);
                    if ui
                        .selectable_value(&mut prefs.appearance.theme, option, label)
                        .changed()
                    {
                        actions.changed = true;
                    }
                }
            });
    });

    thin_separator(ui);

    section_heading(ui, "Color Schemes");
    for scheme in ThemeSchemeId::ALL {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(scheme.label()).strong().size(12.0));
            ui.add_space(6.0);

            for field in ThemeColorField::ALL {
                setting_row(ui, field.label(), |ui| {
                    let mut value = theme_editor.theme_value(scheme, field).to_string();
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut value)
                            .desired_width(120.0)
                            .hint_text(field.placeholder()),
                    );

                    if response.changed() {
                        actions.theme_updates.push((scheme, field, value));
                    }
                });
            }
        });
        ui.add_space(8.0);
    }

    setting_row(ui, "Window Radius", |ui| {
        let current = radius_label(prefs.appearance.radius);
        ComboBox::from_id_salt("radius_combo")
            .selected_text(RichText::new(current).size(12.0))
            .width(110.0)
            .show_ui(ui, |ui| {
                for option in [
                    RadiusPreference::Small,
                    RadiusPreference::Medium,
                    RadiusPreference::Large,
                ] {
                    let label = radius_label(option);
                    if ui
                        .selectable_value(&mut prefs.appearance.radius, option, label)
                        .changed()
                    {
                        actions.changed = true;
                    }
                }
            });
    });

    ui.add_space(12.0);

    // ── Layout ──────────────────────────────────────────────────────────
    section_heading(ui, "Layout");

    setting_row(ui, "Window Size", |ui| {
        let current = size_label(prefs.layout.size);
        ComboBox::from_id_salt("size_combo")
            .selected_text(RichText::new(current).size(12.0))
            .width(110.0)
            .show_ui(ui, |ui| {
                for option in LauncherSize::ALL {
                    let label = size_label(option);
                    if ui
                        .selectable_value(&mut prefs.layout.size, option, label)
                        .changed()
                    {
                        actions.changed = true;
                    }
                }
            });
    });

    thin_separator(ui);

    setting_row(ui, "Window Location", |ui| {
        let current = placement_label(prefs.layout.placement);
        ComboBox::from_id_salt("placement_combo")
            .selected_text(RichText::new(current).size(12.0))
            .width(110.0)
            .show_ui(ui, |ui| {
                for &option in &[LauncherPlacement::Center, LauncherPlacement::RaisedCenter] {
                    let label = placement_label(option);
                    if ui
                        .selectable_value(&mut prefs.layout.placement, option, label)
                        .changed()
                    {
                        actions.changed = true;
                    }
                }
            });
    });

    ui.add_space(12.0);

    // ── System ──────────────────────────────────────────────────────────
    section_heading(ui, "System");

    setting_row(ui, "Start at Login", |ui| {
        if toggle_switch(ui, &mut prefs.system.start_at_login) {
            actions.changed = true;
        }
    });

    actions
}

fn theme_label(theme: ThemePreference) -> &'static str {
    theme.label()
}

fn radius_label(radius: RadiusPreference) -> &'static str {
    match radius {
        RadiusPreference::Custom => RadiusPreference::Medium.label(),
        _ => radius.label(),
    }
}

fn size_label(size: LauncherSize) -> &'static str {
    size.label()
}

fn placement_label(placement: LauncherPlacement) -> &'static str {
    match placement {
        LauncherPlacement::Center => "Center",
        LauncherPlacement::RaisedCenter | LauncherPlacement::Custom => "Elevated Center",
    }
}
