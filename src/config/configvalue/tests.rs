use crate::config::textloc::TextLoc;

use super::*;
use std::path::PathBuf;

const FILENAME: &str = "wonderful_file.txt";

// --- Helpers ---

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn paths(v: &[&str]) -> Vec<PathBuf> {
    v.iter().map(PathBuf::from).collect()
}

fn to_token(v: &str) -> TokenInfo {
    let start_pos = TextLoc::from((FILENAME, 5));
    let end_pos = start_pos.clone_with_line_offset_col(0, v.len());
    TokenInfo {
        token: v.to_owned(),
        start_pos,
        end_pos,
    }
}

fn to_tokens(values: &[&str]) -> Vec<TokenInfo> {
    let mut last_col: usize = 5;
    values
        .iter()
        .map(|v| {
            let start_pos = TextLoc::from((FILENAME, last_col + 5));
            let end_pos = start_pos.clone_with_line_offset_col(0, v.len());
            last_col = end_pos.column();
            TokenInfo {
                token: (*v).to_owned(),
                start_pos,
                end_pos,
            }
        })
        .collect()
}

// =========================================================
// parse_bool
// =========================================================

// --- True variants ---

#[test]
fn test_true_variants() {
    for val in &["true", "yes", "1", "on", "vrai", "oui"] {
        assert_eq!(parse_bool(val), Ok(true), "expected true for '{}'", val);
    }
}

#[test]
fn test_false_variants() {
    for val in &["false", "no", "0", "off", "faux", "non"] {
        assert_eq!(parse_bool(val), Ok(false), "expected false for '{}'", val);
    }
}

// --- Case insensitivity ---

#[test]
fn test_true_variants_uppercase() {
    for val in &["TRUE", "YES", "ON", "VRAI", "OUI"] {
        assert_eq!(parse_bool(val), Ok(true), "expected true for '{}'", val);
    }
}

#[test]
fn test_false_variants_uppercase() {
    for val in &["FALSE", "NO", "OFF", "FAUX", "NON"] {
        assert_eq!(parse_bool(val), Ok(false), "expected false for '{}'", val);
    }
}

#[test]
fn test_true_variants_mixed_case() {
    for val in &["True", "Yes", "On", "Vrai", "Oui"] {
        assert_eq!(parse_bool(val), Ok(true), "expected true for '{}'", val);
    }
}

#[test]
fn test_false_variants_mixed_case() {
    for val in &["False", "No", "Off", "Faux", "Non"] {
        assert_eq!(parse_bool(val), Ok(false), "expected false for '{}'", val);
    }
}

// --- Invalid values ---

#[test]
fn test_invalid_returns_error() {
    for val in &["maybe", "2", "yep", "nope", "oui oui", "", " "] {
        assert!(parse_bool(val).is_err(), "expected error for '{}'", val);
    }
}

#[test]
fn test_error_message_contains_input() {
    let input = "maybe";
    let err = parse_bool(input).unwrap_err();
    assert!(
        err.contains(input),
        "error message should contain the invalid input, got: '{}'",
        err
    );
}

// =========================================================
// parse_single_value
// =========================================================

// DicoType::String
// ----------------

#[test]
fn test_string_unquoted() {
    assert_eq!(
        parse_single_value_2(&to_token("hello"), &DicoType::String).unwrap(),
        ConfigValue::String("hello".into())
    );
}

#[test]
fn test_string_quoted() {
    assert_eq!(
        parse_single_value_2(&to_token("'hello world'"), &DicoType::String).unwrap(),
        ConfigValue::String("hello world".into())
    );
}

#[test]
fn test_string_empty() {
    assert_eq!(
        parse_single_value_2(&to_token(""), &DicoType::String).unwrap(),
        ConfigValue::String("".into())
    );
}

#[test]
fn test_string_empty_quoted() {
    assert_eq!(
        parse_single_value_2(&to_token("''"), &DicoType::String).unwrap(),
        ConfigValue::String("".into())
    );
}

