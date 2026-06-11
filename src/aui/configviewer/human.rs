//! # Human readable output of a config
//!

use super::ConfigViewer;
use anstyle::{AnsiColor, Effects, Style};
use serde_json::value::Value as JsonValue;
use std::io::Write;

// ---------------------------------------------------------------------------
// Colored output
// ---------------------------------------------------------------------------

/// How to colorize some part of the output
struct Palette {
    pub section: Style, // section header  [title]
    pub key: Style,     // key name in key: value
    //pub value:    Style,   // value text
    pub header: Style, // table column headers
    pub sep: Style,    // table separator line
    pub index: Style,  // index column in tables
}

impl Palette {
    /// Full color palette
    fn colored() -> Self {
        Self {
            section: Style::new()
                .fg_color(Some(AnsiColor::Cyan.into()))
                .effects(Effects::BOLD),
            key: Style::new().fg_color(Some(AnsiColor::Yellow.into())),
            //value:   Style::new(),
            header: Style::new().effects(Effects::BOLD),
            sep: Style::new().fg_color(Some(AnsiColor::BrightBlack.into())),
            index: Style::new().fg_color(Some(AnsiColor::BrightBlack.into())),
        }
    }
}

/// Wrap `text` in an anstyle render/reset pair.
fn paint(style: Style, text: &str) -> String {
    format!("{style}{text}{style:#}")
}

fn render_value(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "None".to_owned(),
        JsonValue::Number(n) => {
            if n.is_f64() {
                let v = n.as_f64().unwrap();
                format!("{v:.6}")
            } else {
                n.to_string()
            }
        }
        JsonValue::Bool(b) => {
            if *b {
                "Yes".to_owned()
            } else {
                "No".to_owned()
            }
        }
        JsonValue::String(s) => s.clone(),
        JsonValue::Array(a) => {
            let vec_str: Vec<String> = a
                .iter()
                .map(|v| format!("\n   - {}", render_value(v)))
                .collect();
            vec_str.join(", ")
        }
        JsonValue::Object(obj) => {
            let mut ret = String::from("\n");
            for (k, v) in obj {
                let entry = format!("   - {}: {}\n", k.as_str(), render_value(v).as_str());
                ret += entry.as_str();
            }
            ret
        }
    }
}

/// Display a config in a human readable output
///
/// Typically on stdout or a terminal
pub struct HumanConfigViewer<W: Write> {
    writer: W,
    palette: Palette,
}

impl<W> ConfigViewer for HumanConfigViewer<W>
where
    W: Write,
{
    fn emit_kv(&mut self, key: &str, value: &JsonValue) {
        let k = paint(self.palette.key, key);
        let v = render_value(value);
        writeln!(self.writer, "{k}: {v}").unwrap();
    }

    fn emit_table(&mut self, key: &str, headers: &[&str], rows: &[Vec<JsonValue>]) {
        let rows_str: Vec<Vec<String>> = rows
            .iter()
            .map(|vv| vv.iter().map(render_value).collect())
            .collect();
        let k = paint(self.palette.key, key);

        writeln!(self.writer, "{k}:").unwrap();
        // Compute column widths
        let column_cnt = headers.len();
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
        let painted_header_line = paint(self.palette.header, header_line.as_str());
        let sep: String = widths
            .iter()
            .map(|&w| "-".repeat(w))
            .collect::<Vec<_>>()
            .join("  ");
        let painted_sep = paint(self.palette.sep, sep.as_str());
        writeln!(self.writer, "    {painted_header_line}").unwrap();
        writeln!(self.writer, "    {painted_sep}").unwrap();
        for row in rows_str {
            let line: String = row
                .iter()
                .enumerate()
                .take(column_cnt)
                .map(|(i, c)| {
                    let value = format!("{:width$}", c, width = widths[i]);
                    if i == 0 {
                        paint(self.palette.index, value.as_str())
                    } else {
                        value
                    }
                })
                .collect::<Vec<_>>()
                .join("  ");
            writeln!(self.writer, "    {line}").unwrap();
        }
    }
    fn emit_section_start(&mut self, name: &str) {
        let heading = paint(self.palette.section, &format!("[{name}]"));
        writeln!(self.writer, "\n{heading}").unwrap();
    }

    fn emit_section_end(&mut self) {}

    fn emit_comment(&mut self, _comment: &str) {}

    fn finish(&mut self) {}
}

impl<W> HumanConfigViewer<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            palette: Palette::colored(),
        }
    }
}
