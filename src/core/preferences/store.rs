use super::model::AppPreferences;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let temp_path = temporary_preferences_path(path);
    let mut file = temp_preferences_file(&temp_path)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    drop(file);

    fs::rename(&temp_path, path).or_else(|error| {
        let _ = fs::remove_file(&temp_path);
        Err(error)
    })
}

fn config_home_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path));
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}

fn temporary_preferences_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let suffix = format!("{}.{}.tmp", std::process::id(), nanos);

    match path.file_name().and_then(|name| name.to_str()) {
        Some(file_name) => path.with_file_name(format!("{file_name}.{suffix}")),
        None => path.with_extension(suffix),
    }
}

fn temp_preferences_file(path: &Path) -> io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        options.mode(0o600);
    }

    options.open(path)
}

#[cfg(test)]
mod tests {
    use super::{
        AppPreferences, load_preferences_from_path, preferences_path, save_preferences_to_path,
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

    #[test]
    fn save_overwrites_existing_preferences_atomically() {
        let path = unique_path();
        let mut first = AppPreferences::default();
        first.system.start_at_login = true;

        let mut second = AppPreferences::default();
        second.appearance.custom_radius = 24.0;

        save_preferences_to_path(&first, &path).expect("should write first preferences");
        save_preferences_to_path(&second, &path).expect("should replace preferences");

        let loaded = load_preferences_from_path(&path).expect("should load replaced preferences");
        assert_eq!(loaded, second);

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}
