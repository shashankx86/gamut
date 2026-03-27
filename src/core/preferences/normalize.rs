pub(crate) fn normalize_identifier(value: &str) -> String {
    normalize_with(value, |ch| matches!(ch, ' ' | '_' | '-' | '.'))
}

pub(crate) fn normalize_key_token(value: &str) -> String {
    normalize_with(value, |ch| matches!(ch, ' ' | '_' | '-'))
}

fn normalize_with(value: &str, should_strip: impl Fn(char) -> bool) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !should_strip(*ch))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{normalize_identifier, normalize_key_token};

    #[test]
    fn identifier_normalization_strips_dots_and_separators() {
        assert_eq!(
            normalize_identifier(" shortcuts.move_up "),
            "shortcutsmoveup"
        );
    }

    #[test]
    fn key_token_normalization_keeps_dots() {
        assert_eq!(normalize_key_token(" Arrow-Down "), "arrowdown");
    }
}
