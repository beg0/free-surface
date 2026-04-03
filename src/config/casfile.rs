//! # Telemac ".cas" file - use case configuration file
//!
//! This module allows to parse telemac ".cas" file.
//! These files are also known as steering files.
//!

use std::collections::HashMap;

use super::configvalue;
use super::configvalue::ConfigValue;
use super::dicofile;
use super::parse_helpers::{DamoclesParser, KeywordParseInfo, TokenInfo};
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
        value: String,
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

struct ParserInternal<'dico> {
    keywords: &'dico HashMap<&'dico String, &'dico dicofile::DicoKeyword>,
    top_pos: TextLoc,
    result: HashMap<String, ConfigValue>,
    errors: VecErrorPtr,
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
        // trash previous results
        let mut internal = ParserInternal {
            keywords: &self.keywords,
            result: HashMap::new(),
            errors: Vec::new(),
            top_pos: TextLoc::from((filename, 1)),
        };
        internal.parse_fields(input);

        if internal.errors.is_empty() {
            Ok(internal.result)
        } else {
            Err(internal.errors)
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

    #[allow(dead_code)]
    pub fn config_from(&self, input: &str) -> Result<HashMap<String, ConfigValue>, VecErrorPtr> {
        let mut config = self.parse(input)?;

        self.fill_missing_fields(&mut config);
        Ok(config)
    }
}

impl<'dico> DamoclesParser for ParserInternal<'dico> {
    fn error(&mut self, e: ErrorPtr) {
        self.errors.push(e);
    }

    fn cmd(&mut self, _cmd: &TokenInfo) {
        //TODO
    }

    fn loc(&self, pos: (usize, usize)) -> TextLoc {
        self.top_pos.clone_with_line_col(pos.0, pos.1)
    }

    fn new_field(&mut self, mut kpi: KeywordParseInfo) {
        let Some(keyword) = self.keywords.get(&kpi.keyname().to_uppercase()) else {
            self.error(Box::new(ParseError::UnknownKey {
                pos: kpi.key.start_pos.clone(),
                key: kpi.key.token.clone(),
            }));
            return;
        };

        let nargs: usize = keyword.nargs.try_into().unwrap_or(1);
        kpi.fix_list(&keyword.type_, nargs);
        let value_parse_infos = &mut kpi.values;

        let parse_result = configvalue::parse_value_2::<ErrorPtr, _>(
            value_parse_infos,
            &keyword.type_,
            nargs,
            |entry, reason| {
                Box::new(ParseError::InvalidValue {
                    pos: entry.start_pos.clone(),
                    key: kpi.key.token.clone(),
                    reason,
                })
            },
        );

        let value = match parse_result {
            Ok(v) => v,
            Err(errs) => {
                for e in errs {
                    self.error(e);
                }
                return;
            }
        };

        if let Some(boundaries) = keyword.boundaries {
            let failures = get_out_of_bounds(&value, boundaries);
            let nb_of_failures = failures.len();
            for failed_index in failures {
                if let Some(failed_value) = value_parse_infos.get(failed_index) {
                    self.error(Box::new(ParseError::OutOfBound {
                        pos: failed_value.start_pos.clone(),
                        key: kpi.key.token.clone(),
                        value: failed_value.token.clone(),
                        min: boundaries.0,
                        max: boundaries.1,
                    }));
                }
            }
            if nb_of_failures > 0 {
                return;
            }
        }

        let normalized_value: ConfigValue = match keyword.normalize_choice(&value) {
            Ok(new_value) => new_value,
            Err(failures) => {
                for (failed_index, reason) in failures {
                    if let Some(failed_value) = value_parse_infos.get(failed_index) {
                        self.error(Box::new(ParseError::BadChoice {
                            key: kpi.key.token.clone(),
                            pos: failed_value.start_pos.clone(),
                            value: failed_value.token.clone(),
                            reason,
                        }));
                    }
                }
                return;
            }
        };
        self.result.insert(keyword.name().clone(), normalized_value);
    }
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
