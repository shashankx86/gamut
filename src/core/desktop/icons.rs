use super::model::IconResolveRequest;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const GENERIC_ICON_CACHE_KEY: &str = "__generic_app_icon__";
const PREFERRED_THEMES: &[&str] = &["breeze-dark", "breeze", "Adwaita", "hicolor"];

pub(super) fn resolve_app_icon(request: &IconResolveRequest) -> Option<PathBuf> {
    let mut icon_cache = icon_lookup_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    resolve_icon_path(request.icon_name.as_deref(), &mut icon_cache)
        .or_else(|| {
            resolve_context_icon_path(
                &request.icon_categories,
                &request.name,
                &request.exec_line,
                &mut icon_cache,
            )
        })
        .or_else(|| resolve_generic_icon_path(&mut icon_cache))
}

fn resolve_icon_path(
    icon: Option<&str>,
    icon_cache: &mut HashMap<String, Option<PathBuf>>,
) -> Option<PathBuf> {
    let icon = icon?.trim();
    if icon.is_empty() {
        return None;
    }

    if let Some(cached) = icon_cache.get(icon) {
        return cached.clone();
    }

    let resolved = find_icon_file(icon);
    icon_cache.insert(icon.to_string(), resolved.clone());
    resolved
}

fn resolve_generic_icon_path(icon_cache: &mut HashMap<String, Option<PathBuf>>) -> Option<PathBuf> {
    if let Some(cached) = icon_cache.get(GENERIC_ICON_CACHE_KEY) {
        return cached.clone();
    }

    let resolved = [
        "application-x-executable",
        "application-default-icon",
        "application-default",
        "application-x-desktop",
        "application",
    ]
    .into_iter()
    .find_map(find_icon_file);

    icon_cache.insert(GENERIC_ICON_CACHE_KEY.to_string(), resolved.clone());
    resolved
}

fn resolve_context_icon_path(
    categories: &[String],
    name: &str,
    exec_line: &str,
    icon_cache: &mut HashMap<String, Option<PathBuf>>,
) -> Option<PathBuf> {
    let fallback_names = context_icon_candidates(categories, name, exec_line);

    fallback_names
        .into_iter()
        .find_map(|icon_name| resolve_named_icon(icon_name, icon_cache))
}

fn context_icon_candidates(
    categories: &[String],
    name: &str,
    exec_line: &str,
) -> Vec<&'static str> {
    let mut names = Vec::new();

    let has_category = |wanted: &str| {
        categories
            .iter()
            .any(|category| category.eq_ignore_ascii_case(wanted))
    };

    if has_category("Printing") {
        names.extend(["printer", "printer-network"]);
    }
    if has_category("Scanner") {
        names.extend(["scanner", "scanner-photo"]);
    }
    if has_category("Settings") || has_category("System") || has_category("HardwareSettings") {
        names.extend(["preferences-system", "applications-system"]);
    }
    if has_category("Network") {
        names.extend(["network-workgroup", "applications-internet"]);
    }
    if has_category("Office") {
        names.push("applications-office");
    }
    if has_category("Graphics") {
        names.push("applications-graphics");
    }
    if has_category("AudioVideo") {
        names.push("applications-multimedia");
    }
    if has_category("Development") {
        names.push("applications-development");
    }
    if has_category("Utility") {
        names.push("applications-utilities");
    }

    let low_name = name.to_lowercase();
    let low_exec = exec_line.to_lowercase();
    if low_name.contains("printer")
        || low_exec.contains("printer")
        || low_name.contains("hplip")
        || low_exec.contains("hplip")
        || low_name.contains("hp device")
        || low_exec.contains("hp-")
    {
        names.extend(["printer", "printer-network"]);
    }

    dedupe_names(names)
}

