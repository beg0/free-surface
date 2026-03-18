//! # Telemac ".cas" file - use case configuration file
//!
//! This module allows to parse telemac ".cas" file
//!

use std::collections::HashMap;

use super::configvalue::{parse_value, ConfigValue};
use super::dicofile;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unknown key at line {line}: '{key}'")]
    UnknownKey { key: String, line: usize },
    #[error("Invalid value for key '{key}' at line {line}: {reason}")]
    InvalidValue {
        key: String,
        line: usize,
        reason: String,
    },
    #[error("Too much values for key '{key}' at line {line}: got {got_count} but expected {expected_count}")]
    TooMuchValues {
        key: String,
        line: usize,
        got_count: usize,
        expected_count: usize,
    },
    #[error("Syntax error on line {line}: {reason}")]
    SyntaxError { line: usize, reason: String },
    #[error(
        "Value out of bound for key {key} at line {line}: value should be between {min} and {max}, got '{value}'"
    )]
    OutOfBound {
        key: String,
        line: usize,
        value: String,
        min: f64,
        max: f64,
    },
    #[error("Invalid value for key {key} at line {line}: {reason}")]
    BadChoice {
        key: String,
        line: usize,
        value: ConfigValue,
        #[source]
        reason: dicofile::ChoiceValidationError,
    },
}

pub struct Parser<'dico> {
    /// Map of normalized (uppercase) key -> expected type
    dico: &'dico dicofile::Dico,
    keywords: HashMap<&'dico String, &'dico dicofile::DicoKeyword>,
}

impl<'dico> Parser<'dico> {
    pub fn new(dico: &'dico dicofile::Dico) -> Self {
        let mut ret = Self {
            dico,
            keywords: HashMap::new(),
        };
        for keyword in ret.dico {
            for desc in keyword.text_desc.values() {
                ret.keywords.insert(&desc.name, keyword);
            }
        }
        ret
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
            let Some(eq_pos) = line.find(['=', ':']) else {
                errors.push(ParseError::SyntaxError {
                    line: line_num,
                    reason: "Missing assignment operator ('=' or ':') ".into(),
                });
                continue;
            };
            let raw_key = String::from(line[..eq_pos].trim());
            let raw_value = line[eq_pos + 1..].trim();

            let Some(keyword) = self.keywords.get(&raw_key.to_uppercase()) else {
                errors.push(ParseError::UnknownKey {
                    line: line_num,
                    key: raw_key,
                });
                continue;
            };

            let nargs: usize = keyword.nargs.try_into().unwrap_or(1);

            let parse_result = parse_value(raw_value, &keyword.type_, nargs);

            let Ok(value) = parse_result else {
                errors.push(ParseError::InvalidValue {
                    line: line_num,
                    key: raw_key,
                    reason: parse_result.err().unwrap(),
                });
                continue;
            };

            if (nargs != 0) && (value.len() > nargs) {
                errors.push(ParseError::TooMuchValues {
                    line: line_num,
                    key: raw_key,
                    got_count: value.len(),
                    expected_count: nargs,
                });
                continue;
            }

            if let Some(boundaries) = keyword.boundaries
                && !check_boundaries(&value, boundaries)
            {
                errors.push(ParseError::OutOfBound {
                    line: line_num,
                    key: raw_key,
                    value: String::from(raw_value),
                    min: boundaries.0,
                    max: boundaries.1,
                });
                continue;
            }

            let normalized_value = match keyword.normalize_choice(&value) {
                Ok(new_value) => new_value,
                Err(reasons) => {
                    for reason in reasons {
                        errors.push(ParseError::BadChoice {
                            key: raw_key.clone(),
                            line: line_num,
                            value: value.clone(),
                            reason,
                        });
                    }
                    continue;
                }
            };
            result.insert(keyword.name().clone(), normalized_value);
        }

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors)
        }
    }

    pub fn fill_missing_fields(&self, config: &mut HashMap<String, ConfigValue>) {
        for keyword in self.dico {
            let keyword_name = keyword.name();
            if (keyword.level == 0) && !config.contains_key(keyword_name) {
                config.insert(keyword_name.clone(), keyword.default());
            }
        }
    }

    pub fn config_from(
        &self,
        input: &str,
    ) -> Result<HashMap<String, ConfigValue>, Vec<ParseError>> {
        let mut config = self.parse(input)?;

        self.fill_missing_fields(&mut config);
        Ok(config)
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut chars = line.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            '\'' if !in_double_quotes => {
                // Handle escaped single quote '' - peek at next char
                if in_single_quotes {
                    if chars.peek().map(|(_, c)| *c) == Some('\'') {
                        chars.next(); // consume the second ', it's an escape
                    } else {
                        in_single_quotes = false;
                    }
                } else {
                    in_single_quotes = true;
                }
            }
            '/' | '#' if !in_double_quotes && !in_single_quotes => return &line[..i],
            _ => {}
        }
    }
    line
}

fn check_boundaries(value: &ConfigValue, boundaries: (f64, f64)) -> bool {
    match value {
        ConfigValue::Integer(v) => {
            let min: i64 = boundaries.0 as i64;
            let max: i64 = boundaries.1 as i64;
            min <= *v && *v <= max
        }
        ConfigValue::Float(v) => {
            let min: f64 = boundaries.0;
            let max: f64 = boundaries.1;
            min <= *v && *v <= max
        }
        ConfigValue::IntegerCollection(values) => {
            let min: i64 = boundaries.0 as i64;
            let max: i64 = boundaries.1 as i64;
            let mut result: bool = true;
            for v in values {
                if !(min <= *v && *v <= max) {
                    result = false;
                }
            }
            result
        }
        ConfigValue::FloatCollection(values) => {
            let min: f64 = boundaries.0;
            let max: f64 = boundaries.1;
            let mut result: bool = true;
            for v in values {
                if !(min <= *v && *v <= max) {
                    result = false;
                }
            }
            result
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
