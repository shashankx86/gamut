use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopApp {
    pub name: String,
    pub entry_type: String,
    pub exec_line: String,
    pub command: String,
    pub args: Vec<String>,
    pub icon_name: Option<String>,
    pub icon_categories: Vec<String>,
    pub icon_path: Option<PathBuf>,
    normalized_name: String,
    normalized_command: String,
    normalized_exec_line: String,
}

#[derive(Debug, Clone)]
pub struct IconResolveRequest {
    pub index: usize,
    pub icon_name: Option<String>,
    pub icon_categories: Vec<String>,
    pub name: String,
    pub exec_line: String,
}

impl DesktopApp {
    pub fn new(
        name: String,
        entry_type: String,
        exec_line: String,
        command: String,
        args: Vec<String>,
        icon_name: Option<String>,
        icon_categories: Vec<String>,
        icon_path: Option<PathBuf>,
    ) -> Self {
        let normalized_name = name.to_lowercase();
        let normalized_exec_line = exec_line.to_lowercase();
        let normalized_command = command
            .rsplit('/')
            .next()
            .unwrap_or(command.as_str())
            .to_lowercase();

        Self {
            name,
            entry_type,
            exec_line,
            command,
            args,
            icon_name,
            icon_categories,
            icon_path,
            normalized_name,
            normalized_command,
            normalized_exec_line,
        }
    }

    #[cfg(test)]
    pub fn query_match_score(&self, normalized_query: &str) -> Option<i32> {
        super::search::query_match_score(
            &self.normalized_name,
            &self.normalized_command,
            &self.normalized_exec_line,
            normalized_query,
        )
    }

    pub fn icon_request(&self, index: usize) -> IconResolveRequest {
        IconResolveRequest {
            index,
            icon_name: self.icon_name.clone(),
            icon_categories: self.icon_categories.clone(),
            name: self.name.clone(),
            exec_line: self.exec_line.clone(),
        }
    }

    pub(crate) fn normalized_search_fields(&self) -> (&str, &str, &str) {
        (
            &self.normalized_name,
            &self.normalized_command,
            &self.normalized_exec_line,
        )
    }
}

pub fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

pub fn trim_label(value: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }

    if value.chars().count() <= max_len {
        return value.to_string();
    }

    let mut output: String = value.chars().take(max_len.saturating_sub(1)).collect();
    output.push('~');
    output
}

#[cfg(test)]
mod tests {
    use super::{normalize_query, trim_label, DesktopApp};

    #[test]
    fn query_matching_is_case_insensitive() {
        let app = DesktopApp::new(
            "Firefox".to_string(),
            "Application".to_string(),
            "/usr/bin/firefox --new-window".to_string(),
            "/usr/bin/firefox".to_string(),
            vec!["--new-window".to_string()],
            None,
            Vec::new(),
            None,
        );

        let normalized = normalize_query("NEW-WINDOW");
        let fire = normalize_query("fire");
        let spotify = normalize_query("spotify");

        assert!(app.query_match_score(&fire).is_some());
        assert!(app.query_match_score(&normalized).is_some());
        assert!(app.query_match_score(&spotify).is_none());
    }

    #[test]
    fn name_prefix_beats_exec_path_match() {
        let resolve = DesktopApp::new(
            "DaVinci Resolve".to_string(),
            "Application".to_string(),
            "/opt/resolve/bin/resolve %u".to_string(),
            "/opt/resolve/bin/resolve".to_string(),
            vec!["%u".to_string()],
            None,
            Vec::new(),
            None,
        );
        let raw_player = DesktopApp::new(
            "Blackmagic RAW Player".to_string(),
            "Application".to_string(),
            "/opt/resolve/BlackmagicRAWPlayer/BlackmagicRAWPlayer %f".to_string(),
            "/opt/resolve/BlackmagicRAWPlayer/BlackmagicRAWPlayer".to_string(),
            vec!["%f".to_string()],
            None,
            Vec::new(),
            None,
        );

        let query = normalize_query("resol");
        assert!(
            resolve.query_match_score(&query).unwrap_or_default()
                > raw_player.query_match_score(&query).unwrap_or_default()
        );
    }

    #[test]
    fn shorter_prefix_match_scores_higher() {
        let resolve = DesktopApp::new(
            "DaVinci Resolve".to_string(),
            "Application".to_string(),
            "/opt/resolve/bin/resolve %u".to_string(),
            "/opt/resolve/bin/resolve".to_string(),
            vec!["%u".to_string()],
            None,
            Vec::new(),
            None,
        );
        let control_panels = DesktopApp::new(
            "DaVinci Control Panels Setup".to_string(),
            "Application".to_string(),
            "/opt/resolve/DaVinci Control Panels Setup/DaVinci Control".to_string(),
            "/opt/resolve/DaVinci Control Panels Setup/DaVinci Control".to_string(),
            Vec::new(),
            None,
            Vec::new(),
            None,
        );

        let query = normalize_query("dav");
        assert!(
            resolve.query_match_score(&query).unwrap_or_default()
                > control_panels.query_match_score(&query).unwrap_or_default()
        );
    }

    #[test]
    fn trim_label_is_noop_for_short_text() {
        assert_eq!(trim_label("abc", 10), "abc");
    }

    #[test]
    fn trim_label_truncates_and_appends_tilde() {
        assert_eq!(trim_label("abcdef", 4), "abc~");
    }

    #[test]
    fn trim_label_zero_limit_returns_empty() {
        assert_eq!(trim_label("abcdef", 0), "");
    }
}