#[test]
fn test_string_escaped_single_quote() {
    // '' inside quotes should be unescaped to '
    assert_eq!(
        parse_single_value_2(&to_token("'it''s fine'"), &DicoType::String).unwrap(),
        ConfigValue::String("it's fine".into())
    );
}

// DicoType::Integer
// -----------------

#[test]
fn test_integer_positive() {
    assert_eq!(
        parse_single_value_2(&to_token("42"), &DicoType::Integer).unwrap(),
        ConfigValue::Integer(42)
    );
}

#[test]
fn test_integer_negative() {
    assert_eq!(
        parse_single_value_2(&to_token("-7"), &DicoType::Integer).unwrap(),
        ConfigValue::Integer(-7)
    );
}

#[test]
fn test_integer_zero() {
    assert_eq!(
        parse_single_value_2(&to_token("0"), &DicoType::Integer).unwrap(),
        ConfigValue::Integer(0)
    );
}

#[test]
fn test_integer_max() {
    let raw = i64::MAX.to_string();
    assert_eq!(
        parse_single_value_2(&to_token(&raw), &DicoType::Integer).unwrap(),
        ConfigValue::Integer(i64::MAX)
    );
}

#[test]
fn test_integer_invalid_float() {
    assert!(parse_single_value_2(&to_token("3.14"), &DicoType::Integer).is_err());
}

#[test]
fn test_integer_invalid_word() {
    let token = to_token("notanumber");
    let (err_token, _msg) = parse_single_value_2(&token, &DicoType::Integer).unwrap_err();
    assert_eq!(err_token.token, "notanumber");
}

#[test]
fn test_integer_invalid_empty() {
    assert!(parse_single_value_2(&to_token(""), &DicoType::Integer).is_err());
}

// DicoType::Real
// --------------

#[test]
#[allow(clippy::approx_constant)]
fn test_real_decimal() {
    assert_eq!(
        parse_single_value_2(&to_token("3.14"), &DicoType::Real).unwrap(),
        ConfigValue::Float(3.14)
    );
}

#[test]
fn test_real_whole_number() {
    assert_eq!(
        parse_single_value_2(&to_token("42"), &DicoType::Real).unwrap(),
        ConfigValue::Float(42.0)
    );
}

#[test]
#[allow(clippy::approx_constant)]
fn test_real_negative() {
    assert_eq!(
        parse_single_value_2(&to_token("-2.718"), &DicoType::Real).unwrap(),
        ConfigValue::Float(-2.718)
    );
}

#[test]
fn test_real_scientific_notation() {
    assert_eq!(
        parse_single_value_2(&to_token("1.5e3"), &DicoType::Real).unwrap(),
        ConfigValue::Float(1500.0)
    );
}

#[test]
fn test_real_invalid_word() {
    let token = to_token("notafloat");
    let (err_token, _msg) = parse_single_value_2(&token, &DicoType::Real).unwrap_err();
    assert_eq!(err_token.token, "notafloat");
}

#[test]
fn test_real_invalid_empty() {
    assert!(parse_single_value_2(&to_token(""), &DicoType::Real).is_err());
}

// DicoType::Logical
// -----------------

#[test]
fn test_logical_true_variants() {
    for val in &["true", "yes", "1", "on", "vrai", "oui", "TRUE", "OUI"] {
        assert_eq!(
            parse_single_value_2(&to_token(val), &DicoType::Logical).unwrap(),
            ConfigValue::Boolean(true),
            "expected true for '{}'",
            val
        );
    }
}

#[test]
fn test_logical_false_variants() {
    for val in &["false", "no", "0", "off", "faux", "non", "FALSE", "NON"] {
        assert_eq!(
            parse_single_value_2(&to_token(val), &DicoType::Logical).unwrap(),
            ConfigValue::Boolean(false),
            "expected false for '{}'",
            val
        );
    }
}

