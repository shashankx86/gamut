use super::icons::resolve_app_icon;
use super::model::{DesktopApp, IconResolveRequest};
use freedesktop_desktop_entry::{
    DesktopEntry, Iter, PathSource, default_paths, get_languages_from_env,
};
use std::collections::{HashMap, hash_map::Entry};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ScoredApp {
    app: DesktopApp,
    source_priority: usize,
}

pub fn load_apps() -> Vec<DesktopApp> {
    let locales = get_languages_from_env();
    let mut deduped: HashMap<String, ScoredApp> = HashMap::new();

    for entry in Iter::new(default_paths()).entries(Some(&locales)) {
        if !is_launchable_entry(&entry) {
            continue;
        }

        let Some(app) = parse_desktop_app(&entry, &locales) else {
            continue;
        };
        let scored = ScoredApp {
            app,
            source_priority: source_priority_for_path(&entry.path),
        };

        let key = dedupe_key(&scored.app);
        match deduped.entry(key) {
            Entry::Vacant(slot) => {
                slot.insert(scored);
            }
            Entry::Occupied(mut slot) => {
                if should_replace_existing(slot.get(), &scored) {
                    slot.insert(scored);
                }
            }
        }
    }

    let mut apps: Vec<DesktopApp> = deduped.into_values().map(|entry| entry.app).collect();
    apps.sort_by_cached_key(|app| app.name.to_lowercase());
    apps
}

pub fn resolve_icon_requests(requests: Vec<IconResolveRequest>) -> Vec<(usize, Option<PathBuf>)> {
    requests
        .into_iter()
        .map(|request| {
            let icon_path = resolve_app_icon(&request);
            (request.index, icon_path)
        })
        .collect()
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

fn parse_desktop_app(entry: &DesktopEntry, locales: &[String]) -> Option<DesktopApp> {
    let name = entry.name(locales).map(|value| value.to_string())?;
    let exec_line = entry.exec().map(|value| value.to_string())?;
    let (command, args) = parse_exec_command(entry)?;

    let icon_name = entry.icon().map(str::trim).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    });

    let icon_categories = entry
        .categories()
        .unwrap_or_default()
        .into_iter()
        .map(|category| category.to_string())
        .collect();

    Some(DesktopApp::new(
        name,
        exec_line,
        command,
        args,
        icon_name,
        icon_categories,
        None,
    ))
}

fn dedupe_key(app: &DesktopApp) -> String {
    let normalized_name = canonical_name_for_dedupe(&app.name);
    let command_basename = app
        .command
        .rsplit('/')
        .next()
        .unwrap_or(app.command.as_str())
        .to_lowercase();

    format!("{normalized_name}\0{command_basename}")
}

fn should_replace_existing(existing: &ScoredApp, candidate: &ScoredApp) -> bool {
    (candidate.source_priority, app_preference(&candidate.app))
        < (existing.source_priority, app_preference(&existing.app))
}

fn app_preference(app: &DesktopApp) -> (usize, usize, usize) {
    let meaningful_args = app
        .args
        .iter()
        .filter(|arg| !is_exec_placeholder(arg))
        .count();

    (
        meaningful_args,
        app.name.chars().count(),
        app.exec_line.len(),
    )
}

fn is_exec_placeholder(arg: &str) -> bool {
    matches!(
        arg,
        "%f" | "%F" | "%u" | "%U" | "%i" | "%c" | "%k" | "%d" | "%D" | "%n" | "%N" | "%v" | "%m"
    )
}

fn source_priority_for_path(path: &Path) -> usize {
    match PathSource::guess_from(path) {
        PathSource::Local | PathSource::LocalDesktop => 0,
        PathSource::LocalFlatpak => 1,
        PathSource::LocalNix => 2,
        PathSource::Nix => 3,
        PathSource::System
        | PathSource::SystemLocal
        | PathSource::SystemFlatpak
        | PathSource::SystemSnap => 4,
        PathSource::Other(_) => 5,
    }
}

fn canonical_name_for_dedupe(name: &str) -> String {
    const PREFIXES: [&str; 6] = ["kde ", "gnome ", "xfce ", "lxqt ", "mate ", "plasma "];

    let normalized = name.trim().to_lowercase();
    PREFIXES
        .iter()
        .find_map(|prefix| normalized.strip_prefix(prefix).map(str::trim))
        .unwrap_or(normalized.as_str())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{DesktopApp, ScoredApp, dedupe_key, should_replace_existing};

    fn app(name: &str, command: &str, exec_line: &str, args: Vec<&str>) -> DesktopApp {
        DesktopApp::new(
            name.to_string(),
            exec_line.to_string(),
            command.to_string(),
            args.into_iter().map(str::to_string).collect(),
            None,
            Vec::new(),
            None,
        )
    }

    fn scored_app(
        name: &str,
        command: &str,
        exec_line: &str,
        args: Vec<&str>,
        source_priority: usize,
    ) -> ScoredApp {
        ScoredApp {
            app: app(name, command, exec_line, args),
            source_priority,
        }
    }

    #[test]
    fn dedupe_key_ignores_argument_variants() {
        let with_flag = app(
            "Spotify (Launcher)",
            "/usr/bin/spotify-launcher",
            "spotify-launcher --skip-update %U",
            vec!["--skip-update", "%U"],
        );
        let plain = app(
            "Spotify (Launcher)",
            "/usr/bin/spotify-launcher",
            "spotify-launcher %U",
            vec!["%U"],
        );

        assert_eq!(dedupe_key(&with_flag), dedupe_key(&plain));
    }

    #[test]
    fn prefers_entry_with_fewer_non_placeholder_flags() {
        let with_flag = scored_app(
            "Spotify (Launcher)",
            "/usr/bin/spotify-launcher",
            "spotify-launcher --skip-update %U",
            vec!["--skip-update", "%U"],
            4,
        );
        let plain = scored_app(
            "Spotify (Launcher)",
            "/usr/bin/spotify-launcher",
            "spotify-launcher %U",
            vec!["%U"],
            4,
        );

        assert!(should_replace_existing(&with_flag, &plain));
        assert!(!should_replace_existing(&plain, &with_flag));
    }

    #[test]
    fn dedupe_key_ignores_known_desktop_prefixes() {
        let generic = app(
            "System Settings",
            "/usr/bin/systemsettings",
            "systemsettings",
            Vec::new(),
        );
        let kde = app(
            "KDE System Settings",
            "/usr/bin/systemsettings",
            "systemsettings",
            Vec::new(),
        );

        assert_eq!(dedupe_key(&generic), dedupe_key(&kde));
        let generic_scored = ScoredApp {
            app: generic,
            source_priority: 4,
        };
        let kde_scored = ScoredApp {
            app: kde,
            source_priority: 4,
        };
        assert!(should_replace_existing(&kde_scored, &generic_scored));
    }

    #[test]
    fn higher_priority_source_wins_even_with_extra_args() {
        let local_custom = scored_app(
            "Spotify (Launcher)",
            "/usr/bin/spotify-launcher",
            "spotify-launcher --skip-update %U",
            vec!["--skip-update", "%U"],
            0,
        );
        let system_plain = scored_app(
            "Spotify (Launcher)",
            "/usr/bin/spotify-launcher",
            "spotify-launcher %U",
            vec!["%U"],
            4,
        );

        assert!(!should_replace_existing(&local_custom, &system_plain));
        assert!(should_replace_existing(&system_plain, &local_custom));
    }
}
