use super::icons::{resolve_context_icon_path, resolve_generic_icon_path, resolve_icon_path};
use super::model::DesktopApp;
use freedesktop_desktop_entry::{DesktopEntry, Iter, default_paths, get_languages_from_env};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn load_apps() -> Vec<DesktopApp> {
    let locales = get_languages_from_env();
    let mut seen = HashSet::new();
    let mut icon_cache: HashMap<String, Option<PathBuf>> = HashMap::new();
    let mut apps = Vec::new();

    for entry in Iter::new(default_paths()).entries(Some(&locales)) {
        if !is_launchable_entry(&entry) {
            continue;
        }

        let Some(app) = parse_desktop_app(&entry, &locales, &mut icon_cache) else {
            continue;
        };

        if !seen.insert(dedupe_key(&app)) {
            continue;
        }

        apps.push(app);
    }

    apps.sort_by_cached_key(|app| app.name.to_lowercase());
    apps
}

fn parse_exec_command(entry: &DesktopEntry) -> Option<(String, Vec<String>)> {
    let parsed = entry.parse_exec().ok()?;
    let (command, args) = parsed.split_first()?;
    Some((command.to_string(), args.to_vec()))
}

fn is_launchable_entry(entry: &DesktopEntry) -> bool {
    if entry.hidden() || entry.no_display() {
        return false;
    }

    !matches!(entry.type_(), Some(kind) if kind != "Application")
}

fn parse_desktop_app(
    entry: &DesktopEntry,
    locales: &[String],
    icon_cache: &mut HashMap<String, Option<PathBuf>>,
) -> Option<DesktopApp> {
    let name = entry.name(locales).map(|value| value.to_string())?;
    let exec_line = entry.exec().map(|value| value.to_string())?;
    let (command, args) = parse_exec_command(entry)?;

    let icon_path = resolve_icon_path(entry.icon(), icon_cache)
        .or_else(|| resolve_context_icon_path(entry, &name, &exec_line, icon_cache))
        .or_else(|| resolve_generic_icon_path(icon_cache));

    Some(DesktopApp::new(name, exec_line, command, args, icon_path))
}

fn dedupe_key(app: &DesktopApp) -> String {
    format!("{}\0{}\0{}", app.name, app.command, app.args.join("\0"))
}