#[test]
fn test_logical_invalid() {
    let token = to_token("maybe");
    let (err_token, _msg) = parse_single_value_2(&token, &DicoType::Logical).unwrap_err();
    assert_eq!(err_token.token, "maybe");
}

// Return type sanity - each DicoType maps to the right variant
// -------------------------------------------------------------

#[test]
fn test_string_returns_string_variant() {
    assert!(matches!(
        parse_single_value_2(&to_token("x"), &DicoType::String).unwrap(),
        ConfigValue::String(_)
    ));
}

#[test]
fn test_integer_returns_integer_variant() {
    assert!(matches!(
        parse_single_value_2(&to_token("1"), &DicoType::Integer).unwrap(),
        ConfigValue::Integer(_)
    ));
}

#[test]
fn test_real_returns_float_variant() {
    assert!(matches!(
        parse_single_value_2(&to_token("1.0"), &DicoType::Real).unwrap(),
        ConfigValue::Float(_)
    ));
}

#[test]
fn test_logical_returns_boolean_variant() {
    assert!(matches!(
        parse_single_value_2(&to_token("true"), &DicoType::Logical).unwrap(),
        ConfigValue::Boolean(_)
    ));
}

// =========================================================
// DicoType::String
// =========================================================

#[test]
fn test_string_collection_unquoted() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["alice", "bob", "charlie"]), &DicoType::String)
            .unwrap(),
        ConfigValue::StringCollection(vec!["alice".into(), "bob".into(), "charlie".into()])
    );
}

#[test]
fn test_string_collection_quoted() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["'alice'", "'bob'"]), &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["alice".into(), "bob".into()])
    );
}

#[test]
fn test_string_collection_mixed_quoted_unquoted() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["'alice'", "bob"]), &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["alice".into(), "bob".into()])
    );
}

#[test]
fn test_string_collection_with_escaped_quote() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["'it''s'", "'l''eau'"]), &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["it's".into(), "l'eau".into()])
    );
}

#[test]
fn test_string_collection_empty_strings() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["''", "''"]), &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["".into(), "".into()])
    );
}

#[test]
fn test_string_collection_single_element() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["hello"]), &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["hello".into()])
    );
}

#[test]
fn test_string_collection_empty_input() {
    assert_eq!(
        parse_collection_values_2(&Vec::new(), &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec![])
    );
}

// =========================================================
// DicoType::Integer
// =========================================================

#[test]
fn test_integer_collection() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["1", "2", "3"]), &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![1, 2, 3])
    );
}

#[test]
fn test_integer_collection_negative() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["-1", "0", "42"]), &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![-1, 0, 42])
    );
}

#[test]
fn test_integer_collection_single_element() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["99"]), &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![99])
    );
}

#[test]
fn test_integer_collection_empty_input() {
    assert_eq!(
        parse_collection_values_2(&Vec::new(), &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![])
    );
}

#[test]
fn test_integer_collection_one_invalid() {
    let tokens = to_tokens(&["1", "oops", "3"]);

    let errors = parse_collection_values_2(&tokens, &DicoType::Integer).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors.iter().any(|(ti, _)| ti.token == "oops"));
}

#[test]
fn test_integer_collection_multiple_invalid() {
    let tokens = to_tokens(&["bad", "1", "wrong"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Integer).unwrap_err();

    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|(ti, _)| ti.token == "bad"));
    assert!(errors.iter().any(|(ti, _)| ti.token == "wrong"));
}

#[test]
fn test_integer_collection_all_invalid() {
    let tokens = to_tokens(&["a", "b", "c"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Integer).unwrap_err();
    assert_eq!(errors.len(), 3);
    let errors_tokens: Vec<String> = errors.iter().map(|(ti, _)| ti.token.to_owned()).collect();
    assert!(errors_tokens.contains(&"a".to_string()));
    assert!(errors_tokens.contains(&"b".to_string()));
    assert!(errors_tokens.contains(&"c".to_string()));
}

#[test]
fn test_integer_collection_float_is_invalid() {
    let tokens = to_tokens(&["1", "3.14"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Integer).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors.iter().any(|(ti, _)| ti.token == "3.14"));
}

// =========================================================
// DicoType::Real
// =========================================================

#[test]
fn test_real_collection() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["1.1", "2.2", "3.3"]), &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![1.1, 2.2, 3.3])
    );
}

