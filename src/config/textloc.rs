//! # Text localisation
//!
//! TextLoc stores a position in a text file
//!

use std::convert::From;
use std::fmt;

pub const UNKNOWN_FILE: &str = "<unknown>";

/// A localisation in a text file
#[derive(Clone, Debug, PartialEq)]
pub struct TextLoc {
    filename: String, // Filename if any, an empty string otherwise
    line: usize,      // Line number in the file, starting to 1
    column: usize,    // Column number in the line, starting to 1. 0 if not set
}

// Impl From with filename
//------------------------

impl From<(&str, usize)> for TextLoc {
    fn from((filename, line): (&str, usize)) -> Self {
        Self {
            filename: String::from(filename),
            line,
            column: 0,
        }
    }
}

impl From<(String, usize)> for TextLoc {
    fn from((filename, line): (String, usize)) -> Self {
        Self {
            filename,
            line,
            column: 0,
        }
    }
}

impl From<(String, usize, usize)> for TextLoc {
    fn from((filename, line, column): (String, usize, usize)) -> Self {
        Self {
            filename,
            line,
            column,
        }
    }
}

// Impl From without filename
//---------------------------

impl From<usize> for TextLoc {
    fn from(line: usize) -> Self {
        Self {
            filename: String::new(),
            line,
            column: 0,
        }
    }
}

impl From<(usize, usize)> for TextLoc {
    fn from((line, column): (usize, usize)) -> Self {
        Self {
            filename: String::new(),
            line,
            column,
        }
    }
}

// fmt::Display

impl fmt::Display for TextLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filename = if self.filename.is_empty() {
            UNKNOWN_FILE
        } else {
            self.filename.as_str()
        };

        if self.column == 0 {
            write!(f, "{}:{}", filename, self.line)
        } else {
            write!(f, "{}:{}:{}", filename, self.line, self.column)
        }
    }
}

impl TextLoc {
    /// Clone this TextLoc and modify the line number
    #[allow(dead_code)]
    pub fn clone_with_line(&self, line: usize) -> TextLoc {
        TextLoc {
            filename: self.filename.clone(),
            line,
            column: 0,
        }
    }
}
