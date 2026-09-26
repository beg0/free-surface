//! # Parsing error collection
//!

mod text;

pub use text::{TextParserDiagnostic, TextParserDiagnostics};

/// Diagnostic severity
#[derive(Clone, PartialEq, Debug)]
pub enum Severity {
    /// An error that the system cannot recover from
    Error,

    /// A warning can be two different things:
    /// 1. Something that the system may have interpreted in some way
    ///    and that the user may not expect
    /// 2. or an error that the system has recover in a certain way but
    ///    which may lead to something unexpected for the user
    Warning,

    /// Something that can help to resolve an error or a warning
    Hint,
}
