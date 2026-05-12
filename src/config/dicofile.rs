//! # Dico file parser
//!
//! Parse Telemac dictionary files. Dictionaries contains the list of
//! all keywords allowed
//! for steering files (a.k.a "cas" file) for a given program (Telemac2D,
//! Telemac3D, Artemis, Tomawac...)
use super::configvalue::{parse_single_value, parse_value, ConfigValue, DicoType};
use super::parse_helpers::{find_key_assignment, parse_fields, unquote_single};
use super::textloc::TextLoc;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum GuiControl {
    List,
    DynList,
    MultipleDynList,
    Tuple,
    Path,
}

type ErrorPtr = Box<dyn std::error::Error>;
type VecErrorPtr = Vec<ErrorPtr>;

#[derive(Debug, Clone)]
pub struct ChoiceOptionHelp {
    option: ConfigValue,
    _help: String,
}

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

/// A single keyword entry from the dico file
#[derive(Debug, Clone)]
pub struct DicoKeyword {
    pub text_desc: HashMap<String, KeywordTextDescription>, // localized description
    pub type_: DicoType,
    // pub index: u32,
    pub nargs: u32, // Number of time this keyword may occur. 0 means infinite
    pub submit: Vec<String>,
    //pub mnemo: String,                         // Variable name in code
    pub boundaries: Option<(f64, f64)>,        // min;max
    pub selection_control: Option<GuiControl>, // Which GUI control widget to use for this entry
    pub compose: Option<String>,
    pub comport: Option<String>,
    pub level: u32, // 0 = mandatory
}

#[derive(Debug, thiserror::Error)]
pub enum DicoParseError {
    #[error("{pos}: Missing required field '{field}' in keyword block")]
    MissingField { field: &'static str, pos: TextLoc },
    #[error("{pos}: Unknown field '{field}'")]
    UnknownField { field: String, pos: TextLoc },
    #[error("{pos}: Invalid value for field '{field}': {reason}")]
    InvalidValue {
        field: String,
        reason: String,
        pos: TextLoc,
    },
    #[error("{pos}: Invalid default value '{value}' in field {field}: {reason}")]
    InvalidDefaultValue {
        field: String,
        value: String,
        reason: String,
        pos: TextLoc,
    },
    #[error("{pos}: Invalid value for choice '{option}' in field {field}: {reason}")]
    InvalidChoice {
        field: String,
        option: String,
        reason: String,
        pos: TextLoc,
    },
    // #[error("{pos}: Inconsistent options between choices for {lang1} and {lang2}. Got {choices1} and {choices2}")]
    // InconsistentChoiceOption {
    //     lang1: String,
    //     lang2: String,
    //     choices1: Vec<ConfigValue>,
    //     choices2: Vec<ConfigValue>,
    //     pos: TextLoc,
    // },
    #[error("{pos}: Inconsistent options between choices for different languages.")]
    InconsistentChoiceOption { pos: TextLoc },
}

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
struct ValueParseInfo {
    val: String,
    pos: TextLoc,
}
struct BlockParseInfo {
    val: String,
    pos: TextLoc,
}

pub type Dico = Vec<DicoKeyword>;

pub fn parse_dico(input: &str, filename: &str) -> Result<Dico, VecErrorPtr> {
    let file_pos = TextLoc::from((filename, 0));

    let blocks = split_into_blocks(input, &file_pos);
    let mut keywords: Dico = Vec::new();
    let mut errors: VecErrorPtr = Vec::new();

    for block in blocks {
        if block.val.trim().is_empty() {
            continue;
        }
        match parse_block(&block.val, &block.pos) {
            Ok(kw) => keywords.push(kw),
            Err(mut errs) => errors.append(&mut errs),
        }
    }

    if errors.is_empty() {
        Ok(keywords)
    } else {
        Err(errors)
    }
}

/// Split the file into blocks delimited by a lone "/" on its own line.
/// Lines starting with "///" or "////////" are section headers - skip them.
fn split_into_blocks(input: &str, file_pos: &TextLoc) -> Vec<BlockParseInfo> {
    let mut blocks: Vec<BlockParseInfo> = Vec::new();
    let mut current = String::new();
    let mut start_line = 0;

    for (line_idx, line) in input.lines().enumerate() {
        let trimmed = line.trim();

        // Section header comments (/// or more slashes) - skip
        if trimmed.starts_with("///") {
            continue;
        }

        // A lone "/" or "&DYN" marks a block boundary
        if trimmed == "/" || trimmed == "&DYN" {
            if !current.trim().is_empty() {
                blocks.push(BlockParseInfo {
                    val: current,
                    pos: file_pos.clone_with_line(start_line + 1),
                });
                current = String::new();
                start_line = line_idx;
            }
            continue;
        }

        // Full-line comments starting with "/" or "#" - skip
        if trimmed.starts_with('/') || trimmed.starts_with('#') {
            continue;
        }

        current.push_str(line);
        current.push('\n');
    }

    if !current.trim().is_empty() {
        blocks.push(BlockParseInfo {
            val: current,
            pos: file_pos.clone_with_line(start_line + 1),
        });
    }

    blocks
}

/// Parse a single keyword block into key->raw_value pairs, then build a DicoKeyword.
fn parse_block(block: &str, block_pos: &TextLoc) -> Result<DicoKeyword, VecErrorPtr> {
    let mut errors = Vec::new();
    let fields = parse_dico_fields(block, &mut errors, block_pos);

    let mut text_desc: HashMap<String, KeywordTextDescription> = HashMap::new();
    let mut choices_per_local: HashMap<String, Vec<ConfigValue>> = HashMap::new();

    // Helper closures
    let get = |key: &'static str| -> Option<&ValueParseInfo> { fields.get(key) };
    let get_raw_val = |key: &'static str| -> Option<&String> {
        match fields.get(key) {
            Some(desc) => Some(&desc.val),
            None => None,
        }
    };

