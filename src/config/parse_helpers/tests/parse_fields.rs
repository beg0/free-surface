//! # Unit tests for config::parse_helpers::parse_fields
use super::super::{parse_fields, TextLoc};

//-------------------
// parse_fields
//-------------------

// --- Helper ---

/// Collect all (key, value, loc) triples produced by parse_fields
fn collect_fields(input: &str) -> Vec<(String, String, TextLoc)> {
    let initial_pos = TextLoc::from(("test.txt", 1));
    let mut fields = Vec::new();
    parse_fields(input, &initial_pos, |key, value, loc| {
        fields.push((key, value, loc));
    });
    fields
}

// =========================================================
// Basic parsing
// =========================================================

#[test]
fn test_single_field() {
    let fields = collect_fields("KEY = value");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "KEY");
    assert_eq!(fields[0].1, "value");
}

#[test]
fn test_multiple_fields() {
    let input = "KEY1 = value1\nKEY2 = value2\nKEY3 = value3";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 3);
    assert_eq!(
        fields[0],
        (
            "KEY1".into(),
            "value1".into(),
            TextLoc::from(("test.txt", 1))
        )
    );
    assert_eq!(
        fields[1],
        (
            "KEY2".into(),
            "value2".into(),
            TextLoc::from(("test.txt", 2))
        )
    );
    assert_eq!(
        fields[2],
        (
            "KEY3".into(),
            "value3".into(),
            TextLoc::from(("test.txt", 3))
        )
    );
}

#[test]
fn test_key_is_uppercased() {
    // parse_fields uppercases the key regardless of input case
    let fields = collect_fields("KEY = value");
    assert_eq!(fields[0].0, "KEY");
}

#[test]
fn test_empty_input_produces_no_fields() {
    assert!(collect_fields("").is_empty());
}

#[test]
fn test_whitespace_only_produces_no_fields() {
    assert!(collect_fields("   \n\n   \n").is_empty());
}

#[test]
fn test_empty_value() {
    let fields = collect_fields("KEY =");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "KEY");
    assert_eq!(fields[0].1, "");
}

#[test]
fn test_value_is_trimmed() {
    let fields = collect_fields("KEY =   hello   ");
    assert_eq!(fields[0].1, "hello");
}

#[test]
fn test_lowercase_key_not_parsed_as_field() {
    // Lowercase keys are not recognized as new fields
    let fields = collect_fields("key = value");
    assert!(fields.is_empty());
}

#[test]
fn test_key_with_underscore_and_digits() {
    let fields = collect_fields("MY_KEY_1 = value");
    assert_eq!(fields[0].0, "MY_KEY_1");
}

// =========================================================
// Multiline values
// =========================================================

#[test]
fn test_multiline_value() {
    let input = "KEY = line1\nline2\nline3";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].1, "line1\nline2\nline3");
}

#[test]
fn test_multiline_value_followed_by_new_key() {
    let input = "KEY1 = line1\nline2\nKEY2 = value2";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].0, "KEY1");
    assert_eq!(fields[0].1, "line1\nline2");
    assert_eq!(fields[1].0, "KEY2");
    assert_eq!(fields[1].1, "value2");
}

#[test]
fn test_empty_lines_between_fields_ignored() {
    let input = "KEY1 = value1\n\n\nKEY2 = value2";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].1, "value1");
    assert_eq!(fields[1].1, "value2");
}

// =========================================================
// Single-quoted multiline values
// =========================================================

#[test]
fn test_single_quoted_value_on_one_line() {
    let input = "KEY = 'hello world'";
    let fields = collect_fields(input);
    assert_eq!(fields[0].1, "'hello world'");
}

#[test]
fn test_quoted_multiline_value_preserves_content() {
    // Opening quote on first line, closing on third
    let input = "KEY = 'line1\nline2\nline3'";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 1);
    assert!(fields[0].1.contains("line1"));
    assert!(fields[0].1.contains("line2"));
    assert!(fields[0].1.contains("line3"));
}

#[test]
fn test_quoted_block_does_not_start_new_key() {
    // A line that looks like "KEY = value" inside quotes should not be a new field
    let input = "KEY = 'start\nFAKEKEY = not a key\nend'";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "KEY");
}

#[test]
fn test_escaped_quote_does_not_toggle_in_quote() {
    // '' counts as 2 quotes (even), so in_quote should not toggle
    let input = "KEY = 'it''s fine'\nKEY2 = value2";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 2);
    assert!(fields[0].1.contains("it''s fine"));
    assert_eq!(fields[1].0, "KEY2");
}

#[test]
fn test_multiple_quoted_values() {
    let input = "KEY1 = 'value1'\nKEY2 = 'value2'";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].1, "'value1'");
    assert_eq!(fields[1].1, "'value2'");
}

#[test]
fn test_quoted_block_preserves_indentation() {
    let input = "KEY =\n'  indented line\n  another'";
    let fields = collect_fields(input);
    assert!(fields[0].1.contains("  indented line"));
    assert!(fields[0].1.contains("  another"));
}

// =========================================================
// Location tracking
// =========================================================

#[test]
fn test_location_line_number_first_field() {
    let fields = collect_fields("KEY = value");
    assert_eq!(fields[0].2.line(), 1);
}

#[test]
fn test_location_line_number_second_field() {
    let input = "KEY1 = value1\nKEY2 = value2";
    let fields = collect_fields(input);
    // KEY2 is on line 2 (line_idx=1, offset by initial_pos line=1)
    assert_eq!(fields[1].2.line(), 2);
}

#[test]
fn test_location_line_number_after_blank_lines() {
    let input = "KEY1 = value1\n\n\nKEY2 = value2";
    let fields = collect_fields(input);
    assert_eq!(fields[1].2.line(), 4);
}

#[test]
fn test_location_filename_preserved() {
    let fields = collect_fields("KEY = value");
    assert_eq!(fields[0].2.filename(), "test.txt");
}

#[test]
fn test_initial_pos_line_offset_applied() {
    // If initial_pos starts at line 10, all locations should be offset
    let initial_pos = TextLoc::from(("file.txt", 10));
    let mut fields = Vec::new();
    parse_fields("KEY1 = v1\nKEY2 = v2", &initial_pos, |k, v, loc| {
        fields.push((k, v, loc));
    });
    assert_eq!(fields[0].2.line(), 10);
    assert_eq!(fields[1].2.line(), 11);
}

// =========================================================
// Edge cases
// =========================================================

#[test]
fn test_value_with_equals_sign() {
    // '=' in the value part should not start a new key
    let fields = collect_fields("KEY = a=b=c");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].1, "a=b=c");
}

#[test]
fn test_continuation_line_before_any_key_is_ignored() {
    // Lines before the first key have no current_key, so they are dropped
    let input = "orphan line\nKEY = value";
    let fields = collect_fields(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "KEY");
}

#[test]
fn test_last_field_is_not_dropped() {
    // Regression: the final key must be emitted even without a trailing newline
    let fields = collect_fields("KEY1 = v1\nKEY2 = v2");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[1].0, "KEY2");
}

#[test]
fn test_single_field_no_trailing_newline() {
    let fields = collect_fields("KEY = value");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].1, "value");
}
