use super::display::{print_known_keys, print_shortcuts, show_preferences};
use super::interactive;
use super::{ConfigCommand, ConfigResetTarget, ShortcutConfigCommand};
use crate::core::error::DynError;
use crate::core::ipc::{IpcCommand, send_command};
use crate::core::preferences::{
    AppPreferences, ShortcutBinding, config_path, load_preferences, save_preferences,
};
use std::io;

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