    let get_val = |key: &'static str| -> Option<String> {
        get_raw_val(key).map(|val| unquote_single(val.as_str()))
    };

    let require = |key: &'static str, errors: &mut VecErrorPtr| -> String {
        match fields.get(key) {
            Some(v) => unquote_single(v.val.as_str()),
            None => {
                errors.push(Box::new(DicoParseError::MissingField {
                    field: key,
                    pos: block_pos.clone(),
                }));
                String::new()
            }
        }
    };

    let type_ = get("TYPE")
        .and_then(|desc| match unquote_single(desc.val.trim()).as_str() {
            "STRING" | "CARACTERE" => Some(DicoType::String),
            "INTEGER" | "ENTIER" => Some(DicoType::Integer),
            "REAL" | "REEL" => Some(DicoType::Real),
            "LOGICAL" | "LOGIQUE" => Some(DicoType::Logical),
            other => {
                errors.push(Box::new(DicoParseError::InvalidValue {
                    field: "TYPE".into(),
                    reason: format!("unknown type '{}'", other),
                    pos: desc.pos.clone(),
                }));
                None
            }
        })
        .unwrap_or(DicoType::String);

    let default_taille = ValueParseInfo {
        val: String::from("1"),
        pos: block_pos.clone(),
    };

    let taille = parse_u32_field(
        "TAILLE",
        get("TAILLE").or(Some(&default_taille)),
        &mut errors,
        block_pos,
    );

    let french_names = ("NOM", "AIDE", "DEFAUT", "CHOIX", "RUBRIQUE");
    let english_names = ("NOM1", "AIDE1", "DEFAUT1", "CHOIX1", "RUBRIQUE1");

    for (locale, names) in [("fr", french_names), ("en", english_names)] {
        let name = require(names.0, &mut errors);
        let help = get_val(names.1).unwrap_or_default();
        let default_val_str = get(names.2);

        let default_val = default_val_str.and_then(|desc| {
            match parse_value(desc.val.as_str(), &type_, taille.try_into().unwrap()) {
                Ok(option) => Some(option),
                Err(reason) => {
                    errors.push(Box::new(DicoParseError::InvalidDefaultValue {
                        field: String::from(names.2),
                        value: desc.val.clone(),
                        reason,
                        pos: desc.pos.clone(),
                    }));
                    None
                }
            }
        });

        let choices_text_with_loc = get(names.3)
            .map(|desc| (parse_semicolon_list(&desc.val, true), desc.pos.clone()))
            .unwrap_or((Vec::<String>::new(), TextLoc::default()));

        let classification = get_raw_val(names.4).map(parse_rubrique).unwrap_or_default();

        let choices_text = &choices_text_with_loc.0;
        let mut choices_help: Vec<ChoiceOptionHelp> = Vec::with_capacity(choices_text.len());
        let mut choices_values: Vec<ConfigValue> = Vec::with_capacity(choices_text.len());

        for entry in choices_text {
            let option_text: &str;
            let help_text: String;
            if let Some(eq_pos) = find_key_assignment(entry.as_str(), validate_choice_key) {
                option_text = entry.as_str()[..eq_pos].trim();
                help_text = String::from(entry.as_str()[eq_pos..].trim());
            } else {
                option_text = entry.as_str();
                help_text = String::new();
            };

            match parse_single_value(option_text, &type_) {
                Ok(option) => {
                    choices_values.push(option.clone());
                    choices_help.push(ChoiceOptionHelp {
                        option,
                        _help: help_text,
                    });
                }
                Err(reason) => {
                    errors.push(Box::new(DicoParseError::InvalidChoice {
                        field: String::from(names.3),
                        option: String::from(option_text),
                        reason,
                        pos: choices_text_with_loc.1.clone(),
                    }));
                }
            };
        }

        text_desc.insert(
            String::from(locale),
            KeywordTextDescription {
                name,
                help,
                default_val,
                choices_help,
                classification,
            },
        );
        choices_per_local.insert(String::from(locale), choices_values);
    }

    // let mnemo = require("MNEMO", &mut errors);

    let apparence =
        get("APPARENCE").and_then(|desc| match unquote_single(desc.val.trim()).as_str() {
            "LIST" | "LISTE IS EDITABLE" => Some(GuiControl::List),
            "DYNLIST" => Some(GuiControl::DynList),
            "DYNLIST2" => Some(GuiControl::MultipleDynList),
            "TUPLE" => Some(GuiControl::Tuple),
            "FILE_OR_FOLDER" | "LISTE IS FICHIER" => Some(GuiControl::Path),
            other => {
                errors.push(Box::new(DicoParseError::InvalidValue {
                    field: "APPARENCE".into(),
                    reason: format!("unknown apparence '{}'", other),
                    pos: desc.pos.clone(),
                }));
                None
            }
        });

    // let index = parse_u32_field("INDEX", get("INDEX"), &mut errors, block_pos);
    let submit = get_val("SUBMIT")
        .map(|s| parse_semicolon_list(&s, false))
        .unwrap_or_default();
    let niveau = parse_u32_field("NIVEAU", get("NIVEAU"), &mut errors, block_pos);

    let controle = get("CONTROLE").and_then(|desc| parse_controle(desc, &mut errors));

    // FIXME: we check here that both english & french "CHOIX" expose
    // the same options. We need to make this code more generic to handle other langages
    let choices_cnt_fr = text_desc
        .get("fr")
        .map_or(0, |desc| desc.choices_help.len());
    let choices_cnt_en = text_desc
        .get("en")
        .map_or(0, |desc| desc.choices_help.len());

    if choices_cnt_fr != choices_cnt_en {
        errors.push(Box::new(DicoParseError::InconsistentChoiceOption {
            pos: block_pos.clone(),
        }));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(DicoKeyword {
        text_desc,
        type_,
        //index,
        nargs: taille,
        submit,
        // mnemo,
        boundaries: controle,
        selection_control: apparence,
        compose: get_val("COMPOSE"),
        comport: get_val("COMPORT"),
        level: niveau,
    })
}

