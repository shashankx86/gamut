use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DesktopApp {
    pub name: String,
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

    pub fn query_match_score(&self, normalized_query: &str) -> Option<i32> {
        if normalized_query.is_empty() {
            return Some(0);
        }

        let mut best_score: Option<i32> = None;

        let mut update_best = |candidate: i32| {
            best_score = Some(best_score.map_or(candidate, |current| current.max(candidate)));
        };

        if self.normalized_name == normalized_query {
            update_best(12_000);
        }

        if self.normalized_name.starts_with(normalized_query) {
            update_best(10_000 - self.normalized_name.len() as i32);
        }

        for (index, _) in self.normalized_name.match_indices(normalized_query) {
            if is_word_boundary(&self.normalized_name, index) {
                update_best(9_000 - index as i32);
            } else {
                update_best(8_000 - index as i32);
            }
        }

        if self.normalized_command.starts_with(normalized_query) {
            update_best(6_000 - self.normalized_command.len() as i32);
        }

        if let Some(index) = self.normalized_command.find(normalized_query) {
            update_best(5_000 - index as i32);
        }

        if let Some(index) = self.normalized_exec_line.find(normalized_query) {
            update_best(1_000 - index as i32);
        }

        best_score
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
}

pub fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

fn is_word_boundary(text: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }

    text[..index]
        .chars()
        .next_back()
        .is_none_or(|ch| !ch.is_alphanumeric())
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
    use super::{DesktopApp, normalize_query, trim_label};

    #[test]
    fn query_matching_is_case_insensitive() {
        let app = DesktopApp::new(
            "Firefox".to_string(),
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
            "/opt/resolve/bin/resolve %u".to_string(),
            "/opt/resolve/bin/resolve".to_string(),
            vec!["%u".to_string()],
            None,
            Vec::new(),
            None,
        );
        let raw_player = DesktopApp::new(
            "Blackmagic RAW Player".to_string(),
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
            "/opt/resolve/bin/resolve %u".to_string(),
            "/opt/resolve/bin/resolve".to_string(),
            vec!["%u".to_string()],
            None,
            Vec::new(),
            None,
        );
        let control_panels = DesktopApp::new(
            "DaVinci Control Panels Setup".to_string(),
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
