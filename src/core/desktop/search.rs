pub(crate) fn query_match_score(
    normalized_name: &str,
    normalized_command: &str,
    normalized_exec_line: &str,
    normalized_query: &str,
) -> Option<i32> {
    if normalized_query.is_empty() {
        return Some(0);
    }

    let mut best_score: Option<i32> = None;

    let mut update_best = |candidate: i32| {
        best_score = Some(best_score.map_or(candidate, |current| current.max(candidate)));
    };

    if normalized_name == normalized_query {
        update_best(12_000);
    }

    if normalized_name.starts_with(normalized_query) {
        update_best(10_000 - normalized_name.len() as i32);
    }

    for (index, _) in normalized_name.match_indices(normalized_query) {
        if is_word_boundary(normalized_name, index) {
            update_best(9_000 - index as i32);
        } else {
            update_best(8_000 - index as i32);
        }
    }

    if normalized_command.starts_with(normalized_query) {
        update_best(6_000 - normalized_command.len() as i32);
    }

    if let Some(index) = normalized_command.find(normalized_query) {
        update_best(5_000 - index as i32);
    }

    if let Some(index) = normalized_exec_line.find(normalized_query) {
        update_best(1_000 - index as i32);
    }

    best_score
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

#[cfg(test)]
mod tests {
    use super::query_match_score;

    #[test]
    fn exact_match_beats_prefix_and_substring_matches() {
        let exact =
            query_match_score("firefox", "firefox", "firefox %u", "firefox").unwrap_or_default();
        let prefix =
            query_match_score("firefox developer edition", "firefox", "firefox %u", "fire")
                .unwrap_or_default();
        let substring = query_match_score("mozilla firefox", "firefox", "firefox %u", "fox")
            .unwrap_or_default();

        assert!(exact > prefix);
        assert!(prefix > substring);
    }

    #[test]
    fn missing_query_returns_none() {
        assert_eq!(
            query_match_score("firefox", "firefox", "firefox %u", "spotify"),
            None
        );
    }
}
