use super::model::AppPreferences;
use crate::core::storage::app_config_path;
use crate::core::storage::write_atomic;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "config.toml";

pub fn load_preferences() -> AppPreferences {
    load_or_create_preferences_at_path(&config_path())
}

pub fn save_preferences(preferences: &AppPreferences) -> io::Result<()> {
    let path = config_path();
    save_preferences_to_path(preferences, &path)
}

pub fn config_path() -> PathBuf {
    app_config_path(CONFIG_FILE)
}

fn load_preferences_from_path(path: &Path) -> io::Result<AppPreferences> {
    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(io::Error::other)
}

fn load_or_create_preferences_at_path(path: &Path) -> AppPreferences {
    match load_preferences_from_path(path) {
        Ok(preferences) => preferences,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let preferences = AppPreferences::default();

            if let Err(save_error) = save_preferences_to_path(&preferences, path) {
                eprintln!(
                    "warning: could not create default config at {}: {save_error}",
                    path.display()
                );
            }

            preferences
        }
        Err(_) => AppPreferences::default(),
    }
}

fn save_preferences_to_path(preferences: &AppPreferences, path: &Path) -> io::Result<()> {
    let contents = toml::to_string_pretty(preferences).map_err(io::Error::other)?;
    write_atomic(path, contents.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{
        config_path, load_or_create_preferences_at_path, load_preferences_from_path,
        save_preferences_to_path, AppPreferences,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        PathBuf::from("/tmp")
            .join(format!("gamut-preferences-test-{nanos}"))
            .join("config.toml")
    }

    #[test]
    fn config_path_uses_expected_filename() {
        assert!(config_path().ends_with("gamut/config.toml"));
    }

    #[test]
    fn save_and_load_round_trip_preferences() {
        let path = unique_path();
        let preferences = AppPreferences::default();

        save_preferences_to_path(&preferences, &path).expect("should save preferences");
        let loaded = load_preferences_from_path(&path).expect("should load preferences");

        assert_eq!(loaded, preferences);

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn save_overwrites_existing_preferences_atomically() {
        let path = unique_path();
        let mut first = AppPreferences::default();
        first.system.start_at_login = true;

        let mut second = AppPreferences::default();
        second.layout.custom_top_margin = 96.0;

        save_preferences_to_path(&first, &path).expect("should write first preferences");
        save_preferences_to_path(&second, &path).expect("should replace preferences");

        let loaded = load_preferences_from_path(&path).expect("should load replaced preferences");
        assert_eq!(loaded, second);

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn load_creates_default_config_when_missing() {
        let path = unique_path();

        let loaded = load_or_create_preferences_at_path(&path);
        let saved = load_preferences_from_path(&path).expect("should create config file");

        assert_eq!(loaded, AppPreferences::default());
        assert_eq!(saved, AppPreferences::default());

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}
