use super::discovery::load_apps;
use super::model::DesktopApp;
use crate::core::storage::{app_cache_path, read_limited, write_atomic};
use bincode::Options;
use log::warn;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

const APP_CACHE_FILE: &str = "applications.bin";
const APP_CACHE_FORMAT_VERSION: u32 = 1;
const MAX_CACHE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
struct CachedAppCatalog {
    version: u32,
    apps: Vec<CachedDesktopApp>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedDesktopApp {
    name: String,
    exec_line: String,
    command: String,
    args: Vec<String>,
    icon_name: Option<String>,
    icon_categories: Vec<String>,
    icon_path: Option<PathBuf>,
}

pub fn load_cached_apps() -> Vec<DesktopApp> {
    let path = app_cache_path(APP_CACHE_FILE);
    match load_cached_apps_from_path(&path) {
        Ok(apps) => apps,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            warn!(
                "failed to load application cache from {}: {error}",
                path.display()
            );
            Vec::new()
        }
    }
}

pub fn refresh_app_cache() -> Vec<DesktopApp> {
    let apps = load_apps();
    let path = app_cache_path(APP_CACHE_FILE);

    if let Err(error) = save_cached_apps_to_path(&apps, &path) {
        warn!(
            "failed to save application cache to {}: {error}",
            path.display()
        );
    }

    apps
}

fn load_cached_apps_from_path(path: &Path) -> io::Result<Vec<DesktopApp>> {
    let bytes = read_limited(path, MAX_CACHE_BYTES)?;
    let cache: CachedAppCatalog = cache_codec()
        .deserialize(&bytes)
        .map_err(io::Error::other)?;

    if cache.version != APP_CACHE_FORMAT_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported application cache version {}; expected {}",
                cache.version, APP_CACHE_FORMAT_VERSION
            ),
        ));
    }

    Ok(cache.apps.into_iter().map(DesktopApp::from).collect())
}

fn save_cached_apps_to_path(apps: &[DesktopApp], path: &Path) -> io::Result<()> {
    let cache = CachedAppCatalog {
        version: APP_CACHE_FORMAT_VERSION,
        apps: apps.iter().cloned().map(CachedDesktopApp::from).collect(),
    };
    let bytes = cache_codec().serialize(&cache).map_err(io::Error::other)?;
    write_atomic(path, &bytes)
}

fn cache_codec() -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(MAX_CACHE_BYTES)
}

impl From<CachedDesktopApp> for DesktopApp {
    fn from(value: CachedDesktopApp) -> Self {
        DesktopApp::new(
            value.name,
            value.exec_line,
            value.command,
            value.args,
            value.icon_name,
            value.icon_categories,
            value.icon_path,
        )
    }
}

impl From<DesktopApp> for CachedDesktopApp {
    fn from(value: DesktopApp) -> Self {
        Self {
            name: value.name,
            exec_line: value.exec_line,
            command: value.command,
            args: value.args,
            icon_name: value.icon_name,
            icon_categories: value.icon_categories,
            icon_path: value.icon_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        APP_CACHE_FORMAT_VERSION, CachedAppCatalog, load_cached_apps_from_path,
        save_cached_apps_to_path,
    };
    use crate::core::desktop::DesktopApp;
    use bincode::Options;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();

        PathBuf::from("/tmp")
            .join(format!("gamut-app-cache-test-{nanos}"))
            .join("applications.bin")
    }

    fn sample_app() -> DesktopApp {
        DesktopApp::new(
            "Firefox".to_string(),
            "/usr/bin/firefox %u".to_string(),
            "/usr/bin/firefox".to_string(),
            vec!["%u".to_string()],
            Some("firefox".to_string()),
            vec!["Network".to_string()],
            Some(PathBuf::from("/tmp/firefox.png")),
        )
    }

    #[test]
    fn cache_round_trip_preserves_application_catalog() {
        let path = unique_path();
        let apps = vec![sample_app()];

        save_cached_apps_to_path(&apps, &path).expect("should save app cache");
        let loaded = load_cached_apps_from_path(&path).expect("should load app cache");

        assert_eq!(loaded, apps);

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn cache_rejects_unknown_format_version() {
        let path = unique_path();
        let payload = CachedAppCatalog {
            version: APP_CACHE_FORMAT_VERSION + 1,
            apps: Vec::new(),
        };
        let bytes = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&payload)
            .expect("should serialize incompatible payload");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("should create parent dir");
        }
        fs::write(&path, bytes).expect("should write incompatible cache payload");

        let error =
            load_cached_apps_from_path(&path).expect_err("should reject incompatible cache");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}
