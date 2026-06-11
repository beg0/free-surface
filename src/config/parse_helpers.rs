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

/// Format a `f64` value in a Fortran-compatible scientific notation string.
///
/// The output follows Fortran's `ES` (engineering scientific) format:
/// - One digit before the decimal point
/// - Uppercase `E` separator
/// - Exponent always has an explicit sign (`+` or `-`)
/// - Exponent is zero-padded to at least 2 digits
/// - Mantissa always has an explicit sign (`+` or `-`)
///
/// The number of decimal digits in the mantissa is controlled by the
/// `precision` parameter, matching the Fortran format descriptor `ESw.d`
/// where `d` is the number of digits after the decimal point.
///
/// # Arguments
///
/// * `v` - The `f64` value to format
/// * `precision` - Number of digits after the decimal point in the mantissa.
///   Use `14` for full `REAL*8` precision (15 significant digits total),
///   or `6` for the common Selafin `ES13.6` descriptor.
///
/// # Examples
///
/// ```rust
/// use free_surface::config::parse_helpers::write_fortran_float_with_precision;
///
/// // Positive value
/// assert_eq!(write_fortran_float_with_precision(123.456, 6),    "+1.234560E+02");
///
/// // Negative value
/// assert_eq!(write_fortran_float_with_precision(-0.001, 6),     "-1.000000E-03");
///
/// // Zero
/// assert_eq!(write_fortran_float_with_precision(0.0, 6),        "+0.000000E+00");
///
/// // Negative exponent with single digit (zero-padded to 2)
/// assert_eq!(write_fortran_float_with_precision(1.5, 6),        "+1.500000E+00");
///
/// // Large exponent (more than 2 digits, no truncation)
/// assert_eq!(write_fortran_float_with_precision(1.0e100, 6),    "+1.000000E+100");
///
/// // Full REAL*8 precision (14 decimal digits = 15 significant digits)
/// assert_eq!(write_fortran_float_with_precision(123.456, 14),   "+1.23456000000000E+02");
///
/// // Negative zero is treated as positive zero
/// assert_eq!(write_fortran_float_with_precision(-0.0, 6),       "-0.000000E+00");
/// ```
pub fn write_fortran_float_with_precision(v: f64, precision: usize) -> String {
    // {:E} gives uppercase E and handles sign on the mantissa,
    // but the exponent has no leading zero and no forced sign.
    let s = format!("{:+.*E}", precision, v); // e.g. "+1.23456789012345E2" or "-1.23456789012345E-2"

    // Split on 'E' to fix the exponent part
    let (mantissa, exp_str) = s.split_once('E').unwrap();

    let exp: i32 = exp_str.parse().unwrap();
    format!("{mantissa}E{:+03}", exp) // {:+03} -> sign + at least 2 digits
}

/// Format a `f64` value in a Fortran-compatible scientific notation string.
///
/// The output follows Fortran's `ES` (engineering scientific) format:
/// - One digit before the decimal point
/// - Uppercase `E` separator
/// - Exponent always has an explicit sign (`+` or `-`)
/// - Exponent is zero-padded to at least 2 digits
/// - Mantissa always has an explicit sign (`+` or `-`)
///
/// The number of decimal digits in the mantissa is controlled by the
/// `precision` parameter, matching the Fortran format descriptor `ESw.d`
/// where `d` is the number of digits after the decimal point.
///
/// # Arguments
///
/// * `v` - The `f64` value to format
/// * `precision` - Number of digits after the decimal point in the mantissa.
///   Use `14` for full `REAL*8` precision (15 significant digits total),
///   or `6` for the common Selafin `ES13.6` descriptor.
///
/// # Examples
///
/// ```rust
/// use free_surface::config::parse_helpers::write_fortran_float;
///
/// // Positive value
/// assert_eq!(write_fortran_float(123.456),    "+1.23456E+02");
///
/// // Negative value
/// assert_eq!(write_fortran_float(-0.001),     "-1E-03");
///
/// // Zero
/// assert_eq!(write_fortran_float(0.0),        "+0E+00");
///
/// // Negative exponent with single digit (zero-padded to 2)
/// assert_eq!(write_fortran_float(1.5),        "+1.5E+00");
///
/// // Large exponent (more than 2 digits, no truncation)
/// assert_eq!(write_fortran_float(1.0e100),    "+1E+100");
///
/// // Negative zero is treated as positive zero
/// assert_eq!(write_fortran_float(-0.0),       "-0E+00");
/// ```
pub fn write_fortran_float(v: f64) -> String {
    // {:E} gives uppercase E and handles sign on the mantissa,
    // but the exponent has no leading zero and no forced sign.
    let s = format!("{:+.E}", v); // e.g. "+1.23456789012345E2" or "-1.23456789012345E-2"

    // Split on 'E' to fix the exponent part
    let (mantissa, exp_str) = s.split_once('E').unwrap();

    let exp: i32 = exp_str.parse().unwrap();
    format!("{mantissa}E{:+03}", exp) // {:+03} -> sign + at least 2 digits
}
