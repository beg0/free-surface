//! # Parser of textual telemac dico files
use std::collections::HashMap;
use std::rc::Rc;

use super::super::configvalue::{parse_single_value_2, parse_value_2, ConfigValue, DicoType};
use super::super::parse_helpers::unquote_single;
use super::super::parse_helpers::{
    DamoclesCommandStatus, DamoclesError, DamoclesParser, KeywordParseInfo, TokenInfo,
};
use super::super::textloc::TextLoc;

use super::dicokeyword::{ChoiceOptionHelp, DicoKeyword, GuiControl, KeywordTextDescription};
use super::{normalize_keyword_name, Dico, DicoInner, ErrorPtr, VecErrorPtr, LOCALES};

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
    #[error(
        "{pos}: Too much values for field '{field}': got {got_count} but expected {expected_count}"
    )]
    TooMuchValues {
        pos: TextLoc,
        field: String,
        got_count: usize,
        expected_count: usize,
    },
    #[error("{pos}: Invalid default value '{value}' in field {field}: {reason}")]
    InvalidDefaultValue {
        field: String,
        value: String,
        reason: String,
        pos: TextLoc,
    },
    // #[error("{pos}: Inconsistent default values")]
    // InconsistentDefaultValues {
    //     field: String,
    //     reason: String,
    //     pos: TextLoc,
    // },
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
    #[error("{pos}: Inconsistent options between choices for different languages. Got {cnt1} and {cnt2} options.")]
    InconsistentChoiceOption {
        pos: TextLoc,
        cnt1: usize,
        cnt2: usize,
    },
}

struct BlockParseInfo {
    val: String,
    pos: TextLoc,
}

