pub(super) fn parse_bool(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        _ => Err("expected a boolean: true/false, yes/no, on/off, 1/0".to_string()),
    }
}

pub(super) fn parse_non_negative_f32(value: &str, label: &str) -> Result<f32, String> {
    let parsed = value
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("{label} must be a number"))?;

    if !parsed.is_finite() {
        return Err(format!("{label} must be finite"));
    }

    if parsed < 0.0 {
        return Err(format!("{label} must be non-negative"));
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::{parse_bool, parse_non_negative_f32};

    #[test]
    fn bool_parser_accepts_common_values() {
        assert!(parse_bool("yes").expect("yes should parse"));
        assert!(!parse_bool("off").expect("off should parse"));
    }

    #[test]
    fn numeric_parser_rejects_negative_values() {
        assert!(parse_non_negative_f32("-1", "test").is_err());
    }
}
