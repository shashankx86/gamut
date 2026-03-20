mod interactive;
mod key;

pub use key::ConfigKey;

use super::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ShortcutAction,
    ShortcutBinding, ShortcutPreferences, ThemeColors, ThemePreference, config_path,
    load_preferences, save_preferences,
};
use crate::core::ipc::{IpcCommand, send_command};
use crate::core::theme::default_custom_theme_colors;
use std::error::Error;
use std::io;

type DynError = Box<dyn Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigCommand {
    Show,
    Path,
    Keys,
    Get { key: ConfigKey },
    Set { key: ConfigKey, value: String },
    Reset { target: ConfigResetTarget },
    Shortcut(ShortcutConfigCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigResetTarget {
    All,
    Key(ConfigKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutConfigCommand {
    List,
    Set {
        action: ShortcutAction,
        binding: String,
    },
    Interactive {
        action: Option<ShortcutAction>,
    },
}

pub fn execute(command: ConfigCommand) -> Result<(), DynError> {
    match command {
        ConfigCommand::Show => {
            show_preferences();
            Ok(())
        }
        ConfigCommand::Path => {
            println!("{}", config_path().display());
            Ok(())
        }
        ConfigCommand::Keys => {
            print_known_keys();
            Ok(())
        }
        ConfigCommand::Get { key } => {
            let preferences = load_preferences();
            println!("{} = {}", key, key.value(&preferences));
            Ok(())
        }
        ConfigCommand::Set { key, value } => {
            let mut preferences = load_preferences();
            key.set_value(&mut preferences, &value)?;
            persist_preferences(&preferences)?;
            println!("Updated {} = {}", key, key.value(&preferences));
            Ok(())
        }
        ConfigCommand::Reset { target } => {
            let mut preferences = load_preferences();

            match target {
                ConfigResetTarget::Key(key) => {
                    key.reset_value(&mut preferences);
                    persist_preferences(&preferences)?;
                    println!("Reset {} = {}", key, key.value(&preferences));
                }
                ConfigResetTarget::All => {
                    let preferences = AppPreferences::default();
                    persist_preferences(&preferences)?;
                    println!("Reset all configuration to defaults.");
                }
            }

            Ok(())
        }
        ConfigCommand::Shortcut(command) => execute_shortcut_command(command),
    }
}

fn execute_shortcut_command(command: ShortcutConfigCommand) -> Result<(), DynError> {
    match command {
        ShortcutConfigCommand::List => {
            print_shortcuts(&load_preferences().shortcuts);
            Ok(())
        }
        ShortcutConfigCommand::Set { action, binding } => {
            let mut preferences = load_preferences();
            let binding = binding.parse::<ShortcutBinding>()?;
            preferences.shortcuts.update_binding(action, binding)?;
            persist_preferences(&preferences)?;
            println!(
                "Updated {} = {}",
                action.config_key(),
                preferences.shortcuts.binding(action)
            );
            Ok(())
        }
        ShortcutConfigCommand::Interactive { action } => {
            let mut preferences = load_preferences();

            if interactive::configure_shortcuts(&mut preferences.shortcuts, action)? {
                persist_preferences(&preferences)?;
            }

            Ok(())
        }
    }
}

fn persist_preferences(preferences: &AppPreferences) -> Result<(), DynError> {
    save_preferences(preferences)?;

    if let Err(error) = send_command(IpcCommand::ReloadPreferences)
        && should_warn_about_reload(&error)
    {
        eprintln!("warning: saved config but could not notify daemon: {error}");
    }

    Ok(())
}

fn should_warn_about_reload(error: &io::Error) -> bool {
    !matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused | io::ErrorKind::ConnectionReset
    )
}

fn show_preferences() {
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

fn print_shortcuts(shortcuts: &ShortcutPreferences) {
    for action in ShortcutAction::ALL {
        println!("{} = {}", action.config_key(), shortcuts.binding(action));
    }
}

fn print_known_keys() {
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

fn trim_float(value: f32) -> String {
    let rounded = format!("{value:.2}");
    rounded
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        _ => Err("expected a boolean: true/false, yes/no, on/off, 1/0".to_string()),
    }
}

fn parse_non_negative_f32(value: &str, label: &str) -> Result<f32, String> {
    let parsed = value
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("{label} must be a number"))?;

    if !parsed.is_finite() {
        return Err(format!("{label} must be finite"));
    }

    if parsed < 0.0 {
        return Err(format!("{label} must be non-negative"));
    }

    Ok(parsed)
}

impl ConfigKey {
    pub fn value(self, preferences: &AppPreferences) -> String {
        match self {
            Self::AppearanceTheme => preferences.appearance.theme.to_string(),
            Self::AppearanceRadius => preferences.appearance.radius.to_string(),
            Self::AppearanceThemeBackground => active_theme_colors(preferences).background,
            Self::AppearanceThemeText => active_theme_colors(preferences).text,
            Self::AppearanceThemeAccent => active_theme_colors(preferences).accent,
            Self::LayoutSize => preferences.layout.size.to_string(),
            Self::LayoutPlacement => preferences.layout.placement.to_string(),
            Self::LayoutCustomTopMargin => trim_float(preferences.layout.custom_top_margin),
            Self::Shortcut(action) => preferences.shortcuts.binding(action).to_string(),
            Self::SystemStartAtLogin => preferences.system.start_at_login.to_string(),
        }
    }

    pub fn set_value(self, preferences: &mut AppPreferences, value: &str) -> Result<(), String> {
        match self {
            Self::AppearanceTheme => {
                preferences.appearance.theme = value.parse::<ThemePreference>()?;

                if let ThemePreference::Custom(name) = preferences.appearance.theme.clone() {
                    preferences.appearance.upsert_custom_theme(&name)?;
                }
            }
            Self::AppearanceRadius => {
                preferences.appearance.radius = value.parse::<RadiusPreference>()?;
            }
            Self::AppearanceThemeBackground => {
                selected_custom_theme_mut(preferences)?.background = parse_color(value)?;
            }
            Self::AppearanceThemeText => {
                selected_custom_theme_mut(preferences)?.text = parse_color(value)?;
            }
            Self::AppearanceThemeAccent => {
                selected_custom_theme_mut(preferences)?.accent = parse_color(value)?;
            }
            Self::LayoutSize => {
                preferences.layout.size = value.parse::<LauncherSize>()?;
            }
            Self::LayoutPlacement => {
                preferences.layout.placement = value.parse::<LauncherPlacement>()?;
            }
            Self::LayoutCustomTopMargin => {
                preferences.layout.custom_top_margin =
                    parse_non_negative_f32(value, "layout.custom_top_margin")?;
            }
            Self::Shortcut(action) => {
                preferences
                    .shortcuts
                    .update_binding(action, value.parse::<ShortcutBinding>()?)?;
            }
            Self::SystemStartAtLogin => {
                preferences.system.start_at_login = parse_bool(value)?;
            }
        }

        Ok(())
    }

    pub fn reset_value(self, preferences: &mut AppPreferences) {
        let defaults = AppPreferences::default();

        match self {
            Self::AppearanceTheme => preferences.appearance.theme = defaults.appearance.theme,
            Self::AppearanceRadius => preferences.appearance.radius = defaults.appearance.radius,
            Self::AppearanceThemeBackground => {
                if let Ok(theme) = selected_custom_theme_mut(preferences) {
                    theme.background = default_custom_theme_colors().background;
                }
            }
            Self::AppearanceThemeText => {
                if let Ok(theme) = selected_custom_theme_mut(preferences) {
                    theme.text = default_custom_theme_colors().text;
                }
            }
            Self::AppearanceThemeAccent => {
                if let Ok(theme) = selected_custom_theme_mut(preferences) {
                    theme.accent = default_custom_theme_colors().accent;
                }
            }
            Self::LayoutSize => preferences.layout.size = defaults.layout.size,
            Self::LayoutPlacement => preferences.layout.placement = defaults.layout.placement,
            Self::LayoutCustomTopMargin => {
                preferences.layout.custom_top_margin = defaults.layout.custom_top_margin;
            }
            Self::Shortcut(action) => {
                let binding = defaults.shortcuts.binding(action).clone();
                preferences
                    .shortcuts
                    .update_binding(action, binding)
                    .expect("default shortcut binding should apply");
            }
            Self::SystemStartAtLogin => {
                preferences.system.start_at_login = defaults.system.start_at_login;
            }
        }
    }
}

fn active_theme_colors(preferences: &AppPreferences) -> ThemeColors {
    preferences.appearance.resolved_theme().colors
}

fn selected_custom_theme_mut(preferences: &mut AppPreferences) -> Result<&mut ThemeColors, String> {
    let ThemePreference::Custom(name) = preferences.appearance.theme.clone() else {
        return Err(
            "appearance.theme colors can only be changed for a custom theme; set appearance.theme to a name like `orange` first"
                .to_string(),
        );
    };

    preferences.appearance.upsert_custom_theme(&name)
}

fn parse_color(value: &str) -> Result<String, String> {
    super::normalize_hex_color(value)
        .ok_or_else(|| "expected a 6 or 8 digit hex color like #151516 or #151516FF".to_string())
}

#[cfg(test)]
mod tests {
    use super::{ConfigKey, parse_bool, parse_non_negative_f32};
    use crate::core::preferences::AppPreferences;
    use std::str::FromStr;

    #[test]
    fn bool_parser_accepts_common_values() {
        assert!(parse_bool("yes").expect("yes should parse"));
        assert!(!parse_bool("off").expect("off should parse"));
    }

    #[test]
    fn numeric_parser_rejects_negative_values() {
        assert!(parse_non_negative_f32("-1", "test").is_err());
    }

    #[test]
    fn config_key_updates_shortcuts() {
        let mut preferences = AppPreferences::default();

        ConfigKey::from_str("shortcuts.move_up")
            .expect("key should parse")
            .set_value(&mut preferences, "Ctrl+K")
            .expect("shortcut should update");

        assert_eq!(preferences.shortcuts.move_up.to_string(), "Ctrl+K");
    }

    #[test]
    fn setting_custom_theme_creates_theme_entry() {
        let mut preferences = AppPreferences::default();

        ConfigKey::AppearanceTheme
            .set_value(&mut preferences, "orange")
            .expect("theme should update");

        assert!(preferences.appearance.themes.contains_key("orange"));
    }
}
