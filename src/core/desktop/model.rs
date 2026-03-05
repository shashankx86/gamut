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
    search_blob: String,
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
        let search_blob = format!("{}\n{}", name.to_lowercase(), exec_line.to_lowercase());

        Self {
            name,
            exec_line,
            command,
            args,
            icon_name,
            icon_categories,
            icon_path,
            search_blob,
        }
    }

    pub fn matches_normalized_query(&self, normalized_query: &str) -> bool {
        normalized_query.is_empty() || self.search_blob.contains(normalized_query)
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

        assert!(app.matches_normalized_query(&fire));
        assert!(app.matches_normalized_query(&normalized));
        assert!(!app.matches_normalized_query(&spotify));
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
