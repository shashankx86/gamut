#[derive(Debug, Clone, PartialEq)]
pub(in crate::ui) struct CalculationPreview {
    pub(in crate::ui) expression: String,
    pub(in crate::ui) formatted_value: String,
    pub(in crate::ui) words: Option<String>,
}

const FRACTION_TOLERANCE: f64 = 1e-9;
const MAX_WORDS_INTEGER: i64 = 999_999_999_999;

pub(in crate::ui::launcher) fn calculation_preview(query: &str) -> Option<CalculationPreview> {
    let expression = query.trim();
    if expression.is_empty() {
        return None;
    }

    let evaluated = ExpressionParser::new(expression).parse()?;
    if evaluated.binary_operator_count == 0 {
        return None;
    }

    let formatted_value = format_number(evaluated.value);
    let words = integer_words(evaluated.value);

    Some(CalculationPreview {
        expression: expression.to_string(),
        formatted_value,
        words,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct EvaluatedExpression {
    value: f64,
    binary_operator_count: usize,
}

struct ExpressionParser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    cursor: usize,
    binary_operator_count: usize,
}

impl<'a> ExpressionParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            cursor: 0,
            binary_operator_count: 0,
        }
    }

    fn parse(mut self) -> Option<EvaluatedExpression> {
        let value = self.parse_expression()?;
        self.skip_whitespace();

        if self.cursor != self.bytes.len() || !value.is_finite() {
            return None;
        }

        Some(EvaluatedExpression {
            value,
            binary_operator_count: self.binary_operator_count,
        })
    }

    fn parse_expression(&mut self) -> Option<f64> {
        let mut value = self.parse_term()?;

        loop {
            self.skip_whitespace();
            let operator = match self.peek_byte() {
                Some(b'+') | Some(b'-') => self.consume_byte()?,
                _ => break,
            };

            self.binary_operator_count += 1;
            let rhs = self.parse_term()?;
            value = if operator == b'+' {
                value + rhs
            } else {
                value - rhs
            };
        }

        Some(value)
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut value = self.parse_unary()?;

        loop {
            self.skip_whitespace();
            let operator = match self.peek_byte() {
                Some(b'*') | Some(b'/') | Some(b'%') => self.consume_byte()?,
                _ => break,
            };

            self.binary_operator_count += 1;
            let rhs = self.parse_unary()?;

            value = match operator {
                b'*' => value * rhs,
                b'/' => {
                    if rhs.abs() <= FRACTION_TOLERANCE {
                        return None;
                    }
                    value / rhs
                }
                b'%' => {
                    if rhs.abs() <= FRACTION_TOLERANCE {
                        return None;
                    }
                    value % rhs
                }
                _ => return None,
            };
        }

        Some(value)
    }

    fn parse_unary(&mut self) -> Option<f64> {
        self.skip_whitespace();
        let mut sign = 1.0;

        loop {
            match self.peek_byte() {
                Some(b'+') => {
                    self.consume_byte();
                    self.skip_whitespace();
                }
                Some(b'-') => {
                    self.consume_byte();
                    sign = -sign;
                    self.skip_whitespace();
                }
                _ => break,
            }
        }

        let value = self.parse_primary()?;
        Some(sign * value)
    }

    fn parse_primary(&mut self) -> Option<f64> {
        self.skip_whitespace();

        match self.peek_byte()? {
            b'(' => {
                self.consume_byte();
                let value = self.parse_expression()?;
                self.skip_whitespace();
                if self.consume_byte()? != b')' {
                    return None;
                }
                Some(value)
            }
            b'0'..=b'9' | b'.' => self.parse_number(),
            _ => None,
        }
    }

    fn parse_number(&mut self) -> Option<f64> {
        let start = self.cursor;
        let mut has_digit = false;
        let mut has_decimal = false;

        while let Some(next) = self.peek_byte() {
            match next {
                b'0'..=b'9' => {
                    has_digit = true;
                    self.cursor += 1;
                }
                b'.' if !has_decimal => {
                    has_decimal = true;
                    self.cursor += 1;
                }
                b',' | b'_' => {
                    self.cursor += 1;
                }
                _ => break,
            }
        }

        if !has_digit {
            return None;
        }

        let token = &self.input[start..self.cursor];
        let sanitized: String = token
            .chars()
            .filter(|ch| *ch != ',' && *ch != '_')
            .collect();

        sanitized.parse::<f64>().ok()
    }

    fn skip_whitespace(&mut self) {
        while let Some(next) = self.peek_byte() {
            if next.is_ascii_whitespace() {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.cursor).copied()
    }

    fn consume_byte(&mut self) -> Option<u8> {
        let value = self.peek_byte()?;
        self.cursor += 1;
        Some(value)
    }
}

