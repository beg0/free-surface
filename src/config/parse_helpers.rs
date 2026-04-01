//! # Parse helpers
//!
//! Helper functions to parse config files
use super::textloc::TextLoc;

#[cfg(test)]
mod tests {
    mod find_key_assignment;
    mod parse_fields;
    mod parse_fortran_float;
    mod unquote_single;
}

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

fn toggle_single_quote(v: &str) -> bool {
    let quote_count = v.chars().filter(|c| *c == '\'').count();

    // If there is an even number of quote, it means we either close or open a quote
    // block
    (quote_count % 2) == 1
}

/// Parse "key = value" pairs from a block, handling multiline values.
/// A new key starts when a line matches "IDENTIFIER = ...".
pub fn parse_fields<T: FnMut(&str, String, TextLoc)>(
    input: &str,
    initial_pos: &TextLoc,
    mut new_field: T,
    key_validation_fct: fn(&str) -> bool,
) {
    let mut current_key: Option<&str> = None;
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
            if let Some(eq_pos) = find_key_assignment(trimmed, key_validation_fct) {
                let candidate_key = trimmed[..eq_pos].trim();

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

                if toggle_single_quote(current_value.as_str()) {
                    in_quote = !in_quote;
                }
                continue;
            }
        }

        if toggle_single_quote(trimmed) {
            in_quote = !in_quote;
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
pub fn find_key_assignment(line: &str, key_validation_fct: fn(&str) -> bool) -> Option<usize> {
    let eq_pos = line.find([':', '='])?;
    let key_part = line[..eq_pos].trim();
    // Key must be non-empty and contain only uppercase letters, digits, underscores
    if key_part.is_empty() {
        return None;
    }

    if key_validation_fct(key_part) {
        Some(eq_pos)
    } else {
        None
    }
}
