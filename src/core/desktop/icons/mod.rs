mod context;
mod paths;
mod search;

use super::model::IconResolveRequest;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const GENERIC_ICON_CACHE_KEY: &str = "__generic_app_icon__";

type IconLookupCache = HashMap<String, Option<PathBuf>>;

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

fn resolve_icon_path(icon: Option<&str>, icon_cache: &mut IconLookupCache) -> Option<PathBuf> {
    let icon = icon?.trim();
    if icon.is_empty() {
        return None;
    }

    if let Some(cached) = icon_cache.get(icon) {
        return cached.clone();
    }

    let resolved = search::find_icon_file(icon);
    icon_cache.insert(icon.to_string(), resolved.clone());
    resolved
}

fn resolve_generic_icon_path(icon_cache: &mut IconLookupCache) -> Option<PathBuf> {
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
    .find_map(search::find_icon_file);

    icon_cache.insert(GENERIC_ICON_CACHE_KEY.to_string(), resolved.clone());
    resolved
}

fn resolve_context_icon_path(
    categories: &[String],
    name: &str,
    exec_line: &str,
    icon_cache: &mut IconLookupCache,
) -> Option<PathBuf> {
    context::context_icon_candidates(categories, name, exec_line)
        .into_iter()
        .find_map(|icon_name| resolve_named_icon(icon_name, icon_cache))
}

fn resolve_named_icon(icon_name: &str, icon_cache: &mut IconLookupCache) -> Option<PathBuf> {
    if let Some(cached) = icon_cache.get(icon_name) {
        return cached.clone();
    }

    let resolved = search::find_icon_file(icon_name);
    icon_cache.insert(icon_name.to_string(), resolved.clone());
    resolved
}

fn icon_lookup_cache() -> &'static Mutex<IconLookupCache> {
    static ICON_LOOKUP_CACHE: OnceLock<Mutex<IconLookupCache>> = OnceLock::new();

    ICON_LOOKUP_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
