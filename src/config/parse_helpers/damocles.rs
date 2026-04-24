//! # Damocles - helper to parse Telemac config file format
//!
//! ## Format description
//!
//! The Telemac config files (Dico files and stering files) are (ASCII) text files.
//! The format is as following:
//!
//! - Comments start with a single slash '/' and ends with a new line
//!
//!  - The files contain a list of "key = value(s)" pairs. There is no section
//!  - Assignment operator can be equal sign "=" or column ":".
//!  - A key can have multiple values. In such case, values are separated with semi-column ';'
//!  - a key can contains whitespace (without being quoted)
//!  - a value that contain spaces must be quoted with single quote (double quote not supported by Telemac)
//!  - in single quoted values, the quote character is escaped by doubling it.
//!    e.g. 'this is a sentence ''with quote'' in it'
//!  - All keys & values seem to be uppercase, but there is no technical constraint on that
//!
//! Additionally, there can be "commands" in the file that modify the processing of the file.
//! Commands starts with a '&' and are followed by 3 uppercase characters
//!
//! ## Fun facts
//!
//! The term "Damocles" is a french word games in Telemac that mix:
//! - Damocles, a character in Greek mythology
//! - the term "mot-clé" which means keyword, prefixed with "Da"
//!
//! In original Telemac code, Damocles subroutine is used to parse config file.
//! This module re-use the same name for the module in charge of parsing config file
//!

use std::iter::Iterator;

use super::super::textloc::TextLoc;
use super::locatedchars;
use super::unquote_single;

pub use super::keywordparseinfo::{KeywordParseInfo, TokenInfo};

#[cfg(test)]
mod tests;

/// Errors that can occur while parsing Telemac config files
#[derive(Debug, thiserror::Error)]
pub enum DamoclesError {
    #[error("{pos}: Unexpected assignment '{assignment}'.")]
    UnexpectedAssignment { assignment: char, pos: TextLoc },

    #[error("{pos}: Unexpected list separator '{sep}'.")]
    UnexpectedListSeparator { sep: char, pos: TextLoc },

    #[error("{pos}: Unexpected list separator '{sep}' after key '{key}'.")]
    UnexpectedListSeparatorAfterKey {
        sep: char,
        key: String,
        pos: TextLoc,
    },

    #[error("{pos}: Missing terminal quote '{quote}'.")]
    MissingEndQuote { quote: char, pos: TextLoc },

    #[error("{pos}: Unexpected token, expected assignment ':' or '='.")]
    MissingAssignment { pos: TextLoc },

    #[error("{pos}: Missing value for key {key}.")]
    MissingEndValue { key: String, pos: TextLoc },

    #[error("{pos}: Invalid character {char}.")]
    NonPrintableCharacter { char: char, pos: TextLoc },
}

/// Line & Column position
type LineCol = (usize, usize);

/// States of the tokenizer in the parse_fields state machine
#[derive(PartialEq)]
enum TokenizerState {
    Outside,     // outside any keyword, eat whitespaces until we find a non-whitespace
    PlainToken,  // Parsing token without quote
    SingleQuote, // in a 'single quote' string
    DoubleQuote, // in a "double quote" string
    Comment,     // in a comment
}

struct DamoclesParseContext<'a, T: DamoclesParser + ?Sized> {
    token_start_pos: LineCol,
    token_start_offset: usize,
    last_non_whitespace_pos: LineCol,
    last_non_whitespace_offset: usize,
    field_parser: &'a mut T,
}

impl<'a, T: DamoclesParser> DamoclesParseContext<'a, T> {
    pub fn new(field_parser: &'a mut T) -> Self {
        Self {
            token_start_pos: (0, 0),
            token_start_offset: 0,
            last_non_whitespace_pos: (0, 0),
            last_non_whitespace_offset: 0,
            field_parser,
        }
    }

    fn create_token(&mut self, input: &str) -> Option<TokenInfo> {
        let raw_val = &input[self.token_start_offset..self.last_non_whitespace_offset];
        let ret = Some(TokenInfo {
            token: unquote_single(raw_val),
            start_pos: self.field_parser.loc(self.token_start_pos),
            end_pos: self.field_parser.loc(self.last_non_whitespace_pos),
        });
        self.token_start_offset = 0;
        self.token_start_pos = (0, 0);
        self.last_non_whitespace_offset = 0;
        self.last_non_whitespace_pos = (0, 0);
        ret
    }