/// Parse a Telemac dico file
pub fn parse_dico(input: &str, filename: &str) -> Result<Dico, VecErrorPtr> {
    let file_pos = TextLoc::from((filename, 0));

    let blocks = split_into_blocks(input, &file_pos);
    let mut keywords: Vec<Rc<DicoKeyword>> = Vec::new();
    let mut errors: VecErrorPtr = Vec::new();

    for block in blocks {
        if block.val.trim().is_empty() {
            continue;
        }
        match parse_block(&block.val, &block.pos) {
            Ok(kw) => {
                keywords.push(Rc::new(kw));
            }
            Err(mut errs) => {
                errors.append(&mut errs);
            }
        }
    }

    let mut per_locale: HashMap<String, DicoInner> = HashMap::with_capacity(LOCALES.len());

    for locale in LOCALES {
        let locale = locale.to_owned();
        let mut inner: DicoInner = HashMap::with_capacity(keywords.len());
        for kw in &keywords {
            if let Some(desc) = kw.text_desc.get(&locale) {
                inner.insert(desc.name.clone(), kw.clone());
            }
        }

        per_locale.insert(locale.to_owned(), inner);
    }

    if errors.is_empty() {
        Ok(Dico { per_locale })
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

    // Get a vector of TokenInfo of exactly `expected_count`
    let get_n = |key: &'static str,
                 expected_count: usize,
                 errors: &mut VecErrorPtr|
     -> Option<&Vec<TokenInfo>> {
        let kpi = fields.get(key)?;
        let parse_infos = &kpi.values;
        if parse_infos.len() != expected_count {
            errors.push(Box::new(DicoParseError::TooMuchValues {
                field: key.to_string(),
                pos: if parse_infos.len() >= 2 {
                    parse_infos[1].start_pos.clone()
                } else {
                    block_pos.clone()
                },
                expected_count,
                got_count: parse_infos.len(),
            }));

            None
        } else {
            Some(parse_infos)
        }
    };

    let get_one = |key: &'static str, errors: &mut VecErrorPtr| -> Option<&TokenInfo> {
        get_n(key, 1, errors).map(|v| &v[0])
    };

    let get_val_one = |key: &'static str, errors: &mut VecErrorPtr| -> Option<String> {
        get_one(key, errors).map(|token_info| token_info.token.clone())
    };

    let get_val_n = |key: &'static str,
                     expected_count: usize,
                     errors: &mut VecErrorPtr|
     -> Option<Vec<String>> {
        get_n(key, expected_count, errors).map(|token_infos| {
            token_infos
                .iter()
                .map(|token_info| token_info.token.clone())
                .collect()
        })
    };

    let require_one = |key: &'static str, errors: &mut VecErrorPtr| -> String {
        match get_one(key, errors) {
            Some(token_info) => token_info.token.clone(),
            None => {
                errors.push(Box::new(DicoParseError::MissingField {
                    field: key,
                    pos: block_pos.clone(),
                }));
                String::new()
            }
        }
    };

    let type_ = get_one("TYPE", &mut errors)
        .and_then(|desc| match unquote_single(desc.token.as_str()).as_str() {
            "STRING" | "CARACTERE" => Some(DicoType::String),
            "INTEGER" | "ENTIER" => Some(DicoType::Integer),
            "REAL" | "REEL" => Some(DicoType::Real),
            "LOGICAL" | "LOGIQUE" => Some(DicoType::Logical),
            other => {
                errors.push(Box::new(DicoParseError::InvalidValue {
                    field: "TYPE".into(),
                    reason: format!("unknown type '{}'", other),
                    pos: desc.start_pos.clone(),
                }));
                None
            }
        })
        .unwrap_or(DicoType::String);

    let default_taille = TokenInfo {
        token: String::from("1"),
        start_pos: block_pos.clone(),
        end_pos: block_pos.clone(),
    };

    let taille = parse_u32_field(
        "TAILLE",
        get_one("TAILLE", &mut errors).or(Some(&default_taille)),
        &mut errors,
        block_pos,
    );

    let french_names = ("NOM", "AIDE", "DEFAUT", "CHOIX", "RUBRIQUE");
    let english_names = ("NOM1", "AIDE1", "DEFAUT1", "CHOIX1", "RUBRIQUE1");

    for (locale, names) in [(LOCALES[1], french_names), (LOCALES[0], english_names)] {
        let name = require_one(names.0, &mut errors);
        let help = get_val_one(names.1, &mut errors).unwrap_or_default();
        let kpi_defaults = fields.get(names.2);
        let nargs = taille.try_into().unwrap();

        let default_val: Option<ConfigValue> = kpi_defaults.and_then(|kpi| {
            let values = kpi.fixed_list_values(&type_, nargs);
            match parse_value_2::<ErrorPtr, _>(&values, &type_, nargs, |entry, reason| {
                Box::new(DicoParseError::InvalidDefaultValue {
                    field: String::from(names.2),
                    value: entry.token.clone(),
                    reason,
                    pos: entry.start_pos.clone(),
                })
            }) {
                Ok(res) => Some(res),
                Err(errs) => {
                    errors.extend(errs);
                    None
                }
            }
        });

        let classification: [String; 3] = get_val_n(names.4, 3, &mut errors)
            .map(|values| [values[0].clone(), values[1].clone(), values[2].clone()])
            .unwrap_or(<[String; 3]>::default());

        let choices_text_with_loc = fields
            .get(names.3)
            .map(|kpi| kpi.fixed_list_values(&type_, nargs))
            .unwrap_or_default();

        let mut choices_help: Vec<ChoiceOptionHelp> =
            Vec::with_capacity(choices_text_with_loc.len());
        let mut choices_values: Vec<ConfigValue> = Vec::with_capacity(choices_text_with_loc.len());

        for entry in choices_text_with_loc {
            let (option_token, help_text) = parse_choice_help(entry);

            match parse_single_value_2::<ErrorPtr, _>(&option_token, &type_, |entry, reason| {
                Box::new(DicoParseError::InvalidChoice {
                    field: String::from(names.3),
                    option: entry.token.clone(),
                    reason,
                    pos: entry.start_pos.clone(),
                })
            }) {
                Ok(option) => {
                    choices_values.push(option.clone());
                    choices_help.push(ChoiceOptionHelp {
                        option,
                        help: help_text,
                    });
                }
                Err(errs) => {
                    errors.extend(errs);
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

    // let mnemo = require_one("MNEMO", &mut errors);

    let apparence =
        get_one("APPARENCE", &mut errors).and_then(|token_info| match token_info.token.as_str() {
            "LIST" | "LISTE IS EDITABLE" => Some(GuiControl::List),
            "DYNLIST" => Some(GuiControl::DynList),
            "DYNLIST2" => Some(GuiControl::MultipleDynList),
            "TUPLE" => Some(GuiControl::Tuple),
            "FILE_OR_FOLDER" | "LISTE IS FICHIER" => Some(GuiControl::Path),
            other => {
                errors.push(Box::new(DicoParseError::InvalidValue {
                    field: "APPARENCE".into(),
                    reason: format!("unknown apparence '{}'", other),
                    pos: token_info.start_pos.clone(),
                }));
                None
            }
        });

    // let index = parse_u32_field(
    //     "INDEX",
    //     get_one("INDEX", &mut errors),
    //     &mut errors,
    //     block_pos,
    // );

    let submit = get_val_one("SUBMIT", &mut errors)
        .map(|s| parse_semicolon_list(&s, false))
        .unwrap_or_default();
    let niveau = parse_u32_field(
        "NIVEAU",
        get_one("NIVEAU", &mut errors),
        &mut errors,
        block_pos,
    );

    let controle = get_n("CONTROLE", 2, &mut errors)
        .and_then(|infos| parse_controle(&infos[0], &infos[1], &mut errors));

    let choices_cnt = LOCALES.iter().map(|locale| {
        text_desc
            .get(locale.to_owned())
            .map_or(0, |desc| desc.choices_help.len())
    });

    match all_equals(choices_cnt) {
        Ok(_) => {}
        Err((cnt1, cnt2)) => {
            errors.push(Box::new(DicoParseError::InconsistentChoiceOption {
                pos: block_pos.clone(),
                cnt1,
                cnt2,
            }));
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(DicoKeyword {
        text_desc,
        type_,
        // index,
        nargs: taille,
        submit,
        // mnemo,
        boundaries: controle,
        selection_control: apparence,
        compose: get_val_one("COMPOSE", &mut errors),
        comport: get_val_one("COMPORT", &mut errors),
        level: niveau,
    })
}

struct DicoFieldParser<'a> {
    fields: HashMap<String, KeywordParseInfo>,
    block_pos: &'a TextLoc,
    errors: &'a mut VecErrorPtr,
    known_keys: [&'static str; 20],
}

/// Parse "key = value" pairs from a block, handling multiline values.
/// A new key starts when a line matches "IDENTIFIER = ...".
fn parse_dico_fields(
    block: &str,
    errors: &mut VecErrorPtr,
    block_pos: &TextLoc,
) -> HashMap<String, KeywordParseInfo> {
    let mut parser = DicoFieldParser {
        fields: HashMap::new(),
        block_pos,
        errors,
        known_keys: [
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
        ],
    };

    parser.parse_fields(block);

    parser.fields
}

impl<'a> DamoclesParser for DicoFieldParser<'a> {
    fn error(&mut self, e: ErrorPtr) {
        self.errors.push(e);
    }

    fn cmd(&mut self, cmd: TokenInfo) -> Result<DamoclesCommandStatus, Box<dyn std::error::Error>> {
        let mut exit_code = DamoclesCommandStatus::Success;
        // TODO: better processing of "LIS", "ETA" & "IND" command.
        // For now, they are handled the same way

        match cmd.token[1..].to_ascii_uppercase().as_str() {
            "DYN" => {
                // Consider all dico as "dynamic",  thus ignore this command
            }
            "LIS" | "ETA" | "IND" => {
                dbg!(&self.fields);
            }
            "STO" => {
                return Err(Box::new(DamoclesError::StopCommand {
                    cmd: cmd.token,
                    pos: cmd.start_pos,
                }));
            }
            "FIN" => {
                exit_code = DamoclesCommandStatus::Exit;
            }
            "DOC" => {
                eprintln!("cmd DOC is deprecated");
            }
            _ => {
                return Err(Box::new(DamoclesError::UnknownCommand {
                    cmd: cmd.token,
                    pos: cmd.start_pos,
                }));
            }
        };

        Ok(exit_code)
    }

    fn loc(&self, pos: (usize, usize)) -> TextLoc {
        // pos is 1-based, but here we want an offset, so something which is 0-based
        self.block_pos.clone_with_line_offset_col(pos.0 - 1, pos.1)
    }

    fn new_field(&mut self, kpi: KeywordParseInfo) {
        let key_upper = normalize_keyword_name(kpi.keyname());
        if self.known_keys.contains(&key_upper.as_str()) {
            self.fields.insert(key_upper, kpi);
        } else {
            self.error(Box::new(DicoParseError::UnknownField {
                field: kpi.key.token.to_string(),
                pos: kpi.key.start_pos.clone(),
            }));
        }
    }
}

fn parse_u32_field(
    name: &'static str,
    description: Option<&TokenInfo>,
    errors: &mut VecErrorPtr,
    block_pos: &TextLoc,
) -> u32 {
    match description {
        Some(desc) => desc.token.trim().parse::<u32>().unwrap_or_else(|_| {
            errors.push(Box::new(DicoParseError::InvalidValue {
                field: name.into(),
                reason: format!("'{}' is not a valid unsigned integer", desc.token),
                pos: desc.start_pos.clone(),
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

fn parse_controle(
    min: &TokenInfo,
    max: &TokenInfo,
    errors: &mut VecErrorPtr,
) -> Option<(f64, f64)> {
    match (
        min.token.trim().parse::<f64>(),
        max.token.trim().parse::<f64>(),
    ) {
        (Ok(min_float), Ok(max_float)) => Some((min_float, max_float)),
        (Err(min_err), _) => {
            errors.push(Box::new(DicoParseError::InvalidValue {
                field: "CONTROL".to_owned(),
                reason: format!("Invalid min value '{}': {}", min.token, min_err),
                pos: min.start_pos.clone(),
            }));
            None
        }
        (_, Err(max_err)) => {
            errors.push(Box::new(DicoParseError::InvalidValue {
                field: "CONTROL".to_owned(),
                reason: format!("Invalid max value '{}': {}", max.token, max_err),
                pos: max.start_pos.clone(),
            }));
            None
        }
    }
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

/// Parse one entry in the "CHOIX" (or "CHOIX1") field.
/// It can be in the form "option" or "option=help_text"
fn parse_choice_help(option_and_help: TokenInfo) -> (TokenInfo, String) {
    let text = &option_and_help.token;
    if let Some(eq_pos) = text.find([':', '=']) {
        let non_whitespace = |c: char| !c.is_whitespace();
        let untrimmed_value = &text[..eq_pos];
        let first_non_whitespace = untrimmed_value.find(non_whitespace).unwrap_or(0);

        // Compute new token for the name
        // Assume it starts and end at the same line.
        let new_value = untrimmed_value.trim().to_owned();
        let new_start_pos = option_and_help.start_pos.clone_with_line_offset_col(
            0,
            option_and_help.start_pos.column() + first_non_whitespace,
        );
        let new_end_pos = option_and_help
            .start_pos
            .clone_with_line_offset_col(0, option_and_help.start_pos.column() + new_value.len());

        let option_token = TokenInfo {
            token: new_value,
            start_pos: new_start_pos,
            end_pos: new_end_pos,
        };
        //TODO: often this text is double-quoted. Need to unquote it.
        let help_text = text[eq_pos..].trim().to_owned();

        (option_token, help_text)
    } else {
        // No "equal" sign, it means it's only the name, without an help text
        (option_and_help, String::new())
    }
}

fn all_equals<T: Iterator>(mut iter: T) -> Result<T::Item, (T::Item, T::Item)>
where
    T::Item: Eq + Default,
{
    let first = iter.next();
    if let Some(first_value) = first {
        iter.try_fold(first_value, |prev, new| {
            if prev == new {
                Ok(prev)
            } else {
                Err((prev, new))
            }
        })
    } else {
        Ok(T::Item::default())
    }
}

// cSpell:ignore apparence choix liste fichier entier logique defaut rubrique niveau dynlist
