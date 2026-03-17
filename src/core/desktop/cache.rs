use super::discovery::load_apps;
use super::model::DesktopApp;
use crate::core::storage::{app_cache_path, read_limited, write_atomic};
use bincode::Options;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APP_CACHE_FILE: &str = "applications.bin";
const APP_CACHE_FORMAT_VERSION: u32 = 2;
const MAX_CACHE_BYTES: u64 = 16 * 1024 * 1024;
const APP_CACHE_STALE_AFTER: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug)]
pub struct CachedAppCatalogState {
    pub apps: Vec<DesktopApp>,
    pub needs_refresh: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedAppCatalog {
    version: u32,
    catalog_refreshed_at_secs: u64,
    apps: Vec<CachedDesktopApp>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedDesktopApp {
    name: String,
    entry_type: String,
    exec_line: String,
    command: String,
    args: Vec<String>,
    icon_name: Option<String>,
    icon_categories: Vec<String>,
    icon_path: Option<PathBuf>,
}

pub fn load_cached_apps() -> Vec<DesktopApp> {
    load_cached_app_catalog().apps
}

pub fn load_cached_app_catalog() -> CachedAppCatalogState {
    let path = app_cache_path(APP_CACHE_FILE);
    match load_cached_catalog_from_path(&path) {
        Ok(cache) => {
            let apps: Vec<DesktopApp> = cache.apps.into_iter().map(DesktopApp::from).collect();
            let needs_refresh = apps.is_empty() || is_cache_stale(cache.catalog_refreshed_at_secs);
            CachedAppCatalogState {
                apps,
                needs_refresh,
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => CachedAppCatalogState {
            apps: Vec::new(),
            needs_refresh: true,
        },
        Err(error) => {
            warn!(
                "failed to load application cache from {}: {error}",
                path.display()
            );
            CachedAppCatalogState {
                apps: Vec::new(),
                needs_refresh: true,
            }
        }
    }
}

pub fn refresh_app_cache() -> Vec<DesktopApp> {
    let mut apps = load_apps();
    let path = app_cache_path(APP_CACHE_FILE);
    let cached_apps = load_cached_apps();
    merge_cached_icon_paths(&mut apps, &cached_apps);

    if let Err(error) = save_cached_apps_to_path(&apps, &path, current_timestamp_secs()) {
        warn!(
            "failed to save application cache to {}: {error}",
            path.display()
        );
    }

    apps
}

pub fn save_cached_apps(apps: &[DesktopApp]) -> io::Result<()> {
    let path = app_cache_path(APP_CACHE_FILE);
    let refreshed_at_secs = load_cached_catalog_from_path(&path)
        .map(|cache| cache.catalog_refreshed_at_secs)
        .unwrap_or_else(|_| current_timestamp_secs());
    save_cached_apps_to_path(apps, &path, refreshed_at_secs)
}

#[cfg(test)]
fn load_cached_apps_from_path(path: &Path) -> io::Result<Vec<DesktopApp>> {
    load_cached_catalog_from_path(path)
        .map(|cache| cache.apps.into_iter().map(DesktopApp::from).collect())
}

fn load_cached_catalog_from_path(path: &Path) -> io::Result<CachedAppCatalog> {
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

    Ok(cache)
}

fn save_cached_apps_to_path(
    apps: &[DesktopApp],
    path: &Path,
    catalog_refreshed_at_secs: u64,
) -> io::Result<()> {
    let cache = CachedAppCatalog {
        version: APP_CACHE_FORMAT_VERSION,
        catalog_refreshed_at_secs,
        apps: apps.iter().cloned().map(CachedDesktopApp::from).collect(),
    };
    let bytes = cache_codec().serialize(&cache).map_err(io::Error::other)?;
    write_atomic(path, &bytes)
}

fn merge_cached_icon_paths(apps: &mut [DesktopApp], cached_apps: &[DesktopApp]) {
    let cached_icons: HashMap<(String, String), PathBuf> = cached_apps
        .iter()
        .filter_map(|app| {
            app.icon_path
                .as_ref()
                .map(|path| (app_cache_key(app), path.clone()))
        })
        .collect();

    for app in apps {
        if app.icon_path.is_some() {
            continue;
        }

        if let Some(icon_path) = cached_icons.get(&app_cache_key(app)) {
            app.icon_path = Some(icon_path.clone());
        }
    }
}

fn app_cache_key(app: &DesktopApp) -> (String, String) {
    (app.name.to_lowercase(), app.command.to_lowercase())
}

fn is_cache_stale(catalog_refreshed_at_secs: u64) -> bool {
    if catalog_refreshed_at_secs == 0 {
        return true;
    }

    current_timestamp_secs().saturating_sub(catalog_refreshed_at_secs)
        >= APP_CACHE_STALE_AFTER.as_secs()
}

fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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
            value.entry_type,
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
            entry_type: value.entry_type,
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
        current_timestamp_secs, load_cached_apps_from_path, save_cached_apps_to_path,
        CachedAppCatalog, APP_CACHE_FORMAT_VERSION,
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
            "Application".to_string(),
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

        save_cached_apps_to_path(&apps, &path, current_timestamp_secs())
            .expect("should save app cache");
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
            catalog_refreshed_at_secs: current_timestamp_secs(),
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

    #[test]
    fn cache_round_trip_preserves_refresh_timestamp() {
        let path = unique_path();
        let timestamp = 1_730_000_000;

        save_cached_apps_to_path(&[sample_app()], &path, timestamp).expect("should save app cache");
        let cache = super::load_cached_catalog_from_path(&path).expect("should load raw cache");

        assert_eq!(cache.catalog_refreshed_at_secs, timestamp);

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}
