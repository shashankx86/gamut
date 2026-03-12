use std::sync::mpsc::{self, Receiver, Sender};

pub(crate) trait SearchIndex: Default + Send + 'static {
    type Match: Clone + Send + 'static;

    fn search(&self, normalized_query: &str, limit: usize) -> Vec<Self::Match>;
}

#[derive(Debug)]
pub(crate) enum SearchCommand<I> {
    ReplaceIndex(I),
    Search {
        generation: u64,
        normalized_query: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchResponse<M> {
    pub(crate) generation: u64,
    pub(crate) matches: Vec<M>,
}

pub(crate) struct SearchRuntime<I: SearchIndex> {
    pub(crate) command_sender: Sender<SearchCommand<I>>,
    pub(crate) result_receiver: Receiver<SearchResponse<I::Match>>,
}

pub(crate) fn spawn_search_runtime<I>(thread_name: &'static str, limit: usize) -> SearchRuntime<I>
where
    I: SearchIndex,
{
    let (command_sender, command_receiver) = mpsc::channel();
    let (result_sender, result_receiver) = mpsc::channel();

    std::thread::Builder::new()
        .name(thread_name.to_string())
        .spawn(move || run_search_runtime::<I>(command_receiver, result_sender, limit))
        .expect("failed to spawn search runtime thread");

    SearchRuntime {
        command_sender,
        result_receiver,
    }
}

fn run_search_runtime<I>(
    command_receiver: Receiver<SearchCommand<I>>,
    result_sender: Sender<SearchResponse<I::Match>>,
    limit: usize,
) where
    I: SearchIndex,
{
    let mut index = I::default();
    let mut generation = 0;
    let mut normalized_query = String::new();

    while let Ok(command) = command_receiver.recv() {
        apply_command(command, &mut index, &mut generation, &mut normalized_query);

        while let Ok(command) = command_receiver.try_recv() {
            apply_command(command, &mut index, &mut generation, &mut normalized_query);
        }

        let response = SearchResponse {
            generation,
            matches: index.search(&normalized_query, limit),
        };

        if result_sender.send(response).is_err() {
            break;
        }
    }
}

fn apply_command<I>(
    command: SearchCommand<I>,
    index: &mut I,
    generation: &mut u64,
    normalized_query: &mut String,
) where
    I: SearchIndex,
{
    match command {
        SearchCommand::ReplaceIndex(next_index) => *index = next_index,
        SearchCommand::Search {
            generation: next_generation,
            normalized_query: next_query,
        } => {
            *generation = next_generation;
            *normalized_query = next_query;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SearchCommand, SearchIndex, spawn_search_runtime};
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;

    #[derive(Debug, Clone, Default)]
    struct FakeIndex(Vec<&'static str>);

    impl SearchIndex for FakeIndex {
        type Match = &'static str;

        fn search(&self, normalized_query: &str, limit: usize) -> Vec<Self::Match> {
            self.0
                .iter()
                .copied()
                .filter(|value| value.contains(normalized_query))
                .take(limit)
                .collect()
        }
    }

    fn recv_generation(
        receiver: &std::sync::mpsc::Receiver<super::SearchResponse<&'static str>>,
        generation: u64,
    ) -> Vec<&'static str> {
        loop {
            match receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(response) if response.generation == generation => return response.matches,
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => {
                    panic!("timed out waiting for search generation {generation}")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("search runtime disconnected before returning generation {generation}")
                }
            }
        }
    }

    #[test]
    fn latest_query_generation_wins() {
        let runtime = spawn_search_runtime::<FakeIndex>("test-search-runtime", 10);

        runtime
            .command_sender
            .send(SearchCommand::ReplaceIndex(FakeIndex(vec![
                "alpha", "alpine", "beta",
            ])))
            .unwrap();
        runtime
            .command_sender
            .send(SearchCommand::Search {
                generation: 1,
                normalized_query: "a".to_string(),
            })
            .unwrap();
        runtime
            .command_sender
            .send(SearchCommand::Search {
                generation: 2,
                normalized_query: "alp".to_string(),
            })
            .unwrap();

        assert_eq!(
            recv_generation(&runtime.result_receiver, 2),
            vec!["alpha", "alpine"]
        );
    }

    #[test]
    fn replacing_index_reuses_current_query() {
        let runtime = spawn_search_runtime::<FakeIndex>("test-search-runtime-update", 10);

        runtime
            .command_sender
            .send(SearchCommand::ReplaceIndex(FakeIndex(vec![
                "alpha", "beta",
            ])))
            .unwrap();
        runtime
            .command_sender
            .send(SearchCommand::Search {
                generation: 1,
                normalized_query: "ga".to_string(),
            })
            .unwrap();
        runtime
            .command_sender
            .send(SearchCommand::ReplaceIndex(FakeIndex(vec![
                "gamma", "omega",
            ])))
            .unwrap();

        assert_eq!(
            recv_generation(&runtime.result_receiver, 1),
            vec!["gamma", "omega"]
        );
    }
}
