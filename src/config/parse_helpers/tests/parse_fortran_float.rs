//! # Unit tests for config::parse_helpers::parse_fortran_float
use super::super::parse_fortran_float;

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
