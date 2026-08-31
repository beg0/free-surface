//! # Parsing error collection
//!

mod text;

pub use text::{TextParserDiagnostic, TextParserDiagnostics};

/// Diagnostic severity
#[derive(Clone, PartialEq, Debug)]
pub enum Severity {
    Error,
    Warning,
    Hint,
}