#[test]
fn test_real_collection_whole_numbers() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["1", "2", "3"]), &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![1.0, 2.0, 3.0])
    );
}

#[test]
fn test_real_collection_negative() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["-1.5", "0.0", "2.5"]), &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![-1.5, 0.0, 2.5])
    );
}

#[test]
fn test_real_collection_scientific_notation() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["1.5e3", "2.0e-2"]), &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![1500.0, 0.02])
    );
}

#[test]
#[allow(clippy::approx_constant)]
fn test_real_collection_single_element() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["3.14"]), &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![3.14])
    );
}

#[test]
fn test_real_collection_empty_input() {
    assert_eq!(
        parse_collection_values_2(&Vec::new(), &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![])
    );
}

#[test]
fn test_real_collection_one_invalid() {
    let tokens = to_tokens(&["1.0", "notafloat", "3.0"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Real).unwrap_err();

    assert_eq!(errors.len(), 1);
    assert!(errors.iter().any(|(ti, _)| ti.token == "notafloat"));
}

#[test]
fn test_real_collection_multiple_invalid() {
    let tokens = to_tokens(&["bad", "1.0", "wrong"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Real).unwrap_err();

    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|(ti, _)| ti.token == "bad"));
    assert!(errors.iter().any(|(ti, _)| ti.token == "wrong"));
}

#[test]
fn test_real_collection_all_invalid() {
    let tokens = to_tokens(&["a", "b"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Real).unwrap_err();

    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|(ti, _)| ti.token == "a"));
    assert!(errors.iter().any(|(ti, _)| ti.token == "b"));
}

// =========================================================
// DicoType::Logical
// =========================================================

#[test]
fn test_logical_collection_true_false() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["true", "false"]), &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![true, false])
    );
}

#[test]
fn test_logical_collection_french_variants() {
    assert_eq!(
        parse_collection_values_2(
            &to_tokens(&["vrai", "faux", "oui", "non"]),
            &DicoType::Logical
        )
        .unwrap(),
        ConfigValue::BooleanCollection(vec![true, false, true, false])
    );
}

#[test]
fn test_logical_collection_mixed_variants() {
    assert_eq!(
        parse_collection_values_2(
            &to_tokens(&["1", "0", "yes", "no", "on", "off"]),
            &DicoType::Logical
        )
        .unwrap(),
        ConfigValue::BooleanCollection(vec![true, false, true, false, true, false])
    );
}

#[test]
fn test_logical_collection_case_insensitive() {
    assert_eq!(
        parse_collection_values_2(
            &to_tokens(&["TRUE", "FALSE", "OUI", "NON"]),
            &DicoType::Logical
        )
        .unwrap(),
        ConfigValue::BooleanCollection(vec![true, false, true, false])
    );
}

#[test]
fn test_logical_collection_single_element() {
    assert_eq!(
        parse_collection_values_2(&to_tokens(&["yes"]), &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![true])
    );
}

#[test]
fn test_logical_collection_empty_input() {
    assert_eq!(
        parse_collection_values_2(&Vec::new(), &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![])
    );
}

#[test]
fn test_logical_collection_one_invalid() {
    let tokens = to_tokens(&["true", "maybe", "false"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Logical).unwrap_err();

    assert_eq!(errors.len(), 1);
    assert!(errors.iter().any(|(ti, _)| ti.token == "maybe"));
}

#[test]
fn test_logical_collection_multiple_invalid() {
    let tokens = to_tokens(&["bad", "true", "wrong"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Logical).unwrap_err();

    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|(ti, _)| ti.token == "bad"));
    assert!(errors.iter().any(|(ti, _)| ti.token == "wrong"));
}

