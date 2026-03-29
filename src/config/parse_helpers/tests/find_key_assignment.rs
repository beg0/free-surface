//! # Unit tests for config::parse_helpers::find_key_assignment
use super::super::find_key_assignment;

// =========================================================
// check_key = true  (strict: key must be uppercase/digits/underscores)
// =========================================================

// --- Returns Some ---

#[test]
fn test_simple_uppercase_key_equals() {
    assert_eq!(find_key_assignment("KEY = value", true), Some(4));
}

#[test]
fn test_simple_uppercase_key_colon() {
    assert_eq!(find_key_assignment("KEY : value", true), Some(4));
}

#[test]
fn test_key_with_digits() {
    assert_eq!(find_key_assignment("KEY1 = value", true), Some(5));
}

#[test]
fn test_key_with_underscores() {
    assert_eq!(find_key_assignment("MY_KEY = value", true), Some(7));
}

#[test]
fn test_key_with_underscores_and_digits() {
    assert_eq!(find_key_assignment("MY_KEY_1 = value", true), Some(9));
}

#[test]
fn test_no_spaces_around_equals() {
    assert_eq!(find_key_assignment("KEY=value", true), Some(3));
}

#[test]
fn test_no_spaces_around_colon() {
    assert_eq!(find_key_assignment("KEY:value", true), Some(3));
}

#[test]
fn test_single_char_key() {
    assert_eq!(find_key_assignment("A = x", true), Some(2));
}

#[test]
fn test_value_contains_equals() {
    // Only the FIRST '=' is found
    assert_eq!(find_key_assignment("KEY = a=b", true), Some(4));
}

#[test]
fn test_value_contains_colon() {
    assert_eq!(find_key_assignment("KEY = a:b", true), Some(4));
}

#[test]
fn test_value_is_empty() {
    assert_eq!(find_key_assignment("KEY =", true), Some(4));
}

// --- Returns None ---

#[test]
fn test_no_separator_returns_none() {
    assert_eq!(find_key_assignment("KEY value", true), None);
}

#[test]
fn test_empty_string_returns_none() {
    assert_eq!(find_key_assignment("", true), None);
}

#[test]
fn test_empty_key_equals_only_returns_none() {
    assert_eq!(find_key_assignment("= value", true), None);
}

#[test]
fn test_empty_key_after_trim_returns_none() {
    assert_eq!(find_key_assignment("   = value", true), None);
}

#[test]
fn test_lowercase_key_returns_none() {
    assert_eq!(find_key_assignment("key = value", true), None);
}

#[test]
fn test_mixed_case_key_returns_none() {
    assert_eq!(find_key_assignment("Key = value", true), None);
}

#[test]
fn test_key_with_spaces_returns_none() {
    // Spaces in key are not allowed when check_key = true
    assert_eq!(find_key_assignment("MY KEY = value", true), None);
}

#[test]
fn test_key_with_special_chars_returns_none() {
    assert_eq!(find_key_assignment("MY-KEY = value", true), None);
}

#[test]
fn test_key_with_dot_returns_none() {
    assert_eq!(find_key_assignment("MY.KEY = value", true), None);
}

// =========================================================
// check_key = false  (relaxed: any non-empty key is accepted)
// =========================================================

// --- Returns Some ---

#[test]
fn test_relaxed_lowercase_key() {
    assert_eq!(find_key_assignment("key = value", false), Some(4));
}

#[test]
fn test_relaxed_mixed_case_key() {
    assert_eq!(find_key_assignment("My Key = value", false), Some(7));
}

#[test]
fn test_relaxed_key_with_spaces() {
    assert_eq!(find_key_assignment("my key = value", false), Some(7));
}

#[test]
fn test_relaxed_key_with_special_chars() {
    assert_eq!(find_key_assignment("my-key = value", false), Some(7));
}

#[test]
fn test_relaxed_key_with_dots() {
    assert_eq!(find_key_assignment("my.key = value", false), Some(7));
}

#[test]
fn test_relaxed_uppercase_key_still_works() {
    assert_eq!(find_key_assignment("KEY = value", false), Some(4));
}

#[test]
fn test_relaxed_colon_separator() {
    assert_eq!(find_key_assignment("some key : value", false), Some(9));
}

// --- Returns None ---

#[test]
fn test_relaxed_no_separator_returns_none() {
    assert_eq!(find_key_assignment("key value", false), None);
}

#[test]
fn test_relaxed_empty_string_returns_none() {
    assert_eq!(find_key_assignment("", false), None);
}

#[test]
fn test_relaxed_empty_key_returns_none() {
    assert_eq!(find_key_assignment("= value", false), None);
}

#[test]
fn test_relaxed_whitespace_only_key_returns_none() {
    assert_eq!(find_key_assignment("   = value", false), None);
}

// =========================================================
// Returned position correctness
// =========================================================

#[test]
fn test_returned_position_points_to_equals() {
    let line = "KEY = value";
    let pos = find_key_assignment(line, true).unwrap();
    assert_eq!(&line[pos..pos + 1], "=");
}

#[test]
fn test_returned_position_points_to_colon() {
    let line = "KEY : value";
    let pos = find_key_assignment(line, true).unwrap();
    assert_eq!(&line[pos..pos + 1], ":");
}

#[test]
fn test_value_recoverable_from_position() {
    let line = "KEY = hello world";
    let pos = find_key_assignment(line, true).unwrap();
    assert_eq!(line[pos + 1..].trim(), "hello world");
}
