//! # Unit tests for config::parse_helpers::parse_fields
use super::super::{DamoclesError, DamoclesParser, KeywordParseInfo, TokenInfo};
use crate::config::textloc::TextLoc;

//-------------------
// parse_fields
//-------------------

// --- Helper ---

#[derive(Debug, Default)]
struct DamoclesTester {
    fields: Vec<KeywordParseInfo>,
    errors: Vec<Box<dyn std::error::Error>>,
}

impl DamoclesParser for DamoclesTester {
    fn new_field(&mut self, kpi: KeywordParseInfo) {
        // Just save it for later use
        self.fields.push(kpi);
    }

    fn cmd(&mut self, _cmd: &TokenInfo) {}

    fn error(&mut self, e: Box<dyn std::error::Error>) {
        self.errors.push(e);
    }

    fn loc(&self, pos: (usize, usize)) -> TextLoc {
        TextLoc::from((String::from("test.txt"), pos.0, pos.1))
    }
}

/// Collect all (key, value, loc) triples produced by parse_fields
fn collect_fields_without_errors(input: &str) -> Vec<KeywordParseInfo> {
    let mut tester = DamoclesTester::default();

    tester.parse_fields(input);
    if !tester.errors.is_empty() {
        dbg!(&tester.errors);
    }

    assert!(tester.errors.is_empty());

    tester.fields
}
/// Collect all (key, value, loc) triples produced by parse_fields
fn collect_fields_with_errors(
    input: &str,
) -> (Vec<KeywordParseInfo>, Vec<Box<dyn std::error::Error>>) {
    let mut tester = DamoclesTester::default();

    tester.parse_fields(input);

    (tester.fields, tester.errors)
}
// =========================================================
// Basic parsing
// =========================================================

#[test]
fn test_single_field() {
    let fields = collect_fields_without_errors("KEY = value");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].keyname(), "KEY");
    assert_eq!(fields[0].values.len(), 1);
    assert_eq!(fields[0].values[0].token, "value");
}

#[test]
fn test_multiple_fields() {
    let input = "KEY1 = value1\nKEY2 = value2\nKEY3 = value3";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 3);
    assert_eq!(
        fields[0].key,
        TokenInfo {
            token: "KEY1".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 1, 1)),
            end_pos: TextLoc::from(("test.txt".to_string(), 1, 4)),
        }
    );
    assert_eq!(
        fields[0].values,
        vec![TokenInfo {
            token: "value1".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 1, 8)),
            end_pos: TextLoc::from(("test.txt".to_string(), 1, 13))
        }]
    );

    assert_eq!(
        fields[1].key,
        TokenInfo {
            token: "KEY2".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 2, 1)),
            end_pos: TextLoc::from(("test.txt".to_string(), 2, 4)),
        }
    );
    assert_eq!(
        fields[1].values,
        vec![TokenInfo {
            token: "value2".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 2, 8)),
            end_pos: TextLoc::from(("test.txt".to_string(), 2, 13))
        }]
    );

    assert_eq!(
        fields[2].key,
        TokenInfo {
            token: "KEY3".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 3, 1)),
            end_pos: TextLoc::from(("test.txt".to_string(), 3, 4)),
        }
    );
    assert_eq!(
        fields[2].values,
        vec![TokenInfo {
            token: "value3".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 3, 8)),
            end_pos: TextLoc::from(("test.txt".to_string(), 3, 13))
        }]
    );
}

#[test]
fn test_empty_input_produces_no_fields() {
    assert!(collect_fields_without_errors("").is_empty());
}

#[test]
fn test_whitespace_only_produces_no_fields() {
    assert!(collect_fields_without_errors("   \n\n   \n").is_empty());
}

#[test]
fn test_empty_value() {
    let (fields, errors) = collect_fields_with_errors("KEY =");
    assert_eq!(fields.len(), 0);
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_value_is_trimmed() {
    let fields = collect_fields_without_errors("KEY =   hello   ");
    assert_eq!(fields[0].values[0].token, "hello");
}

#[test]
fn test_lowercase_key() {
    // Lowercase keys are not recognized as new fields
    let fields = collect_fields_without_errors("key = value");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].keyname(), "key");
    assert_eq!(fields[0].valuenames()[0], "value");
}

#[test]
fn test_key_with_underscore_and_digits() {
    let fields = collect_fields_without_errors("MY_KEY_1 = value");
    assert_eq!(fields[0].keyname(), "MY_KEY_1");
}

// =========================================================
// Multiline values
// =========================================================

#[test]
fn test_multiline_value() {
    let input = "KEY = line1;\nline2;\nline3";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].keyname(), "KEY");
    assert_eq!(fields[0].values.len(), 3);
    assert_eq!(fields[0].valuenames(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_multiline_value_followed_by_new_key() {
    let input = "KEY1 = line1.0  ; line1.1 ; \nline2\nKEY2 = value2";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].keyname(), "KEY1");
    assert_eq!(fields[0].valuenames(), vec!["line1.0", "line1.1", "line2"]);
    assert_eq!(fields[1].keyname(), "KEY2");
    assert_eq!(fields[1].values[0].token, "value2");
}

