use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

const PREFERRED_THEMES: &[&str] = &["breeze-dark", "breeze", "Adwaita", "hicolor"];

pub(super) fn icon_theme_search_dirs() -> &'static [PathBuf] {
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

pub(super) fn pixmap_dirs() -> &'static [PathBuf] {
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
