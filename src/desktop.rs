use freedesktop_desktop_entry::{
    DesktopEntry, Iter, default_paths, get_languages_from_env,
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct DesktopApp {
    pub name: String,
    pub exec_line: String,
    pub command: String,
    pub args: Vec<String>,
    search_blob: String,
}

impl DesktopApp {
    pub fn new(name: String, exec_line: String, command: String, args: Vec<String>) -> Self {
        let search_blob = format!(
            "{}\n{}",
            name.to_lowercase(),
            exec_line.to_lowercase()
        );

        Self {
            name,
            exec_line,
            command,
            args,
            search_blob,
        }
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let normalized = query.trim().to_lowercase();
        normalized.is_empty() || self.search_blob.contains(&normalized)
    }
}

pub fn load_apps() -> Vec<DesktopApp> {
    let locales = get_languages_from_env();
    let mut dedupe = HashSet::new();
    let mut apps = Vec::new();

    for entry in Iter::new(default_paths()).entries(Some(&locales)) {
        if entry.hidden() || entry.no_display() {
            continue;
        }

        if let Some(kind) = entry.type_()
            && kind != "Application"
        {
            continue;
        }

        let Some(name) = entry.name(&locales).map(|value| value.to_string()) else {
            continue;
        };

        let Some(exec_line) = entry.exec().map(|value| value.to_string()) else {
            continue;
        };

        let Some((command, args)) = parse_exec_command(&entry) else {
            continue;
        };

        let dedupe_key = format!("{name}\0{command}\0{}", args.join("\0"));
        if !dedupe.insert(dedupe_key) {
            continue;
        }

        apps.push(DesktopApp::new(name, exec_line, command, args));
    }

    apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    apps
}

pub fn trim_label(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }

    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if index >= max_len.saturating_sub(1) {
            break;
        }
        output.push(character);
    }
    output.push('~');
    output
}

fn parse_exec_command(entry: &DesktopEntry) -> Option<(String, Vec<String>)> {
    let parsed = entry.parse_exec().ok()?;
    let (command, args) = parsed.split_first()?;
    Some((command.to_string(), args.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::{DesktopApp, trim_label};

    #[test]
    fn query_matching_is_case_insensitive() {
        let app = DesktopApp::new(
            "Firefox".to_string(),
            "/usr/bin/firefox --new-window".to_string(),
            "/usr/bin/firefox".to_string(),
            vec!["--new-window".to_string()],
        );

        assert!(app.matches_query("fire"));
        assert!(app.matches_query("NEW-WINDOW"));
        assert!(!app.matches_query("spotify"));
    }

    #[test]
    fn trim_label_is_noop_for_short_text() {
        assert_eq!(trim_label("abc", 10), "abc");
    }

    #[test]
    fn trim_label_truncates_and_appends_tilde() {
        assert_eq!(trim_label("abcdef", 4), "abc~");
    }
}
