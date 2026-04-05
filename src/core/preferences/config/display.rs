use super::ConfigKey;
use crate::core::preferences::{
    AppPreferences, ShortcutAction, ShortcutPreferences, ThemeColors, config_path, load_preferences,
};

pub(super) fn show_preferences() {
    let preferences = load_preferences();

    println!("Config file: {}", config_path().display());
    println!();
    println!("[appearance]");
    println!("theme = {}", preferences.appearance.theme);
    println!("radius = {}", preferences.appearance.radius);
    let active_theme = active_theme_colors(&preferences);
    println!("theme.background = {}", active_theme.background);
    println!("theme.text = {}", active_theme.text);
    println!("theme.accent = {}", active_theme.accent);

    if !preferences.appearance.themes.is_empty() {
        println!();
        for (name, colors) in &preferences.appearance.themes {
            println!("[appearance.themes.{name}]");
            println!("background = {}", colors.background);
            println!("text = {}", colors.text);
            println!("accent = {}", colors.accent);
            println!();
        }
    }

    println!();
    println!("[layout]");
    println!("size = {}", preferences.layout.size);
    println!("placement = {}", preferences.layout.placement);
    println!(
        "custom_top_margin = {}",
        trim_float(preferences.layout.custom_top_margin)
    );
    println!();
    println!("[shortcuts]");
    print_shortcuts(&preferences.shortcuts);
    println!();
    println!("[system]");
    println!("start_at_login = {}", preferences.system.start_at_login);
}

pub(super) fn print_shortcuts(shortcuts: &ShortcutPreferences) {
    for action in ShortcutAction::ALL {
        println!("{} = {}", action.config_key(), shortcuts.binding(action));
    }
}

pub(super) fn print_known_keys() {
    println!("Available config keys by section:");

    let mut current_section = "";

    for key in ConfigKey::ALL {
        if key.section() != current_section {
            if !current_section.is_empty() {
                println!();
            }

            current_section = key.section();
            println!("[{current_section}]");
        }

        println!("  - {key}");
        println!("    {}", key.description());
    }
}

pub(super) fn trim_float(value: f32) -> String {
    let rounded = format!("{value:.2}");
    rounded
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn active_theme_colors(preferences: &AppPreferences) -> ThemeColors {
    preferences.appearance.resolved_theme().colors
}