fn validate_dico_key(candidate: &str) -> bool {
    candidate
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

fn validate_choice_key(candidate: &str) -> bool {
    candidate.chars().all(|c| {
        c.is_ascii_uppercase()
            || c.is_ascii_digit()
            || c == '+'
            || c == '-'
            || c == '*'
            || c == '_'
            || c == '?'
    })
}
/// Parse "key = value" pairs from a block, handling multiline values.
/// A new key starts when a line matches "IDENTIFIER = ...".
fn parse_dico_fields(
    block: &str,
    errors: &mut VecErrorPtr,
    block_pos: &TextLoc,
) -> HashMap<String, ValueParseInfo> {
    let mut fields: HashMap<String, ValueParseInfo> = HashMap::new();

    let known_keys = [
        "NOM1",
        "NOM",
        "TYPE",
        "INDEX",
        "TAILLE",
        "SUBMIT",
        "DEFAUT1",
        "DEFAUT",
        "MNEMO",
        "CONTROLE",
        "CHOIX1",
        "CHOIX",
        "APPARENCE",
        "RUBRIQUE1",
        "RUBRIQUE",
        "COMPOSE",
        "COMPORT",
        "NIVEAU",
        "AIDE1",
        "AIDE",
    ];

    parse_fields(
        block,
        block_pos,
        |key: &str, value: String, pos: TextLoc| {
            let key_upper = key.to_uppercase();
            if known_keys.contains(&key_upper.as_str()) {
                fields.insert(key_upper, ValueParseInfo { val: value, pos });
            } else {
                errors.push(Box::new(DicoParseError::UnknownField {
                    field: key.to_string(),
                    pos,
                }));
            }
        },
        validate_dico_key,
    );

    fields
}

fn parse_u32_field(
    name: &'static str,
    description: Option<&ValueParseInfo>,
    errors: &mut VecErrorPtr,
    block_pos: &TextLoc,
) -> u32 {
    match description {
        Some(desc) => desc.val.trim().parse::<u32>().unwrap_or_else(|_| {
            errors.push(Box::new(DicoParseError::InvalidValue {
                field: name.into(),
                reason: format!("'{}' is not a valid unsigned integer", desc.val),
                pos: desc.pos.clone(),
            }));
            0
        }),
        None => {
            errors.push(Box::new(DicoParseError::MissingField {
                field: name,
                pos: block_pos.clone(),
            }));
            0
        }
    }
}

fn parse_controle(desc: &ValueParseInfo, errors: &mut VecErrorPtr) -> Option<(f64, f64)> {
    let parts: Vec<&str> = desc.val.split(';').collect();
    if parts.len() != 2 {
        errors.push(Box::new(DicoParseError::InvalidValue {
            field: "CONTROLE".into(),
            reason: format!("expected 'min;max', got '{}'", desc.val),
            pos: desc.pos.clone(),
        }));
        return None;
    }
    let min = parts[0].trim().parse::<f64>().ok()?;
    let max = parts[1].trim().parse::<f64>().ok()?;
    Some((min, max))
}

/// Parse a semicolon-separated list of possibly single-quoted strings.
/// Handles multiline values like:
///   'CHOICE A';
///   'CHOICE B';
///   'CHOICE C'
fn parse_semicolon_list(raw: &String, unquote_entries: bool) -> Vec<String> {
    let to_split = if !unquote_entries {
        &unquote_single(raw.as_str())
    } else {
        raw
    };

    let single_entry_unquote_fct = if unquote_entries {
        |s: &str| unquote_single(s.trim())
    } else {
        |s: &str| String::from(s)
    };

    to_split
        .split(';')
        .map(single_entry_unquote_fct)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse RUBRIQUE into exactly 3 levels, padding with empty strings.
fn parse_rubrique(raw: &String) -> [String; 3] {
    let items = parse_semicolon_list(raw, true);
    [
        items.first().cloned().unwrap_or_default(),
        items.get(1).cloned().unwrap_or_default(),
        items.get(2).cloned().unwrap_or_default(),
    ]
}

impl DicoKeyword {
    pub fn name(&self) -> &String {
        &self
            .text_desc
            .get("en")
            .expect("No english description")
            .name
    }

    pub fn default(&self) -> ConfigValue {
        let text_desc = &self.text_desc.get("en");

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
    ) -> Result<ConfigValue, Vec<ChoiceValidationError>> {
        let values: Vec<ConfigValue>;
        let mut errors: Vec<ChoiceValidationError> = Vec::new();

        let error_mapper = |reason| {
            vec![ChoiceValidationError::InternalError {
                value: option.clone(),
                reason,
            }]
        };

        if self.has_choices() {
            if option.is_scalar() {
                values = vec![option.clone()];
            } else {
                values = option.clone().into_scalars().map_err(error_mapper)?;
            }

            let mut output_vec: Vec<ConfigValue> = Vec::with_capacity(values.len());
            for candidate in &values {
                match self.normalize_single_choice(candidate) {
                    Ok(new_value) => output_vec.push(new_value),
                    Err(reason) => errors.push(reason),
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
        let mut option_locale: Option<&String> = None;
        for (locale, desc) in &self.text_desc {
            let research = desc
                .choices_help
                .iter()
                .position(|choice_help| choice_help.option == *value);

            if let Some(pos) = research {
                option_pos = Some(pos);
                option_locale = Some(locale);
                break;
            }
        }
        match option_pos {
            Some(pos) => {
                if option_locale
                    .expect("'option_pos' not null but 'option_locale' not defined")
                    .as_str()
                    == "en"
                {
                    Ok(value.clone())
                } else {
                    let english_desc = self
                        .text_desc
                        .get("en")
                        .expect("No 'en' text description in keyword");

                    let english_option_help = english_desc
                        .choices_help
                        .get(pos)
                        .expect("Length of 'en' and other language does not match");

                    Ok(english_option_help.option.clone())
                }
            }
            None => Err(ChoiceValidationError::NotFound {
                value: value.clone(),
                choices: self.get_all_choices(),
            }),
        }
    }
}

// cSpell:ignore apparence choix liste fichier entier logique defaut rubrique niveau dynlist