fn dedupe_names(values: Vec<&'static str>) -> Vec<&'static str> {
    let mut seen = HashSet::new();

    values
        .into_iter()
        .filter(|value| seen.insert(*value))
        .collect()
}

fn resolve_named_icon(
    icon_name: &str,
    icon_cache: &mut HashMap<String, Option<PathBuf>>,
) -> Option<PathBuf> {
    if let Some(cached) = icon_cache.get(icon_name) {
        return cached.clone();
    }

    let resolved = find_icon_file(icon_name);
    icon_cache.insert(icon_name.to_string(), resolved.clone());
    resolved
}

fn find_icon_file(icon: &str) -> Option<PathBuf> {
    let icon_path = Path::new(icon);
    if icon_path.is_absolute() && icon_path.is_file() {
        return Some(icon_path.to_path_buf());
    }

    let candidates = icon_candidate_names(icon);

    for dir in pixmap_dirs() {
        for candidate in &candidates {
            let path = dir.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    for theme_dir in icon_theme_search_dirs() {
        if let Some(path) = find_icon_in_theme(theme_dir, &candidates) {
            return Some(path);
        }
    }

    None
}

fn icon_candidate_names(icon: &str) -> Vec<String> {
    let path = Path::new(icon);
    if path.extension().is_some() {
        return vec![icon.to_string()];
    }

    ["png", "svg"]
        .into_iter()
        .map(|ext| format!("{icon}.{ext}"))
        .collect()
}

fn find_icon_in_theme(theme_dir: &Path, candidates: &[String]) -> Option<PathBuf> {
    const CONTEXTS: &[&str] = &[
        "apps",
        "devices",
        "categories",
        "mimetypes",
        "places",
        "status",
        "actions",
        "panel",
        "emblems",
    ];
    const NUMERIC_SIZES: &[&str] = &[
        "512", "256", "128", "96", "64", "48", "36", "32", "24", "22", "16",
    ];
    const PRIMARY_SPECIAL_SIZES: &[&str] = &["scalable"];
    const FALLBACK_SPECIAL_SIZES: &[&str] = &["symbolic"];

    for context in CONTEXTS {
        for size in PRIMARY_SPECIAL_SIZES {
            for subdir in [format!("{context}/{size}"), format!("{size}/{context}")] {
                let dir = theme_dir.join(subdir);
                for candidate in candidates {
                    let path = dir.join(candidate);
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
        }

        for size in NUMERIC_SIZES {
            for subdir in [
                format!("{context}/{size}"),
                format!("{context}/{size}x{size}"),
                format!("{size}/{context}"),
                format!("{size}x{size}/{context}"),
            ] {
                let dir = theme_dir.join(subdir);
                for candidate in candidates {
                    let path = dir.join(candidate);
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
        }
    }

    for context in CONTEXTS {
        for size in FALLBACK_SPECIAL_SIZES {
            for subdir in [format!("{context}/{size}"), format!("{size}/{context}")] {
                let dir = theme_dir.join(subdir);
                for candidate in candidates {
                    let path = dir.join(candidate);
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

fn icon_theme_roots() -> &'static [PathBuf] {
    static ICON_THEME_ROOTS: OnceLock<Vec<PathBuf>> = OnceLock::new();

    ICON_THEME_ROOTS.get_or_init(|| {
        let mut roots: Vec<PathBuf> = xdg_data_dirs()
            .into_iter()
            .map(|dir| dir.join("icons"))
            .collect();

        if let Some(home) = home_dir() {
            roots.push(home.join(".icons"));
        }

        dedupe_paths(roots)
    })
}

fn pixmap_dirs() -> &'static [PathBuf] {
    static PIXMAP_DIRS: OnceLock<Vec<PathBuf>> = OnceLock::new();

    PIXMAP_DIRS.get_or_init(|| {
        let mut dirs: Vec<PathBuf> = xdg_data_dirs()
            .into_iter()
            .map(|dir| dir.join("pixmaps"))
            .collect();

        dirs.push(PathBuf::from("/usr/share/pixmaps"));
        dirs.push(PathBuf::from("/usr/local/share/pixmaps"));

        dedupe_paths(dirs)
    })
}

fn icon_theme_search_dirs() -> &'static [PathBuf] {
    static ICON_THEME_SEARCH_DIRS: OnceLock<Vec<PathBuf>> = OnceLock::new();

    ICON_THEME_SEARCH_DIRS.get_or_init(|| {
        let roots = icon_theme_roots();
        let mut search_dirs = Vec::new();

        for preferred_theme in PREFERRED_THEMES {
            for root in roots {
                search_dirs.push(root.join(preferred_theme));
            }
        }

        for root in roots {
            let Ok(entries) = fs::read_dir(root) else {
                continue;
            };

            let mut themes: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect();
            themes.sort();

            for theme_dir in themes {
                if let Some(name) = theme_dir.file_name().and_then(|name| name.to_str())
                    && PREFERRED_THEMES.contains(&name)
                {
                    continue;
                }

                search_dirs.push(theme_dir);
            }
        }

        dedupe_paths(search_dirs)
    })
}

fn xdg_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        dirs.push(data_home);
    } else if let Some(home) = home_dir() {
        dirs.push(home.join(".local/share"));
    }

    if let Some(data_dirs) = env::var_os("XDG_DATA_DIRS") {
        dirs.extend(env::split_paths(&data_dirs));
    } else {
        dirs.push(PathBuf::from("/usr/local/share"));
        dirs.push(PathBuf::from("/usr/share"));
    }

    dedupe_paths(dirs)
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();

    paths
        .into_iter()
        .filter(|path| seen.insert(path.clone()))
        .collect()
}

fn icon_lookup_cache() -> &'static Mutex<HashMap<String, Option<PathBuf>>> {
    static ICON_LOOKUP_CACHE: OnceLock<Mutex<HashMap<String, Option<PathBuf>>>> = OnceLock::new();

    ICON_LOOKUP_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
