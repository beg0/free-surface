//! # Telemac steering files format
//!
use super::ConfigViewer;
use crate::config::parse_helpers::{single_quote_if_needed, write_fortran_float};
use serde_json::value::Value as JsonValue;
use std::io::Write;

pub struct DamoclesConfigViewer<W: Write> {
    writer: W,
    section_level: usize,

    /// Was the last "emit" a call to "emit_kv"
    /// If yes, and next call is a comment of a start of section, then add an extra blank line
    last_is_kv: bool,
}

/// Convert a Json value to a damocles-compatible output
fn to_damocles_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Array(items) => {
            let values: Vec<String> = items.iter().map(to_damocles_string).collect();
            values.join(";")
        }
        JsonValue::Object(obj) => {
            // Consider object is a list of "key" ":" "value"
            let values: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}:{}", k, to_damocles_string(v)))
                .collect();
            values.join(";")
        }
        JsonValue::Bool(val) => {
            if *val {
                "Yes".to_owned()
            } else {
                "No".to_owned()
            }
        }
        JsonValue::String(val) => single_quote_if_needed(val.as_str()),
        JsonValue::Null => String::from("''"), // I don't really know how to handle Null here, thus I put an empty string
        JsonValue::Number(val) => {
            if let Some(val_f64) = val.as_f64() {
                write_fortran_float(val_f64)
            } else {
                val.to_string()
            }
        }
    }
}

impl<W: Write> ConfigViewer for DamoclesConfigViewer<W> {
    fn emit_kv(&mut self, key: &str, value: &JsonValue) {
        self.last_is_kv = true;
        writeln!(self.writer, "{} = {}", key, to_damocles_string(value)).unwrap()
    }
    fn emit_table(&mut self, key: &str, headers: &[&str], rows: &[Vec<JsonValue>]) {
        let prefix_len = key.len() + 3; // +3 because of " = " after the key,
        let padding: String = " ".repeat(prefix_len);
        let column_cnt = headers.len();

        self.last_is_kv = true;

        let rows_str: Vec<Vec<String>> = rows
            .iter()
            .map(|vv| {
                vv.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let mut stringified = to_damocles_string(v);
                        if i < (column_cnt - 1) {
                            stringified.push(',');
                        }
                        stringified
                    })
                    .collect()
            })
            .collect();

        // Compute column widths
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in &rows_str {
            for (i, cell) in row.iter().enumerate().take(column_cnt) {
                widths[i] = widths[i].max(cell.len());
            }
        }
        let header_line: String = headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{:width$}", h, width = widths[i]))
            .collect::<Vec<_>>()
            .join("  ");

        writeln!(self.writer, "/{padding}{header_line}").unwrap();

        for (i, row) in rows_str.iter().enumerate() {
            let line: String = row
                .iter()
                .enumerate()
                .take(column_cnt)
                .map(|(i, c)| format!("{:<width$}", c, width = widths[i]))
                .collect::<Vec<_>>()
                .join("  ");
            if i == 0 {
                write!(self.writer, "{} = \"{}\"", key, line).unwrap();
            } else {
                write!(self.writer, "{padding}\"{line}\"").unwrap();
            }
            if i < rows_str.len() - 1 {
                writeln!(self.writer, ";").unwrap();
            } else {
                writeln!(self.writer).unwrap();
            }
        }
    }

    fn emit_section_start(&mut self, name: &str) {
        self.extra_blank_line().unwrap();
        self.section_level += 1;

        let upper_name = name.to_uppercase();

        // Usually steering files has no more than 3 levels of section
        // Thus just consider the first 3 levels
        match self.section_level {
            1 => {
                writeln!(
                    self.writer,
                    "/----------------------------------------------------------------------/"
                )
                .unwrap();
                writeln!(self.writer, "/{:^70}/", upper_name).unwrap();
                writeln!(
                    self.writer,
                    "/----------------------------------------------------------------------/"
                )
                .unwrap();
                writeln!(self.writer).unwrap();
            }
            2 => {
                let name_len = upper_name.len();
                writeln!(self.writer, "/ {}", upper_name).unwrap();
                writeln!(self.writer, "/{}", "-".repeat(name_len + 1)).unwrap();
                writeln!(self.writer).unwrap();
            }
            _ => {
                writeln!(self.writer, "/ {}", upper_name).unwrap();
                writeln!(self.writer).unwrap();
            }
        }
    }

    fn emit_section_end(&mut self) {
        assert!(self.section_level > 0); // We shall have as many emit_section_start() as emit_section_end()
        self.section_level -= 1;
    }
    fn emit_comment(&mut self, comment: &str) {
        self.extra_blank_line().unwrap();
        for line in comment.split("\n") {
            writeln!(self.writer, "/ {}", line).unwrap();
        }
    }
    fn finish(&mut self) {
        assert!(self.section_level == 0); // We shall have as many emit_section_start() as emit_section_end()
    }
}

impl<W: Write> DamoclesConfigViewer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            section_level: 0,
            last_is_kv: false,
        }
    }

    fn extra_blank_line(&mut self) -> std::io::Result<usize> {
        if self.last_is_kv {
            self.last_is_kv = false;
            self.writer.write(b"\n")
        } else {
            Ok(0)
        }
    }
}
