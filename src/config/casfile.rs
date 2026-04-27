//! # Telemac ".cas" file - use case configuration file
//!
//! This module allows to parse telemac ".cas" file.
//! These files are also known as steering files.
//!

use std::collections::HashMap;

use super::configvalue;
use super::configvalue::ConfigValue;
use super::dicofile;
use super::textloc::{TextLoc, UNKNOWN_FILE};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Can't open file {filename} for reading: {error}")]
    FileOpenFailed {
        filename: String,
        error: std::io::Error,
    },
    #[error("{pos}: Unknown key: '{key}'")]
    UnknownKey { pos: TextLoc, key: String },
    #[error("{pos}: Invalid value for key '{key}': {reason}")]
    InvalidValue {
        pos: TextLoc,
        key: String,
        reason: String,
    },
    #[error(
        "{pos}: Too much values for key '{key}': got {got_count} but expected {expected_count}"
    )]
    TooMuchValues {
        pos: TextLoc,
        key: String,
        got_count: usize,
        expected_count: usize,
    },
    #[error("{pos}: Syntax error: {reason}")]
    SyntaxError { pos: TextLoc, reason: String },
    #[error(
        "{pos}: Value out of bound for key {key}: value should be between {min} and {max}, got '{value}'"
    )]
    OutOfBound {
        pos: TextLoc,
        key: String,
        value: String,
        min: f64,
        max: f64,
    },
    #[error("{pos}: Invalid value for key {key}: {reason}")]
    BadChoice {
        pos: TextLoc,
        key: String,
        value: ConfigValue,
        #[source]
        reason: dicofile::ChoiceValidationError,
    },
}

type ErrorPtr = Box<dyn std::error::Error>;
type VecErrorPtr = Vec<ErrorPtr>;

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

    /// Read a CAS file and parse it
    #[allow(dead_code)]
    pub fn parse_from_file(
        &self,
        filename: &String,
    ) -> Result<HashMap<String, ConfigValue>, VecErrorPtr> {
        match std::fs::read_to_string(filename) {
            Ok(cascontent) => {
                self.parse_from_content_and_filename(cascontent.as_str(), filename.as_str())
            }
            Err(error) => {
                let errors: VecErrorPtr = vec![Box::new(ParseError::FileOpenFailed {
                    filename: filename.clone(),
                    error,
                })];
                Err(errors)
            }
        }
    }

    /// Parse a buffer containing the input of a CAS file
    #[allow(dead_code)]
    pub fn parse(&self, input: &str) -> Result<HashMap<String, ConfigValue>, VecErrorPtr> {
        self.parse_from_content_and_filename(input, UNKNOWN_FILE)
    }

    fn parse_from_content_and_filename(
        &self,
        input: &str,
        filename: &str,
    ) -> Result<HashMap<String, ConfigValue>, VecErrorPtr> {
        let mut result = HashMap::new();
        let mut errors: VecErrorPtr = Vec::new();

        for (line_num, line) in input.lines().enumerate() {
            let line_num = line_num + 1;
            let pos = TextLoc::from((filename, line_num));

            // Strip inline comments and trim
            let line = strip_comment(line).trim();

            if line.is_empty() {
                continue;
            }

            // Split on first '=' or ':'
            let Some(eq_pos) = line.find(['=', ':']) else {
                errors.push(Box::new(ParseError::SyntaxError {
                    pos,
                    reason: "Missing assignment operator ('=' or ':') ".into(),
                }));
                continue;
            };
            let raw_key = String::from(line[..eq_pos].trim());
            let raw_value = line[eq_pos + 1..].trim();

            let Some(keyword) = self.keywords.get(&raw_key.to_uppercase()) else {
                errors.push(Box::new(ParseError::UnknownKey { pos, key: raw_key }));
                continue;
            };

            let nargs: usize = keyword.nargs.try_into().unwrap_or(1);

            let parse_result = configvalue::parse_value(raw_value, &keyword.type_, nargs);

            let Ok(value) = parse_result else {
                errors.push(Box::new(ParseError::InvalidValue {
                    pos,
                    key: raw_key,
                    reason: parse_result.err().unwrap(),
                }));
                continue;
            };

            if (nargs != 0) && (value.len() > nargs) {
                errors.push(Box::new(ParseError::TooMuchValues {
                    pos,
                    key: raw_key,
                    got_count: value.len(),
                    expected_count: nargs,
                }));
                continue;
            }

            if let Some(boundaries) = keyword.boundaries {
                let failures = get_out_of_bounds(&value, boundaries);
                let nb_of_failures = failures.len();
                for _failed_index in failures {
                    errors.push(Box::new(ParseError::OutOfBound {
                        pos: pos.clone(),
                        key: raw_key.clone(),
                        value: String::from(raw_value),
                        min: boundaries.0,
                        max: boundaries.1,
                    }));
                }
                if nb_of_failures > 0 {
                    continue;
                }
            };

            let normalized_value = match keyword.normalize_choice(&value) {
                Ok(new_value) => new_value,
                Err(reasons) => {
                    for reason in reasons {
                        errors.push(Box::new(ParseError::BadChoice {
                            key: raw_key.clone(),
                            pos: pos.clone(),
                            value: value.clone(),
                            reason,
                        }));
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

    pub fn config_from(&self, input: &str) -> Result<HashMap<String, ConfigValue>, VecErrorPtr> {
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

/// Check that values are in the min/max interval
/// Return indexes of failures (empty vec means all ok)
fn get_out_of_bounds(value: &ConfigValue, boundaries: (f64, f64)) -> Vec<usize> {
    match value {
        ConfigValue::Integer(v) => {
            let min: i64 = boundaries.0 as i64;
            let max: i64 = boundaries.1 as i64;
            if min <= *v && *v <= max {
                Vec::new()
            } else {
                vec![0]
            }
        }
        ConfigValue::Float(v) => {
            let min: f64 = boundaries.0;
            let max: f64 = boundaries.1;
            if min <= *v && *v <= max {
                Vec::new()
            } else {
                vec![0]
            }
        }
        ConfigValue::IntegerCollection(values) => {
            let min: i64 = boundaries.0 as i64;
            let max: i64 = boundaries.1 as i64;
            let mut failures: Vec<usize> = Vec::new();
            for (i, v) in values.iter().enumerate() {
                if !(min <= *v && *v <= max) {
                    failures.push(i);
                }
            }
            failures
        }
        ConfigValue::FloatCollection(values) => {
            let min: f64 = boundaries.0;
            let max: f64 = boundaries.1;
            let mut failures: Vec<usize> = Vec::new();
            for (i, v) in values.iter().enumerate() {
                if !(min <= *v && *v <= max) {
                    failures.push(i);
                }
            }
            failures
        }
        // All other case: failure: can't check boundaries
        _ => vec![0],
    }
}

#[cfg(test)]
mod tests;
