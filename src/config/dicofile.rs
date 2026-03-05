//! # Dico file parser
//!
//! Parse Telemac dictionary files. Dictionaries contains the list of
//! all keywords allowed
//! for steering files (a.k.a "cas" file) for a given program (Telemac2D,
//!  Telemac3D, Artemis, Tomawac...)

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DicoType {
    String,
    Integer,
    Real,
    Logical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiControl {
    List,
    DynList,
    MultipleDynList,
    Tuple,
    Path,
}

#[derive(Debug, Clone)]
pub struct KeywordTextDescription {
    pub name: String,                // Keyword name
    pub choices: Vec<String>,        // Possible values (if applicable)
    pub default_val: Option<String>, // Default value
    pub classification: [String; 3], // Classification, 3 levels
    pub help: String,
}

/// A single keyword entry from the dico file
#[derive(Debug, Clone)]
pub struct DicoKeyword {
    pub text_desc: HashMap<String, KeywordTextDescription>,
    pub type_: DicoType,
    pub index: u32,
    pub nargs: u32, // Number of time this keyword may occur. 0 means infinite
    pub submit: Vec<String>,
    pub mnemo: String,                         // Variable name in code
    pub boundaries: Option<(f64, f64)>,        // min;max
    pub selection_control: Option<GuiControl>, // Which GUI control widget to use for this entry
    pub compose: Option<String>,
    pub comport: Option<String>,
    pub level: u32, // 0 = mandatory
}

#[derive(Debug, thiserror::Error)]
pub enum DicoParseError {
    #[error("Missing required field '{field}' in keyword block at line {line}")]
    MissingField { field: &'static str, line: usize },
    #[error("Unknown field '{field}' on line {line}")]
    UnknownField { field: String, line: usize },
    #[error("Invalid value for field '{field}' at {line}: {reason}")]
    InvalidValue {
        field: String,
        reason: String,
        line: usize,
    },
    #[error("Malformed line {line}: '{content}'")]
    MalformedLine { line: usize, content: String },
}

struct ValueParseInfo {
    val: String,
    line: usize,
}
struct BlockParseInfo {
    val: String,
    start_line: usize,
}

pub type Dico = Vec<DicoKeyword>;

pub fn parse_dico(input: &str) -> Result<Dico, Vec<DicoParseError>> {
    let blocks = split_into_blocks(input);
    let mut keywords: Dico = Vec::new();
    let mut errors: Vec<DicoParseError> = Vec::new();

    for block in blocks {
        if block.val.trim().is_empty() {
            continue;
        }
        match parse_block(&block.val, block.start_line) {
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
fn split_into_blocks(input: &str) -> Vec<BlockParseInfo> {
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
                    start_line,
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
            start_line,
        });
    }

    blocks
}

/// Parse a single keyword block into key->raw_value pairs, then build a DicoKeyword.
fn parse_block(block: &str, start_line: usize) -> Result<DicoKeyword, Vec<DicoParseError>> {
    let mut errors = Vec::new();
    let fields = parse_fields(block, &mut errors, start_line);

    let mut text_desc: HashMap<String, KeywordTextDescription> = HashMap::new();

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

    let require = |key: &'static str, errors: &mut Vec<DicoParseError>| -> String {
        match fields.get(key) {
            Some(v) => unquote_single(v.val.as_str()),
            None => {
                errors.push(DicoParseError::MissingField {
                    field: key,
                    line: start_line + 1,
                });
                String::new()
            }
        }
    };

    let french_names = ("NOM", "AIDE", "DEFAUT", "CHOIX", "RUBRIQUE");
    let english_names = ("NOM1", "AIDE1", "DEFAUT1", "CHOIX1", "RUBRIQUE1");

    for (locale, names) in [("fr", french_names), ("en", english_names)] {
        let name = require(names.0, &mut errors);
        let help = get_val(names.1).unwrap_or_default();
        let default_val = get_val(names.2);
        let choices = get_raw_val(names.3)
            .map(|s| parse_semicolon_list(s, true))
            .unwrap_or_default();
        let classification = get_raw_val(names.4).map(parse_rubrique).unwrap_or_default();

        text_desc.insert(
            String::from(locale),
            KeywordTextDescription {
                name,
                help,
                default_val,
                choices,
                classification,
            },
        );
    }
    let mnemo = require("MNEMO", &mut errors);

    let type_ = get("TYPE").and_then(|desc| match unquote_single(desc.val.trim()).as_str() {
        "STRING" | "CARACTERE" => Some(DicoType::String),
        "INTEGER" | "ENTIER" => Some(DicoType::Integer),
        "REAL" | "REEL" => Some(DicoType::Real),
        "LOGICAL" | "LOGIQUE" => Some(DicoType::Logical),
        other => {
            errors.push(DicoParseError::InvalidValue {
                field: "TYPE".into(),
                reason: format!("unknown type '{}'", other),
                line: desc.line + 1,
            });
            None
        }
    });

    let index = parse_u32_field("INDEX", get("INDEX"), &mut errors, start_line);

    let default_taille = ValueParseInfo {
        val: String::from("1"),
        line: 0,
    };
    let taille = parse_u32_field(
        "TAILLE",
        get("TAILLE").or(Some(&default_taille)),
        &mut errors,
        start_line,
    );

    let submit = get_val("SUBMIT")
        .map(|s| parse_semicolon_list(&s, false))
        .unwrap_or_default();
    let niveau = parse_u32_field("NIVEAU", get("NIVEAU"), &mut errors, start_line);

    let controle = get("CONTROLE").and_then(|desc| parse_controle(desc, &mut errors));

    let apparence =
        get("APPARENCE").and_then(|desc| match unquote_single(desc.val.trim()).as_str() {
            "LIST" | "LISTE IS EDITABLE" => Some(GuiControl::List),
            "DYNLIST" => Some(GuiControl::DynList),
            "DYNLIST2" => Some(GuiControl::MultipleDynList),
            "TUPLE" => Some(GuiControl::Tuple),
            "FILE_OR_FOLDER" | "LISTE IS FICHIER" => Some(GuiControl::Path),
            other => {
                errors.push(DicoParseError::InvalidValue {
                    field: "APPARENCE".into(),
                    reason: format!("unknown apparence '{}'", other),
                    line: desc.line + 1,
                });
                None
            }
        });

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(DicoKeyword {
        text_desc,
        type_: type_.unwrap_or(DicoType::String),
        index,
        nargs: taille,
        submit,
        mnemo,
        boundaries: controle,
        selection_control: apparence,
        compose: get_val("COMPOSE"),
        comport: get_val("COMPORT"),
        level: niveau,
    })
}

/// Parse "key = value" pairs from a block, handling multiline values.
/// A new key starts when a line matches "IDENTIFIER = ...".
fn parse_fields(
    block: &str,
    errors: &mut Vec<DicoParseError>,
    start_line: usize,
) -> HashMap<String, ValueParseInfo> {
    let mut fields: HashMap<String, ValueParseInfo> = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut current_key_line: usize = start_line;
    let mut current_value = String::new();
    let mut in_quote = false;

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

    for (line_idx, line) in block.lines().enumerate() {
        let trimmed = if in_quote {
            line.trim_end()
        } else {
            line.trim()
        };

        if !in_quote {
            if trimmed.is_empty() {
                continue;
            }

            // Check if this line starts a new key: "KEYWORD = ..."
            // A key is all-uppercase (and underscores/digits), followed by " = "
            if let Some(eq_pos) = find_key_assignment(trimmed) {
                let candidate_key = trimmed[..eq_pos].trim().to_uppercase();

                if known_keys.contains(&candidate_key.as_str()) {
                    // Save the previous key
                    if let Some(key) = current_key.take() {
                        fields.insert(
                            key,
                            ValueParseInfo {
                                val: current_value.trim().to_string(),
                                line: current_key_line,
                            },
                        );
                    }
                    current_key = Some(candidate_key);
                    current_key_line = start_line + line_idx;
                    current_value = trimmed[eq_pos + 1..].trim().to_string();
                    continue;
                } else {
                    errors.push(DicoParseError::UnknownField {
                        field: candidate_key,
                        line: start_line + line_idx + 1,
                    });
                }
            }
        }

        let quote_count = trimmed.chars().filter(|c| *c == '\'').count();

        // If there is an even number of quote, it means we either close or open a quote
        // block
        if (quote_count % 2) == 1 {
            in_quote = !in_quote
        }

        // Continuation of current value
        if current_key.is_some() {
            current_value.push('\n');
            current_value.push_str(trimmed);
        }
    }

    // Don't forget the last key
    if let Some(key) = current_key {
        fields.insert(
            key,
            ValueParseInfo {
                val: current_value.trim().to_string(),
                line: current_key_line,
            },
        );
    }

    fields
}

/// Returns the position of '=' if the line looks like "KEY = value"
/// where KEY is uppercase letters, digits, underscores, and spaces.
fn find_key_assignment(line: &str) -> Option<usize> {
    let eq_pos = line.find('=')?;
    let key_part = line[..eq_pos].trim();
    // Key must be non-empty and contain only uppercase letters, digits, underscores
    if key_part.is_empty() {
        return None;
    }
    let valid = key_part
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if valid {
        Some(eq_pos)
    } else {
        None
    }
}

fn parse_u32_field(
    name: &'static str,
    description: Option<&ValueParseInfo>,
    errors: &mut Vec<DicoParseError>,
    block_start_line: usize,
) -> u32 {
    match description {
        Some(desc) => desc.val.trim().parse::<u32>().unwrap_or_else(|_| {
            errors.push(DicoParseError::InvalidValue {
                field: name.into(),
                reason: format!("'{}' is not a valid unsigned integer", desc.val),
                line: desc.line + 1,
            });
            0
        }),
        None => {
            errors.push(DicoParseError::MissingField {
                field: name,
                line: block_start_line + 1,
            });
            0
        }
    }
}

fn parse_controle(desc: &ValueParseInfo, errors: &mut Vec<DicoParseError>) -> Option<(f64, f64)> {
    let parts: Vec<&str> = desc.val.split(';').collect();
    if parts.len() != 2 {
        errors.push(DicoParseError::InvalidValue {
            field: "CONTROLE".into(),
            reason: format!("expected 'min;max', got '{}'", desc.val),
            line: desc.line + 1,
        });
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

/// Remove surrounding single quotes, and unescape '' -> '
fn unquote_single(s: &str) -> String {
    let inner = if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    };
    inner.replace("''", "'")
}

// cSpell:ignore apparence choix liste fichier entier logique defaut rubrique niveau dynlist
