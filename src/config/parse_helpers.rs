//! # Parse helpers
//!
//! Helper functions to parse config files
use super::textloc::TextLoc;

/// Remove surrounding single quotes, and unescape '' -> '
pub fn unquote_single(s: &str) -> String {
    let inner = if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    };
    inner.replace("''", "'")
}

/// Parse a FORTRAN float literal, handling both single and double precision.
///
/// Supported formats:
///   - Standard:        3.14,  -3.14,  .5,  5.
///   - Single precision exponent:  3.14E+2,  3.14E2,  3.14E-2
///   - Double precision exponent:  3.14D+2,  3.14D2,  3.14D-2
///   - No mantissa decimal: 314E-2, 314D-2
pub fn parse_fortran_float(s: &str) -> Result<f64, std::num::ParseFloatError> {
    // Normalize: replace FORTRAN double precision 'D'/'d' exponent marker with 'e'
    // and strip any explicit '+' from the exponent (Rust's parser handles +/- already)
    let normalized: String = s
        .chars()
        .map(|c| match c {
            'D' | 'd' => 'e',
            'E' | 'e' => 'e',
            _ => c,
        })
        .collect();

    normalized.parse::<f64>()
}

/// Parse "key = value" pairs from a block, handling multiline values.
/// A new key starts when a line matches "IDENTIFIER = ...".
pub fn parse_fields<T: FnMut(String, String, TextLoc)>(
    input: &str,
    initial_pos: &TextLoc,
    mut new_field: T,
) {
    let mut current_key: Option<String> = None;
    let mut current_key_line: usize = initial_pos.line();
    let mut current_value = String::new();
    let mut in_quote = false;

    for (line_idx, line) in input.lines().enumerate() {
        let trimmed = if in_quote {
            line.trim_end()
        } else {
            line.trim()
        };

        if !in_quote {
            if trimmed.is_empty() {
                continue;
            }

            // Check if this line starts a new key: "KEYWORD = ..."
            // A key is all-uppercase (and underscores/digits), followed by " = "
            if let Some(eq_pos) = find_key_assignment(trimmed, true) {
                let candidate_key = trimmed[..eq_pos].trim().to_uppercase();

                // Save the previous key
                if let Some(key) = current_key.take() {
                    new_field(
                        key,
                        current_value.trim().to_string(),
                        initial_pos.clone_with_line_offset(current_key_line),
                    );
                }
                current_key = Some(candidate_key);
                current_key_line = line_idx;
                current_value = trimmed[eq_pos + 1..].trim().to_string();
                continue;
            }
        }

        let quote_count = trimmed.chars().filter(|c| *c == '\'').count();

        // If there is an even number of quote, it means we either close or open a quote
        // block
        if (quote_count % 2) == 1 {
            in_quote = !in_quote
        }

        // Continuation of current value
        if current_key.is_some() {
            current_value.push('\n');
            current_value.push_str(trimmed);
        }
    }

    // Don't forget the last key
    if let Some(key) = current_key {
        new_field(
            key,
            current_value.trim().to_string(),
            initial_pos.clone_with_line_offset(current_key_line),
        );
    }
}