#[test]
fn test_logical_collection_all_invalid() {
    let tokens = to_tokens(&["maybe", "perhaps"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Logical).unwrap_err();

    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|(ti, _)| ti.token == "maybe"));
    assert!(errors.iter().any(|(ti, _)| ti.token == "perhaps"));
}

// =========================================================
// Error message format
// =========================================================

#[test]
fn test_error_message_lists_invalid_integers() {
    let tokens = to_tokens(&["1", "bad", "wrong", "4"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Integer).unwrap_err();

    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|(ti, _)| ti.token == "bad"));
    assert!(errors.iter().any(|(ti, _)| ti.token == "wrong"));
}

#[test]
fn test_error_message_lists_all_invalid_floats() {
    let tokens = to_tokens(&["bad", "wrong"]);
    let errors = parse_collection_values_2(&tokens, &DicoType::Real).unwrap_err();
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|(ti, _)| ti.token == "bad"));
    assert!(errors.iter().any(|(ti, _)| ti.token == "wrong"));
}

// =========================================================
// into_scalars
// =========================================================

#[test]
fn test_into_scalars_string_collection() {
    let col = ConfigValue::StringCollection(strings(&["alice", "bob"]));
    let scalars = col.into_scalars().unwrap();
    assert_eq!(
        scalars,
        vec![
            ConfigValue::String("alice".into()),
            ConfigValue::String("bob".into()),
        ]
    );
}

#[test]
fn test_into_scalars_path_collection() {
    let col = ConfigValue::PathCollection(paths(&["/usr/bin", "/usr/local/bin"]));
    let scalars = col.into_scalars().unwrap();
    assert_eq!(
        scalars,
        vec![
            ConfigValue::Path(PathBuf::from("/usr/bin")),
            ConfigValue::Path(PathBuf::from("/usr/local/bin")),
        ]
    );
}

#[test]
fn test_into_scalars_boolean_collection() {
    let col = ConfigValue::BooleanCollection(vec![true, false, true]);
    let scalars = col.into_scalars().unwrap();
    assert_eq!(
        scalars,
        vec![
            ConfigValue::Boolean(true),
            ConfigValue::Boolean(false),
            ConfigValue::Boolean(true),
        ]
    );
}

#[test]
fn test_into_scalars_integer_collection() {
    let col = ConfigValue::IntegerCollection(vec![1, 2, 3]);
    let scalars = col.into_scalars().unwrap();
    assert_eq!(
        scalars,
        vec![
            ConfigValue::Integer(1),
            ConfigValue::Integer(2),
            ConfigValue::Integer(3),
        ]
    );
}

#[test]
fn test_into_scalars_float_collection() {
    let col = ConfigValue::FloatCollection(vec![1.1, 2.2, 3.3]);
    let scalars = col.into_scalars().unwrap();
    assert_eq!(
        scalars,
        vec![
            ConfigValue::Float(1.1),
            ConfigValue::Float(2.2),
            ConfigValue::Float(3.3),
        ]
    );
}

#[test]
fn test_into_scalars_single_element() {
    let col = ConfigValue::IntegerCollection(vec![42]);
    let scalars = col.into_scalars().unwrap();
    assert_eq!(scalars, vec![ConfigValue::Integer(42)]);
}

#[test]
fn test_into_scalars_empty_collection() {
    let col = ConfigValue::StringCollection(vec![]);
    let scalars = col.into_scalars().unwrap();
    assert!(scalars.is_empty());
}

#[test]
fn test_into_scalars_on_scalar_is_error() {
    for scalar in [
        ConfigValue::String("x".into()),
        ConfigValue::Path(PathBuf::from("/x")),
        ConfigValue::Boolean(true),
        ConfigValue::Integer(1),
        ConfigValue::Float(1.0),
    ] {
        assert!(
            scalar.into_scalars().is_err(),
            "expected error for scalar variant"
        );
    }
}