    pub fn parse_fields(&mut self, input: &str) {
        let mut chars = locatedchars::LocatedChars::new(input);

        let mut key: Option<TokenInfo> = None;
        let mut values: Vec<TokenInfo> = Vec::new();
        let mut token: Option<TokenInfo> = None;

        // FSM variables
        let mut tokenizer_state = TokenizerState::Outside;
        let mut expected_value_cnt: usize = 1;
        let mut find_assignment = false;

        while let Some((i, c)) = chars.next() {
            match c {
                '"' => match tokenizer_state {
                    TokenizerState::Outside => {
                        tokenizer_state = TokenizerState::DoubleQuote;
                        self.token_start_offset = i;
                        self.token_start_pos = chars.pos();
                    }
                    TokenizerState::PlainToken => {
                        tokenizer_state = TokenizerState::DoubleQuote;
                        // token_start = i; // Don't restart the token start. Consider as part of previous token
                    }
                    TokenizerState::SingleQuote | TokenizerState::Comment => {}
                    TokenizerState::DoubleQuote => {
                        // Don't end the token. It may continue with a PlainToken
                        //token = Some(&input[token_start..i]);
                        tokenizer_state = TokenizerState::PlainToken;
                    }
                },
                '\'' => match tokenizer_state {
                    TokenizerState::Outside => {
                        tokenizer_state = TokenizerState::SingleQuote;
                        self.token_start_offset = i;
                        self.token_start_pos = chars.pos();
                    }
                    TokenizerState::PlainToken => {
                        tokenizer_state = TokenizerState::SingleQuote;
                        // Don't restart the token start. Consider as part of previous token
                        // token_start = i;
                    }
                    TokenizerState::DoubleQuote | TokenizerState::Comment => {}
                    TokenizerState::SingleQuote => {
                        if chars.next_if_eq('\'').is_some() {
                            // consume the second ', it's an escape
                        } else {
                            // Don't end the token. It may continue with a PlainToken
                            //token = Some(&input[token_start..i]);
                            tokenizer_state = TokenizerState::PlainToken;
                        }
                    }
                },
                '/' | '#' => match tokenizer_state {
                    TokenizerState::Outside => {
                        tokenizer_state = TokenizerState::Comment;
                    }
                    TokenizerState::PlainToken => {
                        token = self.create_token(input);
                        tokenizer_state = TokenizerState::Comment;
                    }
                    TokenizerState::SingleQuote
                    | TokenizerState::DoubleQuote
                    | TokenizerState::Comment => {}
                },
                '\r' | '\n' => {
                    match tokenizer_state {
                        TokenizerState::Comment => {
                            tokenizer_state = TokenizerState::Outside;
                        }
                        // new line is a valid condition to stop a token (but space is not)
                        TokenizerState::PlainToken => {
                            token = self.create_token(input);
                            tokenizer_state = TokenizerState::Outside;
                        }

                        TokenizerState::SingleQuote
                        | TokenizerState::DoubleQuote
                        | TokenizerState::Outside => {}
                    }
                }
                ':' | '=' => match tokenizer_state {
                    TokenizerState::PlainToken => {
                        token = self.create_token(input);
                        if find_assignment {
                            let e = Box::new(DamoclesError::UnexpectedAssignment {
                                assignment: c,
                                pos: self.field_parser.loc(chars.pos()),
                            });
                            self.field_parser.error(e);
                        }
                        tokenizer_state = TokenizerState::Outside;
                        find_assignment = true;
                    }
                    TokenizerState::SingleQuote
                    | TokenizerState::DoubleQuote
                    | TokenizerState::Comment => {}
                    TokenizerState::Outside => {
                        if find_assignment || key.is_none() {
                            let e = Box::new(DamoclesError::UnexpectedAssignment {
                                assignment: c,
                                pos: self.field_parser.loc(chars.pos()),
                            });
                            self.field_parser.error(e);
                        }
                    }
                },
                // List separator
                ';' => {
                    match tokenizer_state {
                        TokenizerState::PlainToken => {
                            token = self.create_token(input);

                            // List separator before a key is set.
                            // This means a ';' instead of a ':' or '=' (or the key is missing)
                            if key.is_none() {
                                let e = Box::new(DamoclesError::UnexpectedListSeparatorAfterKey {
                                    sep: c,
                                    key: token.clone().unwrap().token.clone(),
                                    pos: self.field_parser.loc(chars.pos()),
                                });
                                self.field_parser.error(e);
                            } else {
                                expected_value_cnt += 1;
                            }
                            tokenizer_state = TokenizerState::Outside;
                        }
                        TokenizerState::SingleQuote
                        | TokenizerState::DoubleQuote
                        | TokenizerState::Comment => {}
                        TokenizerState::Outside => {
                            expected_value_cnt += 1;

                            if key.is_none() || values.is_empty() {
                                let e = Box::new(DamoclesError::UnexpectedListSeparator {
                                    sep: c,
                                    pos: self.field_parser.loc(chars.pos()),
                                });
                                self.field_parser.error(e);
                            }
                        }
                    }
                }
                _ => {
                    // If we find a "control" character, we are most probably parsing a bin file, not a text file...
                    // Note however that tab ('\t') is considered a control character...
                    if c.is_control() && !c.is_whitespace() {
                        let e = Box::new(DamoclesError::NonPrintableCharacter {
                            char: c,
                            pos: self.field_parser.loc(chars.pos()),
                        });
                        self.field_parser.error(e);
                    }

                    match tokenizer_state {
                        TokenizerState::PlainToken => {
                            if key.is_none() {
                                // parsing a key. Any characters can be in the key
                                // Key will be ended by a ':' or a '='
                            } else {
                                // PlainToken 'value' are separated by whitespaces
                                if c.is_whitespace() {
                                    token = self.create_token(input);
                                    tokenizer_state = TokenizerState::Outside;
                                }
                            }
                        }
                        TokenizerState::SingleQuote
                        | TokenizerState::DoubleQuote
                        | TokenizerState::Comment => {}
                        TokenizerState::Outside => {
                            if !c.is_whitespace() {
                                self.token_start_offset = i;
                                self.token_start_pos = chars.pos();
                                tokenizer_state = TokenizerState::PlainToken;
                            }
                        }
                    }
                }
            }

            if !c.is_whitespace() {
                self.last_non_whitespace_offset = i + c.len_utf8();
                self.last_non_whitespace_pos = chars.pos();
            }

            // Got a token.
            // it can be a key or a value
            if let Some(unwrapped_token) = token {
                if key.is_none() {
                    key = Some(unwrapped_token);
                } else if find_assignment {
                    let value = unwrapped_token;
                    values.push(value);
                } else {
                    let e = Box::new(DamoclesError::MissingAssignment {
                        pos: unwrapped_token.start_pos.clone(),
                    });
                    self.field_parser.error(e);
                }
                token = None;
            }

            // We have:
            // - a key/value(s) pair,
            // - the expected number of value
            // - started a new token
            // Between the last token (last value) and the new token (which should be a key)
            // there was no evidence that we are waiting for a new value (e.g. we never met a ';' a.k.a
            // list separator). Thus the list of value is finished and we can process them.
            if key.is_some()
                && (values.len() == expected_value_cnt)
                && is_valid_pos(&self.token_start_pos)
            {
                let unwrapped_key = key.unwrap();
                let kpi = KeywordParseInfo {
                    key: unwrapped_key,
                    values,
                };
                self.field_parser.new_field(kpi);

                // Reset values for next key/value(s) pair
                key = None;
                values = Vec::new();
                expected_value_cnt = 1;
                find_assignment = false;
            }
        }

        // Don't forget the last key
        match tokenizer_state {
            TokenizerState::Outside | TokenizerState::Comment => {}
            TokenizerState::PlainToken => {
                token = self.create_token(input);
            }
            TokenizerState::SingleQuote | TokenizerState::DoubleQuote => {
                let quote = if tokenizer_state == TokenizerState::SingleQuote {
                    '\''
                } else {
                    '"'
                };

                let e = Box::new(DamoclesError::MissingEndQuote {
                    quote,
                    pos: self.field_parser.loc(chars.pos()),
                });
                self.field_parser.error(e);
            }
        }

        // Got a final token.
        // it can be a key or a value
        if token.is_some() {
            if key.is_none() {
                key = token.take();
            } else {
                let value = token.take().unwrap();
                values.push(value);
            }
        }

        if let Some(unwrapped_key) = key {
            if values.len() == expected_value_cnt {
                let kpi = KeywordParseInfo {
                    key: unwrapped_key,
                    values,
                };
                self.field_parser.new_field(kpi);
            } else {
                let e = Box::new(DamoclesError::MissingEndValue {
                    key: unwrapped_key.token.to_owned(),
                    pos: unwrapped_key.start_pos,
                });
                self.field_parser.error(e);
            }
        } else {
            // Nothing to proceed.
        }
    }
}

/// Check if a LineCol is valid
fn is_valid_pos(v: &LineCol) -> bool {
    v.0 > 0 && v.1 > 0
}

pub trait DamoclesParser {
    /// Process a new key=value pair
    fn new_field(&mut self, kpi: KeywordParseInfo);

    /// Handle a command (e.g three characters starting with '&')
    fn cmd(&mut self, cmd: &TokenInfo);

    /// Report a parsing error
    fn error(&mut self, e: Box<dyn std::error::Error>);

    /// Compute TextLoc from a {line, col} pair
    fn loc(&self, pos: (usize, usize)) -> TextLoc;

    /// Parse "key = value" pairs from a block, handling multiline values.
    /// A new key starts when a line matches "IDENTIFIER = ...".
    fn parse_fields(&mut self, input: &str)
    where
        Self: Sized,
    {
        let mut ctx = DamoclesParseContext::new(self);
        ctx.parse_fields(input);
    }
}
