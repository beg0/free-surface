use super::*;
use std::path::PathBuf;

// --- Helpers ---

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn paths(v: &[&str]) -> Vec<PathBuf> {
    v.iter().map(PathBuf::from).collect()
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
        parse_single_value("hello", &DicoType::String).unwrap(),
        ConfigValue::String("hello".into())
    );
}

#[test]
fn test_string_quoted() {
    assert_eq!(
        parse_single_value("'hello world'", &DicoType::String).unwrap(),
        ConfigValue::String("hello world".into())
    );
}

#[test]
fn test_string_empty() {
    assert_eq!(
        parse_single_value("", &DicoType::String).unwrap(),
        ConfigValue::String("".into())
    );
}

#[test]
fn test_string_empty_quoted() {
    assert_eq!(
        parse_single_value("''", &DicoType::String).unwrap(),
        ConfigValue::String("".into())
    );
}

#[test]
fn test_string_escaped_single_quote() {
    // '' inside quotes should be unescaped to '
    assert_eq!(
        parse_single_value("'it''s fine'", &DicoType::String).unwrap(),
        ConfigValue::String("it's fine".into())
    );
}

// DicoType::Integer
// -----------------

#[test]
fn test_integer_positive() {
    assert_eq!(
        parse_single_value("42", &DicoType::Integer).unwrap(),
        ConfigValue::Integer(42)
    );
}

#[test]
fn test_integer_negative() {
    assert_eq!(
        parse_single_value("-7", &DicoType::Integer).unwrap(),
        ConfigValue::Integer(-7)
    );
}

#[test]
fn test_integer_zero() {
    assert_eq!(
        parse_single_value("0", &DicoType::Integer).unwrap(),
        ConfigValue::Integer(0)
    );
}

#[test]
fn test_integer_max() {
    let raw = i64::MAX.to_string();
    assert_eq!(
        parse_single_value(&raw, &DicoType::Integer).unwrap(),
        ConfigValue::Integer(i64::MAX)
    );
}

#[test]
fn test_integer_invalid_float() {
    assert!(parse_single_value("3.14", &DicoType::Integer).is_err());
}

#[test]
fn test_integer_invalid_word() {
    let err = parse_single_value("notanumber", &DicoType::Integer).unwrap_err();
    assert!(err.contains("notanumber"));
}

#[test]
fn test_integer_invalid_empty() {
    assert!(parse_single_value("", &DicoType::Integer).is_err());
}

// DicoType::Real
// --------------

#[test]
fn test_real_decimal() {
    assert_eq!(
        parse_single_value("3.14", &DicoType::Real).unwrap(),
        ConfigValue::Float(3.14)
    );
}

#[test]
fn test_real_whole_number() {
    assert_eq!(
        parse_single_value("42", &DicoType::Real).unwrap(),
        ConfigValue::Float(42.0)
    );
}

#[test]
fn test_real_negative() {
    assert_eq!(
        parse_single_value("-2.718", &DicoType::Real).unwrap(),
        ConfigValue::Float(-2.718)
    );
}

#[test]
fn test_real_scientific_notation() {
    assert_eq!(
        parse_single_value("1.5e3", &DicoType::Real).unwrap(),
        ConfigValue::Float(1500.0)
    );
}

#[test]
fn test_real_invalid_word() {
    let err = parse_single_value("notafloat", &DicoType::Real).unwrap_err();
    assert!(err.contains("notafloat"));
}

#[test]
fn test_real_invalid_empty() {
    assert!(parse_single_value("", &DicoType::Real).is_err());
}

// DicoType::Logical
// -----------------

#[test]
fn test_logical_true_variants() {
    for val in &["true", "yes", "1", "on", "vrai", "oui", "TRUE", "OUI"] {
        assert_eq!(
            parse_single_value(val, &DicoType::Logical).unwrap(),
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
            parse_single_value(val, &DicoType::Logical).unwrap(),
            ConfigValue::Boolean(false),
            "expected false for '{}'",
            val
        );
    }
}

#[test]
fn test_logical_invalid() {
    let err = parse_single_value("maybe", &DicoType::Logical).unwrap_err();
    assert!(err.contains("maybe"));
}

// Return type sanity - each DicoType maps to the right variant
// -------------------------------------------------------------

#[test]
fn test_string_returns_string_variant() {
    assert!(matches!(
        parse_single_value("x", &DicoType::String).unwrap(),
        ConfigValue::String(_)
    ));
}