// =========================================================
// collect
// =========================================================

#[test]
fn test_collect_strings() {
    let values = vec![
        ConfigValue::String("alice".into()),
        ConfigValue::String("bob".into()),
    ];
    assert_eq!(
        ConfigValue::collect(values).unwrap(),
        ConfigValue::StringCollection(strings(&["alice", "bob"]))
    );
}

#[test]
fn test_collect_paths() {
    let values = vec![
        ConfigValue::Path(PathBuf::from("/usr/bin")),
        ConfigValue::Path(PathBuf::from("/usr/local/bin")),
    ];
    assert_eq!(
        ConfigValue::collect(values).unwrap(),
        ConfigValue::PathCollection(paths(&["/usr/bin", "/usr/local/bin"]))
    );
}

#[test]
fn test_collect_booleans() {
    let values = vec![ConfigValue::Boolean(true), ConfigValue::Boolean(false)];
    assert_eq!(
        ConfigValue::collect(values).unwrap(),
        ConfigValue::BooleanCollection(vec![true, false])
    );
}

#[test]
fn test_collect_integers() {
    let values = vec![
        ConfigValue::Integer(10),
        ConfigValue::Integer(20),
        ConfigValue::Integer(30),
    ];
    assert_eq!(
        ConfigValue::collect(values).unwrap(),
        ConfigValue::IntegerCollection(vec![10, 20, 30])
    );
}

#[test]
fn test_collect_floats() {
    let values = vec![ConfigValue::Float(1.1), ConfigValue::Float(2.2)];
    assert_eq!(
        ConfigValue::collect(values).unwrap(),
        ConfigValue::FloatCollection(vec![1.1, 2.2])
    );
}

#[test]
fn test_collect_single_element() {
    let values = vec![ConfigValue::Integer(99)];
    assert_eq!(
        ConfigValue::collect(values).unwrap(),
        ConfigValue::IntegerCollection(vec![99])
    );
}

#[test]
fn test_collect_empty_is_error() {
    assert!(ConfigValue::collect(vec![]).is_err());
}

#[test]
fn test_collect_mixed_types_is_error() {
    let values = vec![
        ConfigValue::Integer(1),
        ConfigValue::String("oops".into()),
        ConfigValue::Integer(3),
    ];
    let err = ConfigValue::collect(values).unwrap_err();
    assert!(
        err.contains("element 1"),
        "error should identify the offending index"
    );
}

#[test]
fn test_collect_collection_variant_is_error() {
    // A Vec containing collection variants should be rejected
    let values = vec![ConfigValue::IntegerCollection(vec![1, 2])];
    assert!(ConfigValue::collect(values).is_err());
}

// =========================================================
// Round-trip: collect . into_scalars == identity
// =========================================================

#[test]
fn test_roundtrip_into_scalars_then_collect_string() {
    let original = ConfigValue::StringCollection(strings(&["x", "y", "z"]));
    let roundtripped = ConfigValue::collect(original.clone().into_scalars().unwrap()).unwrap();
    assert_eq!(original, roundtripped);
}

#[test]
fn test_roundtrip_into_scalars_then_collect_integer() {
    let original = ConfigValue::IntegerCollection(vec![1, 2, 3]);
    let roundtripped = ConfigValue::collect(original.clone().into_scalars().unwrap()).unwrap();
    assert_eq!(original, roundtripped);
}

#[test]
fn test_roundtrip_collect_then_into_scalars_float() {
    let scalars = vec![ConfigValue::Float(1.0), ConfigValue::Float(2.0)];
    let roundtripped = ConfigValue::collect(scalars.clone())
        .unwrap()
        .into_scalars()
        .unwrap();
    assert_eq!(scalars, roundtripped);
}

// Ignore french word used in telemac
// cSpell:ignore vrai faux oui non
