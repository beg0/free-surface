//! # machine-parsable output of a config
//!
use super::ConfigViewer;
use serde_json::value::Value as JsonValue;
use std::io::Write;

pub struct PorcelainConfigViewer<W: Write> {
    writer: W,
    sections_title: Vec<String>,
}

/// Return a name prefixed with every section names, separated by a dot
fn fully_qualified_name(sections_title: &[String], name: &str) -> String {
    if sections_title.is_empty() {
        name.to_owned()
    } else {
        let prefix = sections_title.join(".");
        format!("{prefix}.{name}")
    }
}

impl<W: Write> ConfigViewer for PorcelainConfigViewer<W> {
    fn emit_kv(&mut self, key: &str, value: &JsonValue) {
        let fqn = fully_qualified_name(&self.sections_title, key);
        match value {
            JsonValue::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    writeln!(self.writer, "{fqn}[{i}]={item}").unwrap();
                }
            }
            JsonValue::Object(obj) => {
                for (subkey, item) in obj.iter() {
                    writeln!(self.writer, "{fqn}.{subkey}={item}").unwrap();
                }
            }
            _ => writeln!(self.writer, "{fqn}={value}").unwrap(),
        }
    }
    fn emit_table(&mut self, key: &str, headers: &[&str], rows: &[Vec<JsonValue>]) {
        let fqn = fully_qualified_name(&self.sections_title, key);

        for (i, row) in rows.iter().enumerate() {
            for (j, cell) in row.iter().enumerate().take(headers.len()) {
                writeln!(self.writer, "{fqn}[{i}].{}={cell}", headers[j]).unwrap();
            }
        }
    }

    fn emit_section_start(&mut self, name: &str) {
        self.sections_title.push(name.to_owned());
    }

    fn emit_section_end(&mut self) {
        self.sections_title.pop();
    }
    fn emit_comment(&mut self, comment: &str) {
        writeln!(self.writer, "# {}", comment).unwrap()
    }
    fn finish(&mut self) {}
}

impl<W: Write> PorcelainConfigViewer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            sections_title: Vec::new(),
        }
    }
}
