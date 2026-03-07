use super::model::AppPreferences;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const APP_CONFIG_DIR: &str = "gamut";
const PREFERENCES_FILE: &str = "preferences.toml";

pub fn load_preferences() -> AppPreferences {
    let path = preferences_path();
    load_preferences_from_path(&path).unwrap_or_default()
}

pub fn save_preferences(preferences: &AppPreferences) -> io::Result<()> {
    let path = preferences_path();
    save_preferences_to_path(preferences, &path)
}

pub fn preferences_path() -> PathBuf {
    config_home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(APP_CONFIG_DIR)
        .join(PREFERENCES_FILE)
}

fn load_preferences_from_path(path: &Path) -> io::Result<AppPreferences> {
    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(io::Error::other)
}

fn save_preferences_to_path(preferences: &AppPreferences, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(preferences).map_err(io::Error::other)?;
    fs::write(path, contents)
}

fn config_home_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path));
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}

#[cfg(test)]
mod tests {
    use super::{
        load_preferences_from_path, preferences_path, save_preferences_to_path, AppPreferences,
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
            .join("preferences.toml")
    }

    #[test]
    fn preferences_path_uses_expected_filename() {
        assert!(preferences_path().ends_with("gamut/preferences.toml"));
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
}
