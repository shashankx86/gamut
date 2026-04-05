pub(super) fn truncate_middle_with_ellipsis(value: &str, max_chars: usize) -> String {
    if max_chars <= 3 {
        return "...".to_string();
    }

    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }

    let visible = max_chars.saturating_sub(3);
    let left_count = visible / 2;
    let right_count = visible.saturating_sub(left_count);

    let left: String = value.chars().take(left_count).collect();
    let right: String = value
        .chars()
        .skip(char_count.saturating_sub(right_count))
        .collect();

    let mut output = left;
    output.push_str("...");
    output.push_str(&right);
    output
}

pub(super) fn normalize_result_display_value(value: &str) -> String {
    let cleaned: String = value.chars().filter(|ch| *ch != ',').collect();

    if let Ok(integer) = cleaned.parse::<i128>() {
        return crate::ui::format::group_i128(integer);
    }

    if let Some((integer, fraction)) = cleaned.split_once('.')
        && let Ok(integer) = integer.parse::<i128>()
    {
        return format!("{}.{}", crate::ui::format::group_i128(integer), fraction);
    }

    value.to_string()
}

pub(super) fn number_text_for_value(value: &str) -> String {
    let mut parts = Vec::new();

    for ch in value.chars() {
        let part = match ch {
            '0' => Some("Zero"),
            '1' => Some("One"),
            '2' => Some("Two"),
            '3' => Some("Three"),
            '4' => Some("Four"),
            '5' => Some("Five"),
            '6' => Some("Six"),
            '7' => Some("Seven"),
            '8' => Some("Eight"),
            '9' => Some("Nine"),
            '-' => Some("Negative"),
            '.' => Some("Point"),
            ',' => None,
            _ => None,
        };

        if let Some(part) = part {
            parts.push(part);
        }
    }

    if parts.is_empty() {
        "Result".to_string()
    } else {
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_uses_middle_three_dot_ellipsis() {
        let truncated = truncate_middle_with_ellipsis("abcdef", 5);

        assert_eq!(truncated.chars().count(), 5);
        assert!(truncated.starts_with('a'));
        assert!(truncated.ends_with('f'));
        assert!(truncated.contains("..."));
        assert_eq!(truncate_middle_with_ellipsis("abc", 5), "abc");
    }

    #[test]
    fn value_fallback_converts_digits_to_words() {
        let spoken = number_text_for_value("-10.5");

        assert!(spoken.starts_with("Negative"));
        assert!(spoken.contains("One"));
        assert!(spoken.contains("Zero"));
        assert!(spoken.contains("Point"));
        assert!(spoken.ends_with("Five"));
        assert_eq!(number_text_for_value("abc"), "Result");
    }

    #[test]
    fn result_display_is_grouped_with_commas() {
        assert_eq!(normalize_result_display_value("1234567"), "1,234,567");
        assert_eq!(normalize_result_display_value("1234567.89"), "1,234,567.89");
    }
}
