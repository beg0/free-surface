//! # Telemac ".cas" file - use case configuration file
//!
//! This module allows to parse telemac ".cas" file.
//! These files are also known as steering files.
//!

use std::collections::HashMap;
use std::path::Path;

use super::configvalue;
use super::configvalue::ConfigValue;
use super::dicofile;
use super::parse_helpers::{
    DamoclesCommandStatus, DamoclesError, DamoclesParser, KeywordParseInfo, TokenInfo,
};
use super::textloc::TextLoc;

use crate::aui::diagnostic::TextParserDiagnostics;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Can't open file {filename} for reading: {error}")]
    FileOpenFailed {
        filename: String,
        error: std::io::Error,
    },
    #[error("Unknown key: '{key}'")]
    UnknownKey { key: String },
    #[error("Invalid value for key '{key}': {reason}")]
    InvalidValue { key: String, reason: String },
    #[error("Too much values for key '{key}': got {got_count} but expected {expected_count}")]
    TooMuchValues {
        key: String,
        got_count: usize,
        expected_count: usize,
    },
    #[error(
        "Value out of bound for key {key}: value should be between {min} and {max}, got '{value}'"
    )]
    OutOfBound {
        key: String,
        value: String,
        min: f64,
        max: f64,
    },
    #[error("Invalid value for key {key}: {reason}")]
    BadChoice {
        key: String,
        value: String,
        #[source]
        reason: dicofile::ChoiceValidationError,
    },
}

pub struct Parser<'a> {
    /// Map of normalized (uppercase) key -> expected type
    dico: &'a dicofile::Dico,
}

struct ParserInternal<'a> {
    dico: &'a dicofile::Dico,
    top_pos: TextLoc,
    result: HashMap<String, ConfigValue>,
    diag: TextParserDiagnostics,
}

impl<'a> Parser<'a> {
    pub fn new(dico: &'a dicofile::Dico) -> Self {
        Self { dico }
    }

    /// Read a CAS file and parse it
    #[allow(dead_code)]
    pub fn parse_file<P: AsRef<Path>>(
        &self,
        filename: P,
    ) -> Result<HashMap<String, ConfigValue>, TextParserDiagnostics> {
        let file_pos = TextLoc::from((&filename, 0));

        let cascontent = std::fs::read_to_string(&filename).map_err(|err| {
            TextParserDiagnostics::from_single_error(err.to_string(), file_pos.clone())
        })?;
        self.parse_from_content_and_textloc(cascontent.as_str(), file_pos)
    }

    /// Parse a buffer containing the input of a CAS file
    #[allow(dead_code)]
    pub fn parse(
        &self,
        input: &str,
    ) -> Result<HashMap<String, ConfigValue>, TextParserDiagnostics> {
        self.parse_from_content_and_textloc(input, TextLoc::default())
    }

    fn parse_from_content_and_textloc(
        &self,
        input: &str,
        top_pos: TextLoc,
    ) -> Result<HashMap<String, ConfigValue>, TextParserDiagnostics> {
        // trash previous results
        let mut internal = ParserInternal {
            dico: self.dico,
            result: HashMap::new(),
            diag: TextParserDiagnostics::default(),
            top_pos,
        };
        internal.parse_fields(input);

        if internal.diag.has_errors(false) {
            Err(internal.diag)
        } else {
            Ok(internal.result)
        }
    }

    pub fn fill_missing_fields(&self, config: &mut HashMap<String, ConfigValue>) {
        for (keyword_name, keyword) in self.dico.iter() {
            if (keyword.level == 0) && !config.contains_key(keyword_name) {
                config.insert(keyword_name.clone(), keyword.default());
            }
        }
    }

    #[allow(dead_code)]
    pub fn config_from_content(
        &self,
        input: &str,
    ) -> Result<HashMap<String, ConfigValue>, TextParserDiagnostics> {
        let mut config = self.parse(input)?;

        self.fill_missing_fields(&mut config);
        Ok(config)
    }

    /// Load a config from a steering file
    ///
    /// Get the full config from a steering file.
    pub fn config_from_file<P: AsRef<Path>>(
        &self,
        filename: P,
    ) -> Result<HashMap<String, ConfigValue>, TextParserDiagnostics> {
        let mut config = self.parse_file(filename)?;

        self.fill_missing_fields(&mut config);
        Ok(config)
    }
}

impl<'a> DamoclesParser for ParserInternal<'a> {
    fn diag(&mut self) -> &mut TextParserDiagnostics {
        &mut self.diag
    }

    fn cmd(&mut self, cmd: TokenInfo) -> Option<DamoclesCommandStatus> {
        let mut exit_code = DamoclesCommandStatus::Success;

        // TODO: better processing of "ETA" & "IND" command.
        // For now, they are handled the same way

        match cmd.token[1..].to_ascii_uppercase().as_str() {
            "DYN" => {
                //Ignored in steering file
            }
            "LIS" => {
                // Dump the dico
                dbg!(&self.dico);
            }
            "ETA" => {
                // Dump the config file
                dbg!(&self.result);
            }
            "IND" => {
                // Dump values in config files
                dbg!(&self.result);
            }
            "STO" => {
                self.diag.error(
                    DamoclesError::StopCommand { cmd: cmd.token }.to_string(),
                    cmd.start_pos,
                );
                return None;
            }
            "FIN" => {
                exit_code = DamoclesCommandStatus::Exit;
            }
            "DOC" => {
                eprintln!("cmd DOC is deprecated");
            }
            _ => {
                self.diag.error(
                    DamoclesError::UnknownCommand { cmd: cmd.token }.to_string(),
                    cmd.start_pos,
                );
                return None;
            }
        };

        Some(exit_code)
    }

    fn loc(&self, pos: (usize, usize)) -> TextLoc {
        self.top_pos.clone_with_line_col(pos.0, pos.1)
    }

    fn new_field(&mut self, mut kpi: KeywordParseInfo) {
        let Some(keyword) = self.dico.get(kpi.keyname()) else {
            self.diag.error(
                ParseError::UnknownKey {
                    key: kpi.key.token.clone(),
                }
                .to_string(),
                kpi.key.start_pos.clone(),
            );
            return;
        };

        let nargs: usize = keyword.nargs.try_into().unwrap_or(1);
        kpi.fix_list(&keyword.type_, nargs);
        let value_parse_infos = &mut kpi.values;

        let parse_result = configvalue::parse_value_2(value_parse_infos, &keyword.type_, nargs);

        let value = match parse_result {
            Ok(v) => v,
            Err(errors) => {
                for (entry, reason) in errors {
                    self.diag.error(
                        ParseError::InvalidValue {
                            key: kpi.key.token.clone(),
                            reason,
                        }
                        .to_string(),
                        entry.start_pos.clone(),
                    );
                }
                return;
            }
        };

        if let Some(boundaries) = keyword.boundaries {
            let failures = get_out_of_bounds(&value, boundaries);
            let nb_of_failures = failures.len();
            for failed_index in failures {
                if let Some(failed_value) = value_parse_infos.get(failed_index) {
                    self.diag.error(
                        ParseError::OutOfBound {
                            key: kpi.key.token.clone(),
                            value: failed_value.token.clone(),
                            min: boundaries.0,
                            max: boundaries.1,
                        }
                        .to_string(),
                        failed_value.start_pos.clone(),
                    );
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
                        self.diag.error(
                            ParseError::BadChoice {
                                key: kpi.key.token.clone(),
                                value: failed_value.token.clone(),
                                reason,
                            }
                            .to_string(),
                            failed_value.start_pos.clone(),
                        );
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
