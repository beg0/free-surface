//! # Display a config-like content
//!
//! e.g. display anything with key=value pairs, optionally organized in sections

use anstream::stream::{AsLockedWrite, RawStream};
use anstream::AutoStream;
use clap::ColorChoice;
use serde_json::value::Value as JsonValue;

mod damocles;
mod human;
mod json;
mod porcelain;

pub use damocles::DamoclesConfigViewer;
pub use human::HumanConfigViewer;
pub use json::JsonConfigViewer;
pub use porcelain::PorcelainConfigViewer;

pub trait ConfigViewer {
    /// Display a (key,value) pair
    fn emit_kv(&mut self, key: &str, value: &JsonValue);

    /// Display a list of object of the same type
    fn emit_table(&mut self, key: &str, headers: &[&str], rows: &[Vec<JsonValue>]);

    /// Display a start of section
    fn emit_section_start(&mut self, name: &str);

    /// Display a end of section
    fn emit_section_end(&mut self) {}

    /// Display a comment (if supported)
    fn emit_comment(&mut self, _comment: &str) {}

    /// Terminate the display
    fn finish(&mut self) {}
}

pub enum ConfigViewerOptions {
    Damocles,
    Human { color: ColorChoice },
    Json { pretty: bool },
    Machine,
}

/// Factory to create one of the supported format
pub fn create_config_viewer<W: RawStream + AsLockedWrite + 'static>(
    out: W,
    options: ConfigViewerOptions,
) -> Box<dyn ConfigViewer> {
    // Dispatch to the right rendrer
    match options {
        ConfigViewerOptions::Damocles => Box::new(DamoclesConfigViewer::new(out)),
        ConfigViewerOptions::Human { color } => {
            // AutoStream strips ANSI codes automatically when the output is
            // not a TTY or when NO_COLOR / --no-color is set.
            let stream = match color {
                ColorChoice::Always => AutoStream::always(out),
                ColorChoice::Never => AutoStream::never(out),
                ColorChoice::Auto => AutoStream::auto(out),
            };

            Box::new(HumanConfigViewer::new(stream))
        }
        ConfigViewerOptions::Json { pretty } => Box::new(JsonConfigViewer::new(out, pretty)),
        ConfigViewerOptions::Machine => Box::new(PorcelainConfigViewer::new(out)),
    }
}
