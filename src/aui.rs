//! # Abstract User Interface
//!
//! Render things to the user, without knowing how.
use clap::ValueEnum;

pub mod configviewer;

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