/// Returns the position of '=' if the line looks like "KEY = value"
/// where KEY is uppercase letters, digits, underscores, and spaces.
pub fn find_key_assignment(line: &str, check_key: bool) -> Option<usize> {
    let eq_pos = line.find([':', '='])?;
    let key_part = line[..eq_pos].trim();
    // Key must be non-empty and contain only uppercase letters, digits, underscores
    if key_part.is_empty() {
        return None;
    }
    let valid = !check_key
        || key_part
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if valid {
        Some(eq_pos)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- No quotes ---

    #[test]
    fn test_unquoted_empty_string() {
        assert_eq!(unquote_single(""), "");
    }

    #[test]
    fn test_unquoted_with_spaces() {
        assert_eq!(unquote_single("hello world"), "hello world");
    }

    // --- Surrounding quotes removed ---

    #[test]
    fn test_quoted_string() {
        assert_eq!(unquote_single("'hello'"), "hello");
    }

    #[test]
    fn test_quoted_empty_string() {
        assert_eq!(unquote_single("''"), "");
    }

    #[test]
    fn test_quoted_string_with_spaces() {
        assert_eq!(unquote_single("'hello world'"), "hello world");
    }

    #[test]
    fn test_quoted_single_character() {
        assert_eq!(unquote_single("'x'"), "x");
    }

    // --- Escape sequence '' -> ' ---

    #[test]
    fn test_escaped_quote_in_quoted_string() {
        assert_eq!(unquote_single("'it''s'"), "it's");
    }

    #[test]
    fn test_escaped_quote_at_start_of_quoted_string() {
        assert_eq!(unquote_single("'''hello'"), "'hello");
    }

    #[test]
    fn test_escaped_quote_at_end_of_quoted_string() {
        assert_eq!(unquote_single("'hello'''"), "hello'");
    }

    #[test]
    fn test_multiple_escaped_quotes_in_quoted_string() {
        assert_eq!(unquote_single("'it''s a l''amour'"), "it's a l'amour");
    }

    #[test]
    fn test_escaped_quote_in_unquoted_string() {
        // No surrounding quotes: '' is still unescaped to '
        assert_eq!(unquote_single("it''s"), "it's");
    }

    // --- Edge cases ---

    #[test]
    fn test_single_quote_alone_is_not_unquoted() {
        // A lone "'" has len < 2 for the starts+ends check to be meaningful,
        // but starts_with and ends_with both match the same character -
        // len >= 2 guard prevents stripping, so it stays as-is.
        assert_eq!(unquote_single("'"), "'");
    }

    #[test]
    fn test_only_opening_quote_not_stripped() {
        assert_eq!(unquote_single("'hello"), "'hello");
    }

    #[test]
    fn test_only_closing_quote_not_stripped() {
        assert_eq!(unquote_single("hello'"), "hello'");
    }

    #[test]
    fn test_double_quoted_string_not_stripped() {
        // Double quotes are not handled by this function
        assert_eq!(unquote_single("\"hello\""), "\"hello\"");
    }

    #[test]
    fn test_quoted_whitespace_only() {
        assert_eq!(unquote_single("'   '"), "   ");
    }

    // ------------------------------
    // parse_fortran_float
    // ------------------------------

    // Helper for float comparison with tolerance
    fn assert_float_eq(result: Result<f64, std::num::ParseFloatError>, expected: f64) {
        let val = result.expect("expected Ok but got Err");
        assert!(
            (val - expected).abs() < 1e-10 * expected.abs().max(1.0),
            "expected {}, got {}",
            expected,
            val
        );
    }

    // --- Plain decimals (no exponent) ---

    #[test]
    fn test_plain_integer_looking() {
        assert_float_eq(parse_fortran_float("42"), 42.0);
    }

    #[test]
    fn test_plain_decimal() {
        assert_float_eq(parse_fortran_float("3.14"), 3.14);
    }

    #[test]
    fn test_leading_dot() {
        assert_float_eq(parse_fortran_float(".5"), 0.5);
    }

    #[test]
    fn test_trailing_dot() {
        assert_float_eq(parse_fortran_float("5."), 5.0);
    }

    #[test]
    fn test_negative() {
        assert_float_eq(parse_fortran_float("-3.14"), -3.14);
    }

    #[test]
    fn test_positive_sign() {
        assert_float_eq(parse_fortran_float("+3.14"), 3.14);
    }

    #[test]
    fn test_zero() {
        assert_float_eq(parse_fortran_float("0.0"), 0.0);
    }

    #[test]
    fn test_negative_zero() {
        let val = parse_fortran_float("-0.0").unwrap();
        assert_eq!(val, 0.0);
    }

    // --- Single precision: E exponent ---

    #[test]
    fn test_e_exponent_positive() {
        assert_float_eq(parse_fortran_float("3.14E2"), 314.0);
    }

    #[test]
    fn test_e_exponent_explicit_positive_sign() {
        assert_float_eq(parse_fortran_float("3.14E+2"), 314.0);
    }

    #[test]
    fn test_e_exponent_negative() {
        assert_float_eq(parse_fortran_float("3.14E-2"), 0.0314);
    }

    #[test]
    fn test_e_exponent_zero() {
        assert_float_eq(parse_fortran_float("3.14E0"), 3.14);
    }

    #[test]
    fn test_e_uppercase() {
        assert_float_eq(parse_fortran_float("1.0E3"), 1000.0);
    }

    #[test]
    fn test_e_lowercase() {
        assert_float_eq(parse_fortran_float("1.0e3"), 1000.0);
    }

    #[test]
    fn test_e_no_decimal_in_mantissa() {
        assert_float_eq(parse_fortran_float("314E-2"), 3.14);
    }

    #[test]
    fn test_e_leading_dot_mantissa() {
        assert_float_eq(parse_fortran_float(".314E1"), 3.14);
    }

    #[test]
    fn test_e_trailing_dot_mantissa() {
        assert_float_eq(parse_fortran_float("3.E2"), 300.0);
    }

    #[test]
    fn test_e_negative_mantissa() {
        assert_float_eq(parse_fortran_float("-1.5E2"), -150.0);
    }

    // --- Double precision: D exponent ---

    #[test]
    fn test_d_exponent_positive() {
        assert_float_eq(parse_fortran_float("3.14D2"), 314.0);
    }

    #[test]
    fn test_d_exponent_explicit_positive_sign() {
        assert_float_eq(parse_fortran_float("3.14D+2"), 314.0);
    }

    #[test]
    fn test_d_exponent_negative() {
        assert_float_eq(parse_fortran_float("3.14D-2"), 0.0314);
    }

    #[test]
    fn test_d_exponent_zero() {
        assert_float_eq(parse_fortran_float("3.14D0"), 3.14);
    }

    #[test]
    fn test_d_uppercase() {
        assert_float_eq(parse_fortran_float("1.0D3"), 1000.0);
    }

    #[test]
    fn test_d_lowercase() {
        assert_float_eq(parse_fortran_float("1.0d3"), 1000.0);
    }

    #[test]
    fn test_d_no_decimal_in_mantissa() {
        assert_float_eq(parse_fortran_float("314D-2"), 3.14);
    }

    #[test]
    fn test_d_leading_dot_mantissa() {
        assert_float_eq(parse_fortran_float(".314D1"), 3.14);
    }

    #[test]
    fn test_d_trailing_dot_mantissa() {
        assert_float_eq(parse_fortran_float("3.D2"), 300.0);
    }

    #[test]
    fn test_d_negative_mantissa() {
        assert_float_eq(parse_fortran_float("-1.5D2"), -150.0);
    }

    // --- Special values ---

    #[test]
    fn test_very_large_exponent() {
        assert_float_eq(parse_fortran_float("1.0D300"), 1.0e300);
    }

    #[test]
    fn test_very_small_exponent() {
        assert_float_eq(parse_fortran_float("1.0D-300"), 1.0e-300);
    }

    // --- Invalid inputs ---

    #[test]
    fn test_empty_string_is_error() {
        assert!(parse_fortran_float("").is_err());
    }

    #[test]
    fn test_plain_text_is_error() {
        assert!(parse_fortran_float("abc").is_err());
    }

    #[test]
    fn test_double_dot_is_error() {
        assert!(parse_fortran_float("1.2.3").is_err());
    }

    #[test]
    fn test_double_exponent_is_error() {
        assert!(parse_fortran_float("1.0E2E3").is_err());
    }

    #[test]
    fn test_exponent_only_is_error() {
        assert!(parse_fortran_float("E5").is_err());
    }

    #[test]
    fn test_exponent_with_no_value_is_error() {
        assert!(parse_fortran_float("1.0E").is_err());
    }
}
