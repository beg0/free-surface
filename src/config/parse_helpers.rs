//! # Parse helpers
//!
//! Helper functions to parse config files

mod damocles;
mod keywordparseinfo;
mod locatedchars;

pub use damocles::*;

#[cfg(test)]
mod tests {
    mod find_key_assignment;
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