fn format_number(value: f64) -> String {
    if let Some(integer) = rounded_integer(value) {
        return format_integer(integer);
    }

    let mut decimal = format!("{value:.8}");
    while decimal.contains('.') && decimal.ends_with('0') {
        decimal.pop();
    }
    if decimal.ends_with('.') {
        decimal.pop();
    }

    if let Some((integer_part, fractional_part)) = decimal.split_once('.') {
        format!(
            "{}.{}",
            format_integer_from_str(integer_part),
            fractional_part
        )
    } else {
        format_integer_from_str(&decimal)
    }
}

fn rounded_integer(value: f64) -> Option<i64> {
    let rounded = value.round();
    if (value - rounded).abs() <= FRACTION_TOLERANCE
        && rounded.is_finite()
        && rounded >= i64::MIN as f64
        && rounded <= i64::MAX as f64
    {
        Some(rounded as i64)
    } else {
        None
    }
}

fn format_integer(value: i64) -> String {
    crate::ui::format::group_i128(i128::from(value))
}

fn format_integer_from_str(value: &str) -> String {
    crate::ui::format::group_signed_integer_str(value)
}

fn integer_words(value: f64) -> Option<String> {
    let integer = rounded_integer(value)?;

    if integer.unsigned_abs() > MAX_WORDS_INTEGER as u64 {
        return None;
    }

    if integer == 0 {
        return Some("Zero".to_string());
    }

    let mut chunks = Vec::new();
    let mut remaining = integer.unsigned_abs();
    let scales = ["", "Thousand", "Million", "Billion"];
    let mut scale_index = 0;

    while remaining > 0 {
        let chunk = (remaining % 1000) as u16;
        if chunk > 0 {
            let mut chunk_words = chunk_to_words(chunk);
            if let Some(scale) = scales.get(scale_index)
                && !scale.is_empty()
            {
                chunk_words.push(' ');
                chunk_words.push_str(scale);
            }
            chunks.push(chunk_words);
        }
        remaining /= 1000;
        scale_index += 1;
    }

    chunks.reverse();
    let mut sentence = chunks.join(" ");
    if integer < 0 {
        sentence = format!("Negative {sentence}");
    }

    Some(sentence)
}

fn chunk_to_words(value: u16) -> String {
    const BELOW_TWENTY: [&str; 20] = [
        "",
        "One",
        "Two",
        "Three",
        "Four",
        "Five",
        "Six",
        "Seven",
        "Eight",
        "Nine",
        "Ten",
        "Eleven",
        "Twelve",
        "Thirteen",
        "Fourteen",
        "Fifteen",
        "Sixteen",
        "Seventeen",
        "Eighteen",
        "Nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
    ];

    let hundreds = value / 100;
    let remainder = value % 100;
    let mut parts = Vec::new();

    if hundreds > 0 {
        parts.push(format!("{} Hundred", BELOW_TWENTY[hundreds as usize]));
    }

    if remainder > 0 {
        if remainder < 20 {
            parts.push(BELOW_TWENTY[remainder as usize].to_string());
        } else {
            let tens = remainder / 10;
            let ones = remainder % 10;
            if ones == 0 {
                parts.push(TENS[tens as usize].to_string());
            } else {
                parts.push(format!(
                    "{} {}",
                    TENS[tens as usize], BELOW_TWENTY[ones as usize]
                ));
            }
        }
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::{calculation_preview, chunk_to_words, format_number};

    #[test]
    fn detects_arithmetic_expression_with_operator() {
        let preview = calculation_preview("334+333").expect("expression should parse");

        assert_eq!(preview.formatted_value, "667");
        let words = preview
            .words
            .as_deref()
            .expect("whole-number result should expose spoken form");
        assert!(words.contains("Hundred"));
        assert!(words.contains("Sixty") || words.contains("Six"));
    }

    #[test]
    fn plain_number_does_not_trigger_calculator_preview() {
        assert!(calculation_preview("334").is_none());
    }

    #[test]
    fn honors_parentheses_and_precedence() {
        let preview = calculation_preview("(10 + 2) * 3").expect("expression should parse");
        assert_eq!(preview.formatted_value, "36");
    }

    #[test]
    fn invalid_or_partial_expressions_are_ignored() {
        assert!(calculation_preview("1234*").is_none());
        assert!(calculation_preview("abc+123").is_none());
    }

    #[test]
    fn division_by_zero_is_rejected() {
        assert!(calculation_preview("6 / 0").is_none());
    }

    #[test]
    fn formatter_adds_grouping_and_trims_fraction_zeros() {
        assert_eq!(format_number(12345.0), "12,345");
        assert_eq!(format_number(12345.5000), "12,345.5");
    }

    #[test]
    fn chunk_wording_handles_hundreds() {
        let one_hundred_five = chunk_to_words(105);
        let nine_ninety_nine = chunk_to_words(999);

        assert!(one_hundred_five.contains("Hundred"));
        assert!(!one_hundred_five.is_empty());
        assert!(nine_ninety_nine.contains("Hundred"));
        assert!(!nine_ninety_nine.is_empty());
    }
}
