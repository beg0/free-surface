//! # Display a config-like content
//!
//! e.g. display anything with key=value pairs, optionally organized in sections

use serde_json::value::Value as JsonValue;

mod human;
mod json;
mod porcelain;

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