#[test]
fn test_integer_returns_integer_variant() {
    assert!(matches!(
        parse_single_value("1", &DicoType::Integer).unwrap(),
        ConfigValue::Integer(_)
    ));
}

#[test]
fn test_real_returns_float_variant() {
    assert!(matches!(
        parse_single_value("1.0", &DicoType::Real).unwrap(),
        ConfigValue::Float(_)
    ));
}

#[test]
fn test_logical_returns_boolean_variant() {
    assert!(matches!(
        parse_single_value("true", &DicoType::Logical).unwrap(),
        ConfigValue::Boolean(_)
    ));
}

// =========================================================
// DicoType::String
// =========================================================

#[test]
fn test_string_collection_unquoted() {
    assert_eq!(
        parse_collection_values(vec!["alice", "bob", "charlie"], &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["alice".into(), "bob".into(), "charlie".into()])
    );
}

#[test]
fn test_string_collection_quoted() {
    assert_eq!(
        parse_collection_values(vec!["'alice'", "'bob'"], &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["alice".into(), "bob".into()])
    );
}

#[test]
fn test_string_collection_mixed_quoted_unquoted() {
    assert_eq!(
        parse_collection_values(vec!["'alice'", "bob"], &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["alice".into(), "bob".into()])
    );
}

#[test]
fn test_string_collection_with_escaped_quote() {
    assert_eq!(
        parse_collection_values(vec!["'it''s'", "'l''eau'"], &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["it's".into(), "l'eau".into()])
    );
}

#[test]
fn test_string_collection_empty_strings() {
    assert_eq!(
        parse_collection_values(vec!["''", "''"], &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["".into(), "".into()])
    );
}

#[test]
fn test_string_collection_single_element() {
    assert_eq!(
        parse_collection_values(vec!["hello"], &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec!["hello".into()])
    );
}

#[test]
fn test_string_collection_empty_input() {
    assert_eq!(
        parse_collection_values(vec![], &DicoType::String).unwrap(),
        ConfigValue::StringCollection(vec![])
    );
}

// =========================================================
// DicoType::Integer
// =========================================================

#[test]
fn test_integer_collection() {
    assert_eq!(
        parse_collection_values(vec!["1", "2", "3"], &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![1, 2, 3])
    );
}

#[test]
fn test_integer_collection_negative() {
    assert_eq!(
        parse_collection_values(vec!["-1", "0", "42"], &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![-1, 0, 42])
    );
}

#[test]
fn test_integer_collection_single_element() {
    assert_eq!(
        parse_collection_values(vec!["99"], &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![99])
    );
}

#[test]
fn test_integer_collection_empty_input() {
    assert_eq!(
        parse_collection_values(vec![], &DicoType::Integer).unwrap(),
        ConfigValue::IntegerCollection(vec![])
    );
}

#[test]
fn test_integer_collection_one_invalid() {
    let err = parse_collection_values(vec!["1", "oops", "3"], &DicoType::Integer).unwrap_err();
    assert!(err.contains("oops"));
}

#[test]
fn test_integer_collection_multiple_invalid() {
    let err = parse_collection_values(vec!["bad", "1", "wrong"], &DicoType::Integer).unwrap_err();
    assert!(err.contains("bad"));
    assert!(err.contains("wrong"));
}

#[test]
fn test_integer_collection_all_invalid() {
    let err = parse_collection_values(vec!["a", "b", "c"], &DicoType::Integer).unwrap_err();
    assert!(err.contains("a"));
    assert!(err.contains("b"));
    assert!(err.contains("c"));
}

#[test]
fn test_integer_collection_float_is_invalid() {
    let err = parse_collection_values(vec!["1", "3.14"], &DicoType::Integer).unwrap_err();
    assert!(err.contains("3.14"));
}

// =========================================================
// DicoType::Real
// =========================================================

#[test]
fn test_real_collection() {
    assert_eq!(
        parse_collection_values(vec!["1.1", "2.2", "3.3"], &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![1.1, 2.2, 3.3])
    );
}

#[test]
fn test_real_collection_whole_numbers() {
    assert_eq!(
        parse_collection_values(vec!["1", "2", "3"], &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![1.0, 2.0, 3.0])
    );
}

