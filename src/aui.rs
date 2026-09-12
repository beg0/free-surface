//! # Abstract User Interface
//!
//! Render things to the user, without knowing how.
use clap::ValueEnum;

pub mod configviewer;
pub mod diagnostic;
mod helpers;

/// How to render output values
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Format {
    /// Telemac steering files format (e.g. cas files)
    Damocles,
    /// Readable prose output
    Human,
    /// JSON (one object per section)
    Json,
    /// Machine-parsable key=value lines
    #[value(alias("porcelain"), hide = false)]
    Machine,
}

/// Format for outputing diagnostics
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum DiagFormat {
    /// Console output format
    Terminal,
    /// JSON (one object per file)
    Json,
}
