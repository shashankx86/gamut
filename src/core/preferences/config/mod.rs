mod interactive;
mod key;

pub use key::ConfigKey;

use super::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ShortcutAction,
    ShortcutBinding, ShortcutPreferences, ThemePreference, ThemeSchemeId, config_path,
    load_preferences, save_preferences,
};
use crate::core::ipc::{IpcCommand, send_command};
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
    println!(
        "custom_radius = {}",
        trim_float(preferences.appearance.custom_radius)
    );
    println!(
        "schemes.light.background = {}",
        preferences.appearance.schemes.light.background
    );
    println!(
        "schemes.light.text = {}",
        preferences.appearance.schemes.light.text
    );
    println!(
        "schemes.light.accent = {}",
        preferences.appearance.schemes.light.accent
    );
    println!(
        "schemes.dark.background = {}",
        preferences.appearance.schemes.dark.background
    );
    println!(
        "schemes.dark.text = {}",
        preferences.appearance.schemes.dark.text
    );
    println!(
        "schemes.dark.accent = {}",
        preferences.appearance.schemes.dark.accent
    );
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
            Self::AppearanceCustomRadius => trim_float(preferences.appearance.custom_radius),
            Self::AppearanceLightBackground => {
                preferences.appearance.schemes.light.background.clone()
            }
            Self::AppearanceLightText => preferences.appearance.schemes.light.text.clone(),
            Self::AppearanceLightAccent => preferences.appearance.schemes.light.accent.clone(),
            Self::AppearanceDarkBackground => {
                preferences.appearance.schemes.dark.background.clone()
            }
            Self::AppearanceDarkText => preferences.appearance.schemes.dark.text.clone(),
            Self::AppearanceDarkAccent => preferences.appearance.schemes.dark.accent.clone(),
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
            }
            Self::AppearanceRadius => {
                preferences.appearance.radius = value.parse::<RadiusPreference>()?;
            }
            Self::AppearanceCustomRadius => {
                preferences.appearance.custom_radius =
                    parse_non_negative_f32(value, "appearance.custom_radius")?;
            }
            Self::AppearanceLightBackground => {
                preferences
                    .appearance
                    .scheme_mut(ThemeSchemeId::Light)
                    .background = parse_color(value)?;
            }
            Self::AppearanceLightText => {
                preferences.appearance.scheme_mut(ThemeSchemeId::Light).text = parse_color(value)?;
            }
            Self::AppearanceLightAccent => {
                preferences
                    .appearance
                    .scheme_mut(ThemeSchemeId::Light)
                    .accent = parse_color(value)?;
            }
            Self::AppearanceDarkBackground => {
                preferences
                    .appearance
                    .scheme_mut(ThemeSchemeId::Dark)
                    .background = parse_color(value)?;
            }
            Self::AppearanceDarkText => {
                preferences.appearance.scheme_mut(ThemeSchemeId::Dark).text = parse_color(value)?;
            }
            Self::AppearanceDarkAccent => {
                preferences
                    .appearance
                    .scheme_mut(ThemeSchemeId::Dark)
                    .accent = parse_color(value)?;
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
            Self::AppearanceCustomRadius => {
                preferences.appearance.custom_radius = defaults.appearance.custom_radius;
            }
            Self::AppearanceLightBackground => {
                preferences.appearance.schemes.light.background =
                    defaults.appearance.schemes.light.background;
            }
            Self::AppearanceLightText => {
                preferences.appearance.schemes.light.text = defaults.appearance.schemes.light.text;
            }
            Self::AppearanceLightAccent => {
                preferences.appearance.schemes.light.accent =
                    defaults.appearance.schemes.light.accent;
            }
            Self::AppearanceDarkBackground => {
                preferences.appearance.schemes.dark.background =
                    defaults.appearance.schemes.dark.background;
            }
            Self::AppearanceDarkText => {
                preferences.appearance.schemes.dark.text = defaults.appearance.schemes.dark.text;
            }
            Self::AppearanceDarkAccent => {
                preferences.appearance.schemes.dark.accent =
                    defaults.appearance.schemes.dark.accent;
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
                    .expect("default shortcut bindings should be unique");
            }
            Self::SystemStartAtLogin => {
                preferences.system.start_at_login = defaults.system.start_at_login;
            }
        }
    }
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
}