#[test]
fn test_empty_lines_between_fields_ignored() {
    let input = "KEY1 = value1\n\n\nKEY2 = value2";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].keyname(), "KEY1");
    assert_eq!(fields[0].values[0].token, "value1");
    assert_eq!(fields[1].keyname(), "KEY2");
    assert_eq!(fields[1].values[0].token, "value2");
}

// =========================================================
// Single-quoted multiline values
// =========================================================

#[test]
fn test_single_quoted_value_on_one_line() {
    let input = "KEY = 'hello world'";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields[0].values[0].token, "hello world");
}

#[test]
fn test_quoted_multiline_value_preserves_content() {
    // Opening quote on first line, closing on third
    let input = "KEY = 'line1\nline2\nline3'";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].valuenames()[0], "line1\nline2\nline3");
}

#[test]
fn test_quoted_block_does_not_start_new_key() {
    // A line that looks like "KEY = value" inside quotes should not be a new field
    let input = "KEY = 'start\nFAKEKEY = not a key\nend'";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].keyname(), "KEY");
    assert_eq!(
        fields[0].values,
        vec![TokenInfo {
            token: "start\nFAKEKEY = not a key\nend".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 1, 7)),
            end_pos: TextLoc::from(("test.txt".to_string(), 3, 4)),
        }]
    );
}

#[test]
fn test_escaped_quote_in_quote() {
    let input = "KEY = 'it''s fine'\nKEY2 = value2";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].keyname(), "KEY");
    assert_eq!(
        fields[0].values,
        vec![TokenInfo {
            token: "it's fine".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 1, 7)),
            end_pos: TextLoc::from(("test.txt".to_string(), 1, 18))
        }]
    );
    assert_eq!(fields[1].keyname(), "KEY2");
    assert_eq!(
        fields[1].values,
        vec![TokenInfo {
            token: "value2".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 2, 8)),
            end_pos: TextLoc::from(("test.txt".to_string(), 2, 13))
        }]
    );
}

#[test]
fn test_multiple_quoted_values() {
    let input = "KEY1 = 'value1'\nKEY2 = 'value2'";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].keyname(), "KEY1");
    assert_eq!(
        fields[0].values,
        vec![TokenInfo {
            token: "value1".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 1, 8)),
            end_pos: TextLoc::from(("test.txt".to_string(), 1, 15))
        }]
    );
    assert_eq!(fields[1].keyname(), "KEY2");
    assert_eq!(
        fields[1].values,
        vec![TokenInfo {
            token: "value2".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 2, 8)),
            end_pos: TextLoc::from(("test.txt".to_string(), 2, 15))
        }]
    );
}

#[test]
fn test_quoted_block_preserves_indentation() {
    let input = "KEY =\n'  indented line\n  another'";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].keyname(), "KEY");
    assert_eq!(
        fields[0].values,
        vec![TokenInfo {
            token: "  indented line\n  another".into(),
            start_pos: TextLoc::from(("test.txt".to_string(), 2, 1)),
            end_pos: TextLoc::from(("test.txt".to_string(), 3, 10))
        }]
    );
}

// =========================================================
// Edge cases
// =========================================================

#[test]
fn test_value_with_equals_sign_on_next_line() {
    let input = "KEY\n=\nvalue";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 1);
}

#[test]
fn test_two_assignment_on_the_same_line() {
    let input = "KEY=value key with spaces = value2";
    let fields = collect_fields_without_errors(input);
    assert_eq!(fields.len(), 2);
}

#[test]
fn test_value_with_equals_sign() {
    // '=' in the value part should not start a new key
    let (fields, mut errors) = collect_fields_with_errors("KEY = a=b=c");
    assert_eq!(fields.len(), 2);
    assert_eq!(errors.len(), 1);
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<DamoclesError>());
    let parse_error: Box<DamoclesError> = err0.downcast().expect("not a ParseFieldsErrors");

    assert!(matches!(
        *parse_error,
        DamoclesError::UnexpectedAssignment { .. }
    ));
}

#[test]
fn test_continuation_line_before_any_key_is_ignored() {
    // Lines before the first key have no current_key, so they are dropped
    let input = "orphan line\nKEY = value";
    let (fields, mut errors) = collect_fields_with_errors(input);
    assert_eq!(fields.len(), 1);
    assert_eq!(errors.len(), 1);
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<DamoclesError>());
    let parse_error: Box<DamoclesError> = err0.downcast().expect("not a ParseFieldsErrors");

    assert!(matches!(
        *parse_error,
        DamoclesError::MissingAssignment { .. }
    ));
}

#[test]
fn test_last_field_is_not_dropped() {
    // Regression: the final key must be emitted even without a trailing newline
    let fields = collect_fields_without_errors("KEY1 = v1\nKEY2 = v2");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[1].keyname(), "KEY2");
}

#[test]
fn test_single_field_no_trailing_newline() {
    let fields = collect_fields_without_errors("KEY = value");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].valuenames()[0], "value");
}
