//! # Parse helpers
//!
//! Helper functions to parse config files

mod damocles;
mod keywordparseinfo;
mod locatedchars;

pub use damocles::*;

#[cfg(test)]
mod tests {
    mod parse_fortran_float;
    mod unquote_single;
}

/// Quote a string only if necessary, using single-quote with doubling escapes.
///
/// The quoting rules are:
/// - If the string contains **no whitespace and no single quotes**, it is
///   returned as-is.
/// - Otherwise it is wrapped in single quotes (`'...'`). Any single quote
///   character (`'`) inside the string is escaped by doubling it (`''`),
///   following the POSIX shell and SQL single-quote conventions.
///
/// # Examples
///
/// ```rust
/// use free_surface::config::parse_helpers::single_quote_if_needed;
///
/// // Plain word - no quoting needed
/// assert_eq!(single_quote_if_needed("hello"),          "hello");
///
/// // Contains a space - must be quoted
/// assert_eq!(single_quote_if_needed("hello world"),    "'hello world'");
///
/// // Contains a tab - must be quoted
/// assert_eq!(single_quote_if_needed("col1\tcol2"),     "'col1\tcol2'");
///
/// // Contains a single quote - must be quoted and escaped by doubling
/// assert_eq!(single_quote_if_needed("it's"),           "'it''s'");
///
/// // Contains both a space and a single quote
/// assert_eq!(single_quote_if_needed("it's fine"),      "'it''s fine'");
///
/// // Multiple consecutive single quotes
/// assert_eq!(single_quote_if_needed("''"),             "''''''");
///
/// // Empty string - no whitespace, no quotes, returned as-is
/// assert_eq!(single_quote_if_needed(""),               "");
///
/// // Already looks quoted - treated as plain text, not double-quoted
/// assert_eq!(single_quote_if_needed("'hello'"),        "'''hello'''");
/// ```
pub fn single_quote_if_needed(s: &str) -> String {
    let needs_quoting = s.contains(|c: char| c.is_whitespace() || c == '\'');

    if !needs_quoting {
        return s.to_string();
    }

    let escaped = s.replace('\'', "''");
    format!("'{escaped}'")
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
///
/// # Examples
///
/// ```rust
/// use free_surface::config::parse_helpers::parse_fortran_float;
///
/// assert_eq!(parse_fortran_float("0.31415E+1").unwrap(), 3.1415);
/// assert_eq!(parse_fortran_float("114.42D-2").unwrap(), 1.1442);
/// ```
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
