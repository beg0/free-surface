//! # Telemac ".cas" file - use case configuration file
//!
//! This module allows to parse telemac ".cas" file
//!

use std::collections::HashMap;
use std::str::FromStr;

use super::configvalue::ConfigValue;
use super::configvalue::DicoType;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unknown key: '{0}'")]
    UnknownKey(String),
    #[error("Invalid value for key '{key}': {reason}")]
    InvalidValue { key: String, reason: String },
    #[error("Syntax error on line {line}: {reason}")]
    SyntaxError { line: usize, reason: String },
}

pub struct Parser {
    /// Map of normalized (lowercase) key -> expected type
    dico: HashMap<String, DicoType>,
}

impl Parser {
    pub fn new(dico: impl IntoIterator<Item = (impl Into<String>, DicoType)>) -> Self {
        Self {
            dico: dico
                .into_iter()
                .map(|(k, v)| (k.into().to_lowercase(), v))
                .collect(),
        }
    }

    pub fn parse(&self, input: &str) -> Result<HashMap<String, ConfigValue>, Vec<ParseError>> {
        let mut result = HashMap::new();
        let mut errors = Vec::new();

        for (line_num, line) in input.lines().enumerate() {
            let line_num = line_num + 1;

            // Strip inline comments and trim
            let line = strip_comment(line).trim();

            if line.is_empty() {
                continue;
            }

            // Split on first '=' or ':'
            let Some(eq_pos) = line.find(&['=', ':']) else {
                errors.push(ParseError::SyntaxError {
                    line: line_num,
                    reason: "Missing assignment operator ('=' or ':') ".into(),
                });
                continue;
            };

            let raw_key = line[..eq_pos].trim().to_lowercase();
            let raw_value = line[eq_pos + 1..].trim();

            let Some(kind) = self.dico.get(&raw_key) else {
                errors.push(ParseError::UnknownKey(raw_key));
                continue;
            };

            match parse_value(raw_value, kind.clone()) {
                Ok(value) => {
                    result.insert(raw_key, value);
                }
                Err(reason) => {
                    errors.push(ParseError::InvalidValue {
                        key: raw_key,
                        reason,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors)
        }
    }
}

fn strip_comment(line: &str) -> &str {
    // Find first '/' or '#' that isn't inside a quoted string
    let mut in_quotes = false;
    let chars = line.char_indices().peekable();

    for (i, c) in chars {
        match c {
            '"' => in_quotes = !in_quotes,
            '/' | '#' if !in_quotes => return &line[..i],
            _ => {}
        }
    }
    line
}

fn parse_value(raw: &str, kind: DicoType) -> Result<ConfigValue, String> {
    match kind {
        DicoType::Logical => match raw.to_lowercase().as_str() {
            "true" | "yes" | "1" | "on" => Ok(ConfigValue::Boolean(true)),
            "false" | "no" | "0" | "off" => Ok(ConfigValue::Boolean(false)),
            _ => Err(format!("'{}' is not a valid boolean", raw)),
        },
        DicoType::Integer => i64::from_str(raw)
            .map(ConfigValue::Integer)
            .map_err(|_| format!("'{}' is not a valid integer", raw)),
        DicoType::Real => f64::from_str(raw)
            .map(ConfigValue::Float)
            .map_err(|_| format!("'{}' is not a valid float", raw)),
        // DicoType::Path => {
        //     let path = unquote(raw);
        //     Ok(ConfigValue::Path(std::path::PathBuf::from(path)))
        // }
        DicoType::String => Ok(ConfigValue::String(unquote(raw).to_string())),
    }
}

/// Remove surrounding quotes if present
fn unquote(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn make_parser() -> Parser {
        Parser::new([
            ("who mom loves", DicoType::String),
            ("my age", DicoType::Integer),
            ("am i serious", DicoType::Logical),
            ("my favorite number", DicoType::Real),
        ])
    }

    // --- Helpers ---

    fn parse_ok(input: &str) -> HashMap<String, ConfigValue> {
        make_parser()
            .parse(input)
            .expect("Expected successful parse")
    }

    fn parse_err(input: &str) -> Vec<ParseError> {
        make_parser()
            .parse(input)
            .expect_err("Expected parse errors")
    }

    // --- Happy path ---

    #[test]
    fn test_string_unquoted() {
        let config = parse_ok("who mom loves = me");
        assert_eq!(config["who mom loves"], ConfigValue::String("me".into()));
    }

    #[test]
    fn test_string_quoted() {
        let config = parse_ok(r#"who mom loves = "definitely me""#);
        assert_eq!(
            config["who mom loves"],
            ConfigValue::String("definitely me".into())
        );
    }

    #[test]
    fn test_integer() {
        let config = parse_ok("my age = 25");
        assert_eq!(config["my age"], ConfigValue::Integer(25));
    }

    #[test]
    fn test_integer_negative() {
        let config = parse_ok("my age = -5");
        assert_eq!(config["my age"], ConfigValue::Integer(-5));
    }

    #[test]
    fn test_float() {
        let config = parse_ok("my favorite number = 3.14");
        assert_eq!(config["my favorite number"], ConfigValue::Float(3.14));
    }

    #[test]
    fn test_float_whole_number() {
        let config = parse_ok("my favorite number = 42");
        assert_eq!(config["my favorite number"], ConfigValue::Float(42.0));
    }

    #[test]
    fn test_float_negative() {
        let config = parse_ok("my favorite number = -2.718");
        assert_eq!(config["my favorite number"], ConfigValue::Float(-2.718));
    }

    #[test]
    fn test_boolean_true_variants() {
        for val in &["true", "True", "TRUE", "yes", "Yes", "1", "on", "On"] {
            let input = format!("am i serious = {}", val);
            let config = parse_ok(&input);
            assert_eq!(
                config["am i serious"],
                ConfigValue::Boolean(true),
                "Failed for boolean input: {}",
                val
            );
        }
    }

    #[test]
    fn test_boolean_false_variants() {
        for val in &["false", "False", "FALSE", "no", "No", "0", "off", "Off"] {
            let input = format!("am i serious = {}", val);
            let config = parse_ok(&input);
            assert_eq!(
                config["am i serious"],
                ConfigValue::Boolean(false),
                "Failed for boolean input: {}",
                val
            );
        }
    }

    // #[test]
    // fn test_path_bare() {
    //     let config = parse_ok("where do i live = /my/house");
    //     assert_eq!(
    //         config["where do i live"],
    //         ConfigValue::Path(std::path::PathBuf::from("/my/house"))
    //     );
    // }

    // #[test]
    // fn test_path_quoted() {
    //     let config = parse_ok(r#"where do i live = "/my/cozy house""#);
    //     assert_eq!(
    //         config["where do i live"],
    //         ConfigValue::Path(std::path::PathBuf::from("/my/cozy house"))
    //     );
    // }

    // --- Case insensitivity ---

    #[test]
    fn test_key_case_insensitive_upper() {
        let config = parse_ok("WHO MOM LOVES = me");
        assert_eq!(config["who mom loves"], ConfigValue::String("me".into()));
    }

    #[test]
    fn test_key_mixed_case() {
        let config = parse_ok("Who Mom Loves = me");
        assert_eq!(config["who mom loves"], ConfigValue::String("me".into()));
    }

    // --- Comments ---

    #[test]
    fn test_hash_comment_line() {
        let config = parse_ok("# this is a comment\nmy age = 30");
        assert_eq!(config["my age"], ConfigValue::Integer(30));
    }

    #[test]
    fn test_slash_comment_line() {
        let config = parse_ok("/ this is a comment\nmy age = 30");
        assert_eq!(config["my age"], ConfigValue::Integer(30));
    }

    #[test]
    fn test_inline_hash_comment() {
        let config = parse_ok("my age = 30 # my real age");
        assert_eq!(config["my age"], ConfigValue::Integer(30));
    }

    #[test]
    fn test_inline_slash_comment() {
        let config = parse_ok("my age = 30 / my real age");
        assert_eq!(config["my age"], ConfigValue::Integer(30));
    }

    #[test]
    fn test_comment_inside_quoted_string_preserved() {
        let config = parse_ok(r#"who mom loves = "me / always""#);
        assert_eq!(
            config["who mom loves"],
            ConfigValue::String("me / always".into())
        );
    }

    #[test]
    fn test_hash_inside_quoted_string_preserved() {
        let config = parse_ok(r#"who mom loves = "me # always""#);
        assert_eq!(
            config["who mom loves"],
            ConfigValue::String("me # always".into())
        );
    }

    // --- Whitespace handling ---

    #[test]
    fn test_leading_trailing_whitespace_on_key() {
        let config = parse_ok("  who mom loves  = me");
        assert_eq!(config["who mom loves"], ConfigValue::String("me".into()));
    }

    #[test]
    fn test_leading_trailing_whitespace_on_value() {
        let config = parse_ok("my age =   25  ");
        assert_eq!(config["my age"], ConfigValue::Integer(25));
    }

    #[test]
    fn test_empty_lines_ignored() {
        let config = parse_ok("\n\nmy age = 25\n\n");
        assert_eq!(config["my age"], ConfigValue::Integer(25));
    }

    // --- Multiple keys ---

    #[test]
    fn test_multiple_keys() {
        let input = indoc::indoc! {"
            who mom loves = me
            my age = 25
            am i serious = false
            my favorite number = 3.14
        "};
        let config = parse_ok(input);
        assert_eq!(config["who mom loves"], ConfigValue::String("me".into()));
        assert_eq!(config["my age"], ConfigValue::Integer(25));
        assert_eq!(config["am i serious"], ConfigValue::Boolean(false));
        assert_eq!(config["my favorite number"], ConfigValue::Float(3.14));
    }

    #[test]
    fn test_last_value_wins_on_duplicate_key() {
        let config = parse_ok("my age = 20\nmy age = 99");
        assert_eq!(config["my age"], ConfigValue::Integer(99));
    }

    // --- Error cases ---

    #[test]
    fn test_unknown_key_produces_error() {
        let errors = parse_err("my cat = fluffy");
        assert!(errors
            .iter()
            .any(|e| matches!(e, ParseError::UnknownKey(k) if k == "my cat")));
    }

    #[test]
    fn test_invalid_integer() {
        let errors = parse_err("my age = olderthandirt");
        assert!(errors
            .iter()
            .any(|e| matches!(e, ParseError::InvalidValue { key, .. } if key == "my age")));
    }

    #[test]
    fn test_invalid_float() {
        let errors = parse_err("my favorite number = a lot");
        assert!(errors.iter().any(
            |e| matches!(e, ParseError::InvalidValue { key, .. } if key == "my favorite number")
        ));
    }

    #[test]
    fn test_invalid_boolean() {
        let errors = parse_err("am i serious = maybe");
        assert!(errors
            .iter()
            .any(|e| matches!(e, ParseError::InvalidValue { key, .. } if key == "am i serious")));
    }

    #[test]
    fn test_missing_equals_sign() {
        let errors = parse_err("my age 25");
        assert!(errors
            .iter()
            .any(|e| matches!(e, ParseError::SyntaxError { .. })));
    }

    #[test]
    fn test_multiple_errors_collected() {
        let input = indoc::indoc! {"
            my cat = fluffy
            my age = olderthandirt
            missing equals
        "};
        let errors = parse_err(input);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_valid_and_invalid_lines_mixed() {
        let input = "my age = 25\nmy cat = fluffy";
        let errors = make_parser().parse(input).expect_err("should have errors");
        assert_eq!(errors.len(), 1);
    }
}
