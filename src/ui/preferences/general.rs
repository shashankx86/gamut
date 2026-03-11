use egui::{ComboBox, RichText, Ui};

use crate::core::preferences::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ThemePreference,
};

use super::widgets::{section_heading, setting_row, thin_separator, toggle_switch};

/// Returns `true` if any preference was changed (needs persist).
pub fn render_general(ui: &mut Ui, prefs: &mut AppPreferences) -> bool {
    let mut changed = false;

    // ── Appearance ──────────────────────────────────────────────────────
    section_heading(ui, "Appearance");

    setting_row(ui, "Theme", |ui| {
        let current = theme_label(prefs.appearance.theme);
        ComboBox::from_id_salt("theme_combo")
            .selected_text(RichText::new(current).size(12.0))
            .width(110.0)
            .show_ui(ui, |ui| {
                for &option in &[
                    ThemePreference::System,
                    ThemePreference::Light,
                    ThemePreference::Dark,
                ] {
                    let label = theme_label(option);
                    if ui
                        .selectable_value(&mut prefs.appearance.theme, option, label)
                        .changed()
                    {
                        changed = true;
                    }
                }
            });
    });

    thin_separator(ui);

    setting_row(ui, "Window Radius", |ui| {
        let current = radius_label(prefs.appearance.radius);
        ComboBox::from_id_salt("radius_combo")
            .selected_text(RichText::new(current).size(12.0))
            .width(110.0)
            .show_ui(ui, |ui| {
                for &option in &[
                    RadiusPreference::Small,
                    RadiusPreference::Medium,
                    RadiusPreference::Large,
                ] {
                    let label = radius_label(option);
                    if ui
                        .selectable_value(&mut prefs.appearance.radius, option, label)
                        .changed()
                    {
                        changed = true;
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
                for &option in &[
                    LauncherSize::Small,
                    LauncherSize::Medium,
                    LauncherSize::Large,
                    LauncherSize::ExtraLarge,
                ] {
                    let label = size_label(option);
                    if ui
                        .selectable_value(&mut prefs.layout.size, option, label)
                        .changed()
                    {
                        changed = true;
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
                        changed = true;
                    }
                }
            });
    });

    ui.add_space(12.0);

    // ── System ──────────────────────────────────────────────────────────
    section_heading(ui, "System");

    setting_row(ui, "Start at Login", |ui| {
        if toggle_switch(ui, &mut prefs.system.start_at_login) {
            changed = true;
        }
    });

    changed
}

fn theme_label(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "Light",
        ThemePreference::Dark => "Dark",
        ThemePreference::System | ThemePreference::Custom => "System",
    }
}

fn radius_label(radius: RadiusPreference) -> &'static str {
    match radius {
        RadiusPreference::Small => "Small",
        RadiusPreference::Medium | RadiusPreference::Custom => "Medium",
        RadiusPreference::Large => "Large",
    }
}

fn size_label(size: LauncherSize) -> &'static str {
    match size {
        LauncherSize::Small => "Small",
        LauncherSize::Medium => "Medium",
        LauncherSize::Large => "Large",
        LauncherSize::ExtraLarge => "Extra Large",
    }
}

fn placement_label(placement: LauncherPlacement) -> &'static str {
    match placement {
        LauncherPlacement::Center => "Center",
        LauncherPlacement::RaisedCenter | LauncherPlacement::Custom => "Above Center",
    }
}
