use super::paths::{icon_theme_search_dirs, pixmap_dirs};
use std::path::{Path, PathBuf};

pub(super) fn find_icon_file(icon: &str) -> Option<PathBuf> {
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
