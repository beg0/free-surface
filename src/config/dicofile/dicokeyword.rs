//! # Keywords within a Telemac dico file
use std::collections::HashMap;

use super::super::configvalue::{ConfigValue, DicoType};
use super::{normalize_keyword_name, LOCALES};

#[derive(Debug, thiserror::Error)]
pub enum ChoiceValidationError {
    #[error("Value {value:?} is not a valid choice")]
    NotFound {
        value: ConfigValue,
        choices: Vec<ChoiceOptionHelp>,
    },
    #[error("Something wrong with value {value:?}: {reason}")]
    InternalError { value: ConfigValue, reason: String },
}

/// Localized choices options for a [KeywordTextDescription]
#[derive(Debug, Clone)]
pub struct ChoiceOptionHelp {
    pub option: ConfigValue,
    pub _help: String,
}

/// Localized data for a [DicoKeyword]
#[derive(Debug, Clone)]
pub struct KeywordTextDescription {
    pub name: String,                        // Keyword name
    pub choices_help: Vec<ChoiceOptionHelp>, // Help text for each possible values
    pub default_val: Option<ConfigValue>,    // Default value

    #[allow(dead_code)]
    pub classification: [String; 3], // Classification, 3 levels

    #[allow(dead_code)]
    pub help: String, // Keyword description
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiControl {
    List,
    DynList,
    MultipleDynList,
    Tuple,
    Path,
}

/// A single keyword entry from the dico file
#[derive(Debug, Clone)]
pub struct DicoKeyword {
    pub text_desc: HashMap<String, KeywordTextDescription>, // localized description
    pub type_: DicoType, // Type of value that are stored for this keyword
    // pub index: u32,          // Unused in free-surface
    pub nargs: u32,          // Number of time this keyword may occur. 0 means infinite
    pub submit: Vec<String>, // If not empty, location to store the config in the Bief file.
    // pub mnemo: String,       // Variable name in code
    pub boundaries: Option<(f64, f64)>,        // min;max
    pub selection_control: Option<GuiControl>, // Which GUI control widget to use for this entry
    pub compose: Option<String>,               // Unused in free-surface
    pub comport: Option<String>,               // Unused in free-surface
    pub level: u32,                            // 0 = mandatory
}

impl DicoKeyword {
    pub fn name(&self) -> &String {
        &self
            .text_desc
            .get(LOCALES[0])
            .expect("No english description")
            .name
    }

    pub fn default(&self) -> ConfigValue {
        let text_desc = &self.text_desc.get(LOCALES[0]);

        let default_generator = || {
            if self.nargs == 1 {
                match &self.type_ {
                    DicoType::String => ConfigValue::String(String::new()),
                    DicoType::Integer => ConfigValue::Integer(0),
                    DicoType::Logical => ConfigValue::Boolean(false),
                    DicoType::Real => ConfigValue::Float(0.0),
                }
            } else {
                match &self.type_ {
                    DicoType::String => ConfigValue::StringCollection(vec![]),
                    DicoType::Integer => ConfigValue::IntegerCollection(vec![]),
                    DicoType::Logical => ConfigValue::BooleanCollection(vec![]),
                    DicoType::Real => ConfigValue::FloatCollection(vec![]),
                }
            }
        };
        text_desc
            .and_then(|desc| desc.default_val.clone())
            .unwrap_or_else(default_generator)
    }

    pub fn has_choices(&self) -> bool {
        let choice_cnt = self
            .text_desc
            .iter()
            .fold(0, |acc, (_locale, desc)| acc + desc.choices_help.len());
        choice_cnt > 0
    }

    fn get_all_choices(&self) -> Vec<ChoiceOptionHelp> {
        let mut ret: Vec<ChoiceOptionHelp> = Vec::new();
        for desc in self.text_desc.values() {
            ret.append(&mut desc.choices_help.clone());
        }
        ret
    }

    pub fn normalize_choice(
        &self,
        option: &ConfigValue,
    ) -> Result<ConfigValue, Vec<(usize, ChoiceValidationError)>> {
        let values: Vec<ConfigValue>;
        let mut errors: Vec<(usize, ChoiceValidationError)> = Vec::new();

        let error_mapper = |reason| {
            vec![(
                0,
                ChoiceValidationError::InternalError {
                    value: option.clone(),
                    reason,
                },
            )]
        };

        if self.has_choices() {
            if option.is_scalar() {
                values = vec![option.clone()];
            } else {
                values = option.clone().into_scalars().map_err(error_mapper)?;
            }

            let mut output_vec: Vec<ConfigValue> = Vec::with_capacity(values.len());

            #[allow(clippy::needless_range_loop)]
            for index in 0..values.len() {
                let candidate = &values[index];
                match self.normalize_single_choice(candidate) {
                    Ok(new_value) => output_vec.push(new_value),
                    Err(reason) => {
                        // Be permissive on DynList
                        if self.selection_control == Some(GuiControl::DynList) {
                            output_vec.push(candidate.clone());
                        } else {
                            errors.push((index, reason))
                        }
                    }
                };
            }

            if !errors.is_empty() {
                Err(errors)
            } else if option.is_scalar() {
                Ok(output_vec.remove(0))
            } else {
                ConfigValue::collect(output_vec).map_err(error_mapper)
            }
        } else {
            Ok(option.clone())
        }
    }

    fn normalize_single_choice(
        &self,
        value: &ConfigValue,
    ) -> Result<ConfigValue, ChoiceValidationError> {
        let mut option_pos: Option<usize> = None;
        for desc in self.text_desc.values() {
            let upper_value: ConfigValue;
            let normalized_value: &ConfigValue;
            if let ConfigValue::String(s) = value {
                upper_value = ConfigValue::String(normalize_keyword_name(s));
                normalized_value = &upper_value;
            } else {
                normalized_value = value;
            }
            let research = desc
                .choices_help
                .iter()
                .position(|choice_help| choice_help.option == *normalized_value);

            if let Some(pos) = research {
                option_pos = Some(pos);
                break;
            }
        }

        // Return the english version of the choice
        // Note that, even if the original choice is in english,
        // return the one in self.text_desc, so that we are sure it is normalized
        // this is especially true for string choices (which can be trimmed and/or lowercase)
        match option_pos {
            Some(pos) => {
                let english_desc = self
                    .text_desc
                    .get(LOCALES[0])
                    .expect("No 'en' text description in keyword");

                let english_option_help = english_desc
                    .choices_help
                    .get(pos)
                    .expect("Length of 'en' and other language does not match");

                Ok(english_option_help.option.clone())
            }
            None => Err(ChoiceValidationError::NotFound {
                value: value.clone(),
                choices: self.get_all_choices(),
            }),
        }
    }
}
