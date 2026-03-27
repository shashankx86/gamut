pub(crate) fn group_i128(value: i128) -> String {
    group_signed_integer_str(&value.to_string())
}

pub(crate) fn group_signed_integer_str(value: &str) -> String {
    let (sign, digits) = if let Some(rest) = value.strip_prefix('-') {
        ("-", rest)
    } else {
        ("", value)
    };

    let mut grouped = String::new();
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }

    let grouped: String = grouped.chars().rev().collect();
    format!("{sign}{grouped}")
}

#[cfg(test)]
mod tests {
    use super::{group_i128, group_signed_integer_str};

    #[test]
    fn groups_i128_values() {
        assert_eq!(group_i128(1_234_567), "1,234,567");
        assert_eq!(group_i128(-1_234_567), "-1,234,567");
    }

    #[test]
    fn groups_signed_integer_strings() {
        assert_eq!(group_signed_integer_str("12345"), "12,345");
        assert_eq!(group_signed_integer_str("-12345"), "-12,345");
    }
}
