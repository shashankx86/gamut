use super::runtime::{SearchCommand, SearchIndex, SearchResponse, spawn_search_runtime};
use crate::core::desktop::DesktopApp;
use std::sync::mpsc::{Receiver, Sender};

const DEFAULT_RESULT_LIMIT: usize = 200;

#[derive(Debug, Clone, Default)]
struct ApplicationSearchIndex {
    entries: Vec<ApplicationSearchEntry>,
}

#[derive(Debug, Clone)]
struct ApplicationSearchEntry {
    app_index: usize,
    name: String,
    normalized_name: String,
    normalized_command: String,
    normalized_exec_line: String,
}

pub(crate) type ApplicationSearchResponse = SearchResponse<usize>;

#[derive(Debug, Clone)]
pub(crate) struct ApplicationSearchEngine {
    command_sender: Sender<SearchCommand<ApplicationSearchIndex>>,
}

impl ApplicationSearchEngine {
    pub(crate) fn spawn(limit: usize) -> (Self, Receiver<ApplicationSearchResponse>) {
        let runtime = spawn_search_runtime::<ApplicationSearchIndex>("gamut-app-search", limit);
        (
            Self {
                command_sender: runtime.command_sender,
            },
            runtime.result_receiver,
        )
    }

    pub(crate) fn replace_apps(&self, apps: &[DesktopApp]) -> bool {
        self.command_sender
            .send(SearchCommand::ReplaceIndex(
                ApplicationSearchIndex::from_apps(apps),
            ))
            .is_ok()
    }

    pub(crate) fn search(&self, generation: u64, normalized_query: String) -> bool {
        self.command_sender
            .send(SearchCommand::Search {
                generation,
                normalized_query,
            })
            .is_ok()
    }
}

pub(crate) fn rank_applications(apps: &[DesktopApp], normalized_query: &str) -> Vec<usize> {
    ApplicationSearchIndex::from_apps(apps).search(normalized_query, DEFAULT_RESULT_LIMIT)
}

impl ApplicationSearchIndex {
    fn from_apps(apps: &[DesktopApp]) -> Self {
        let entries = apps
            .iter()
            .enumerate()
            .map(|(app_index, app)| {
                let (normalized_name, normalized_command, normalized_exec_line) =
                    app.normalized_search_fields();

                ApplicationSearchEntry {
                    app_index,
                    name: app.name.clone(),
                    normalized_name: normalized_name.to_string(),
                    normalized_command: normalized_command.to_string(),
                    normalized_exec_line: normalized_exec_line.to_string(),
                }
            })
            .collect();

        Self { entries }
    }
}

impl SearchIndex for ApplicationSearchIndex {
    type Match = usize;

    fn search(&self, normalized_query: &str, limit: usize) -> Vec<Self::Match> {
        let mut ranked_matches: Vec<(usize, i32, &str)> = if normalized_query.is_empty() {
            self.entries
                .iter()
                .map(|entry| (entry.app_index, 0, entry.name.as_str()))
                .collect()
        } else {
            self.entries
                .iter()
                .filter_map(|entry| {
                    crate::core::desktop::search::query_match_score(
                        &entry.normalized_name,
                        &entry.normalized_command,
                        &entry.normalized_exec_line,
                        normalized_query,
                    )
                    .map(|score| (entry.app_index, score, entry.name.as_str()))
                })
                .collect()
        };

        if !normalized_query.is_empty() {
            ranked_matches.sort_by(
                |(left_index, left_score, left_name), (right_index, right_score, right_name)| {
                    right_score
                        .cmp(left_score)
                        .then_with(|| left_name.cmp(right_name))
                        .then_with(|| left_index.cmp(right_index))
                },
            );
        }

        ranked_matches
            .into_iter()
            .take(limit)
            .map(|(app_index, _, _)| app_index)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::rank_applications;
    use crate::core::desktop::{DesktopApp, normalize_query};

    fn app(name: &str, command: &str, exec_line: &str) -> DesktopApp {
        DesktopApp::new(
            name.to_string(),
            exec_line.to_string(),
            command.to_string(),
            Vec::new(),
            None,
            Vec::new(),
            None,
        )
    }

    #[test]
    fn empty_query_preserves_catalog_order() {
        let apps = vec![
            app("Alpha", "/usr/bin/alpha", "alpha"),
            app("Beta", "/usr/bin/beta", "beta"),
            app("Gamma", "/usr/bin/gamma", "gamma"),
        ];

        assert_eq!(rank_applications(&apps, ""), vec![0, 1, 2]);
    }

    #[test]
    fn ranked_matches_follow_existing_scoring_rules() {
        let apps = vec![
            app(
                "DaVinci Resolve",
                "/opt/resolve/bin/resolve",
                "/opt/resolve/bin/resolve %u",
            ),
            app(
                "Blackmagic RAW Player",
                "/opt/resolve/BlackmagicRAWPlayer/BlackmagicRAWPlayer",
                "/opt/resolve/BlackmagicRAWPlayer/BlackmagicRAWPlayer %f",
            ),
        ];

        assert_eq!(
            rank_applications(&apps, &normalize_query("resol")),
            vec![0, 1]
        );
    }
}