#[test]
fn test_real_collection_negative() {
    assert_eq!(
        parse_collection_values(vec!["-1.5", "0.0", "2.5"], &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![-1.5, 0.0, 2.5])
    );
}

#[test]
fn test_real_collection_scientific_notation() {
    assert_eq!(
        parse_collection_values(vec!["1.5e3", "2.0e-2"], &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![1500.0, 0.02])
    );
}

#[test]
fn test_real_collection_single_element() {
    assert_eq!(
        parse_collection_values(vec!["3.14"], &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![3.14])
    );
}

#[test]
fn test_real_collection_empty_input() {
    assert_eq!(
        parse_collection_values(vec![], &DicoType::Real).unwrap(),
        ConfigValue::FloatCollection(vec![])
    );
}

#[test]
fn test_real_collection_one_invalid() {
    let err =
        parse_collection_values(vec!["1.0", "notafloat", "3.0"], &DicoType::Real).unwrap_err();
    assert!(err.contains("notafloat"));
}

#[test]
fn test_real_collection_multiple_invalid() {
    let err = parse_collection_values(vec!["bad", "1.0", "wrong"], &DicoType::Real).unwrap_err();
    assert!(err.contains("bad"));
    assert!(err.contains("wrong"));
}

#[test]
fn test_real_collection_all_invalid() {
    let err = parse_collection_values(vec!["a", "b"], &DicoType::Real).unwrap_err();
    assert!(err.contains("a"));
    assert!(err.contains("b"));
}

// =========================================================
// DicoType::Logical
// =========================================================

#[test]
fn test_logical_collection_true_false() {
    assert_eq!(
        parse_collection_values(vec!["true", "false"], &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![true, false])
    );
}

#[test]
fn test_logical_collection_french_variants() {
    assert_eq!(
        parse_collection_values(vec!["vrai", "faux", "oui", "non"], &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![true, false, true, false])
    );
}

#[test]
fn test_logical_collection_mixed_variants() {
    assert_eq!(
        parse_collection_values(vec!["1", "0", "yes", "no", "on", "off"], &DicoType::Logical)
            .unwrap(),
        ConfigValue::BooleanCollection(vec![true, false, true, false, true, false])
    );
}

#[test]
fn test_logical_collection_case_insensitive() {
    assert_eq!(
        parse_collection_values(vec!["TRUE", "FALSE", "OUI", "NON"], &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![true, false, true, false])
    );
}

#[test]
fn test_logical_collection_single_element() {
    assert_eq!(
        parse_collection_values(vec!["yes"], &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![true])
    );
}

#[test]
fn test_logical_collection_empty_input() {
    assert_eq!(
        parse_collection_values(vec![], &DicoType::Logical).unwrap(),
        ConfigValue::BooleanCollection(vec![])
    );
}

#[test]
fn test_logical_collection_one_invalid() {
    let err =
        parse_collection_values(vec!["true", "maybe", "false"], &DicoType::Logical).unwrap_err();
    assert!(err.contains("maybe"));
}

#[test]
fn test_logical_collection_multiple_invalid() {
    let err =
        parse_collection_values(vec!["bad", "true", "wrong"], &DicoType::Logical).unwrap_err();
    assert!(err.contains("bad"));
    assert!(err.contains("wrong"));
}

#[test]
fn test_logical_collection_all_invalid() {
    let err = parse_collection_values(vec!["maybe", "perhaps"], &DicoType::Logical).unwrap_err();
    assert!(err.contains("maybe"));
    assert!(err.contains("perhaps"));
}

// =========================================================
// Error message format
// =========================================================

#[test]
fn test_error_message_lists_all_invalid_integers() {
    let err =
        parse_collection_values(vec!["1", "bad", "wrong", "4"], &DicoType::Integer).unwrap_err();
    // Both invalid values should appear, joined by ", "
    assert!(err.contains("bad"));
    assert!(err.contains("wrong"));
    assert!(err.contains("are not valid integers"));
}

#[test]
fn test_error_message_lists_all_invalid_floats() {
    let err = parse_collection_values(vec!["bad", "wrong"], &DicoType::Real).unwrap_err();
    assert!(err.contains("are not valid floats"));
}

#[test]
fn test_error_message_lists_all_invalid_booleans() {
    let err = parse_collection_values(vec!["maybe", "perhaps"], &DicoType::Logical).unwrap_err();
    assert!(err.contains("are not valid booleans"));
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
