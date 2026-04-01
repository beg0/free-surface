//! # Unit tests for config::parse_helpers::find_key_assignment
use super::super::find_key_assignment;

fn key_always_valid(_: &str) -> bool {
    true
}

fn key_is_uppercase(v: &str) -> bool {
    v.chars().all(|c| c.is_ascii_uppercase())
}

// =========================================================
// check_key = true  (strict: key must be uppercase/digits/underscores)
// =========================================================

// --- Returns Some ---

#[test]
fn test_simple_uppercase_key_equals() {
    assert_eq!(
        find_key_assignment("KEY = value", key_always_valid),
        Some(4)
    );
}

#[test]
fn test_simple_uppercase_key_colon() {
    assert_eq!(
        find_key_assignment("KEY : value", key_always_valid),
        Some(4)
    );
}

#[test]
fn test_key_with_digits() {
    assert_eq!(
        find_key_assignment("KEY1 = value", key_always_valid),
        Some(5)
    );
}

#[test]
fn test_key_with_underscores() {
    assert_eq!(
        find_key_assignment("MY_KEY = value", key_always_valid),
        Some(7)
    );
}

#[test]
fn test_key_with_underscores_and_digits() {
    assert_eq!(
        find_key_assignment("MY_KEY_1 = value", key_always_valid),
        Some(9)
    );
}

#[test]
fn test_no_spaces_around_equals() {
    assert_eq!(find_key_assignment("KEY=value", key_always_valid), Some(3));
}

#[test]
fn test_no_spaces_around_colon() {
    assert_eq!(find_key_assignment("KEY:value", key_always_valid), Some(3));
}

#[test]
fn test_single_char_key() {
    assert_eq!(find_key_assignment("A = x", key_always_valid), Some(2));
}

#[test]
fn test_value_contains_equals() {
    // Only the FIRST '=' is found
    assert_eq!(find_key_assignment("KEY = a=b", key_always_valid), Some(4));
}

#[test]
fn test_value_contains_colon() {
    assert_eq!(find_key_assignment("KEY = a:b", key_always_valid), Some(4));
}

#[test]
fn test_value_is_empty() {
    assert_eq!(find_key_assignment("KEY =", key_always_valid), Some(4));
}

// --- Returns None ---

#[test]
fn test_no_separator_returns_none() {
    assert_eq!(find_key_assignment("KEY value", key_always_valid), None);
}

#[test]
fn test_empty_string_returns_none() {
    assert_eq!(find_key_assignment("", key_always_valid), None);
}

#[test]
fn test_empty_key_equals_only_returns_none() {
    assert_eq!(find_key_assignment("= value", key_always_valid), None);
}

#[test]
fn test_empty_key_after_trim_returns_none() {
    assert_eq!(find_key_assignment("   = value", key_always_valid), None);
}

#[test]
fn test_lowercase_key_returns_none() {
    assert_eq!(find_key_assignment("key = value", key_is_uppercase), None);
}

#[test]
fn test_key_with_spaces_returns_none() {
    // Spaces in key are not allowed when check_key = true
    assert_eq!(
        find_key_assignment("MY KEY = value", key_is_uppercase),
        None
    );
}

#[test]
fn test_key_with_dot_returns_none() {
    assert_eq!(
        find_key_assignment("MY.KEY = value", key_is_uppercase),
        None
    );
}

// =========================================================
// check_key = false  (relaxed: any non-empty key is accepted)
// =========================================================

// --- Returns Some ---

#[test]
fn test_relaxed_lowercase_key() {
    assert_eq!(
        find_key_assignment("key = value", key_always_valid),
        Some(4)
    );
}

#[test]
fn test_relaxed_mixed_case_key() {
    assert_eq!(
        find_key_assignment("My Key = value", key_always_valid),
        Some(7)
    );
}

// --- Returns None ---

#[test]
fn test_relaxed_no_separator_returns_none() {
    assert_eq!(find_key_assignment("key value", key_always_valid), None);
}

// =========================================================
// Returned position correctness
// =========================================================

#[test]
fn test_returned_position_points_to_equals() {
    let line = "KEY = value";
    let pos = find_key_assignment(line, key_is_uppercase).unwrap();
    assert_eq!(&line[pos..pos + 1], "=");
}

#[test]
fn test_returned_position_points_to_colon() {
    let line = "KEY : value";
    let pos = find_key_assignment(line, key_is_uppercase).unwrap();
    assert_eq!(&line[pos..pos + 1], ":");
}

#[test]
fn test_value_recoverable_from_position() {
    let line = "KEY = hello world";
    let pos = find_key_assignment(line, key_is_uppercase).unwrap();
    assert_eq!(line[pos + 1..].trim(), "hello world");
}
