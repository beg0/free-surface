// #! Render text parsing diagnostics in JSON format

use std::collections::HashMap;
use std::io::Write;

use super::super::super::collector::Severity;
use super::super::super::collector::TextParserDiagnostics;

use super::TextDiagnosticsRenderer;

use serde::Serialize;
use serde_json;

/// Renderer for text parsing diagnostics in JSON format
pub struct JsonTextDiagnosticsRenderer<W: Write> {
    writer: W,
    pretty: bool,
}

impl<W: Write> TextDiagnosticsRenderer for JsonTextDiagnosticsRenderer<W> {
    /// Render Text Parser Diagnostics in a machine-readable format
    fn render(&mut self, diagnostics: &TextParserDiagnostics) -> Result<(), std::io::Error> {
        let all_diags: Vec<serde_json::Value> = diagnostics
            .all()
            .iter()
            .map(|d| {
                let kind = match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Hint => "hint",
                };
                let mut location: HashMap<&str, serde_json::Value> = HashMap::new();
                location.insert("filename", serde_json::to_value(d.loc.filename())?);
                location.insert("line", serde_json::to_value(d.loc.line())?);
                location.insert("column", serde_json::to_value(d.loc.column())?);

                let locations = vec![location];

                let mut root: HashMap<&str, serde_json::Value> = HashMap::new();

                root.insert("message", serde_json::to_value(d.message.clone())?);
                root.insert("kind", serde_json::to_value(kind)?);

                root.insert("locations", serde_json::to_value(locations)?);

                serde_json::to_value(root)
            })
            .collect::<Result<_, _>>()?;

        if self.pretty {
            let mut ser = serde_json::Serializer::pretty(&mut self.writer);
            all_diags.serialize(&mut ser)?;
        } else {
            let mut ser = serde_json::Serializer::new(&mut self.writer);
            all_diags.serialize(&mut ser)?;
        }

        Ok(())
    }
}

impl<W: Write> JsonTextDiagnosticsRenderer<W> {
    pub fn new(w: W, pretty: bool) -> Self {
        Self { writer: w, pretty }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aui::diagnostic::collector::TextParserDiagnostics;
    use crate::config::textloc::TextLoc;
    use serde_json::json;

    const ERROR_MSG: &str = "you made a mistake!";
    const FILE_WITH_ERROR: &str = "corrupted_file.txt";
    const ERROR_LINE: usize = 666;

    const WARNING_MSG: &str = "Be careful!";
    const FILE_WITH_WARNING: &str = "strange_file.txt";
    const WARNING_LINE: usize = 42;

    const HINT_MSG: &str = "Try that!";
    const FILE_WITH_HINT: &str = "no_clue_file.txt";
    const HINT_LINE: usize = 777;

    fn sample_diagnostics() -> TextParserDiagnostics {
        let mut diagnostics = TextParserDiagnostics::default();
        diagnostics.error(ERROR_MSG, TextLoc::from((FILE_WITH_ERROR, ERROR_LINE)));
        diagnostics.warning(
            WARNING_MSG,
            TextLoc::from((FILE_WITH_WARNING, WARNING_LINE)),
        );
        diagnostics.hint(HINT_MSG, TextLoc::from((FILE_WITH_HINT, HINT_LINE)));
        diagnostics
    }

    /// Build the expected JSON value from the same `TextLoc` construction
    /// used above, rather than hardcoding a `column` value we don't own.
    fn expected_value() -> serde_json::Value {
        let error_loc = TextLoc::from((FILE_WITH_ERROR, ERROR_LINE));
        let warning_loc = TextLoc::from((FILE_WITH_WARNING, WARNING_LINE));
        let hint_loc = TextLoc::from((FILE_WITH_HINT, HINT_LINE));

        json!([
            {
                "message": ERROR_MSG,
                "kind": "error",
                "locations": [
                    {
                        "filename": error_loc.filename(),
                        "line": error_loc.line(),
                        "column": error_loc.column(),
                    }
                ]
            },
            {
                "message": WARNING_MSG,
                "kind": "warning",
                "locations": [
                    {
                        "filename": warning_loc.filename(),
                        "line": warning_loc.line(),
                        "column": warning_loc.column(),
                    }
                ]
            },
            {
                "message": HINT_MSG,
                "kind": "hint",
                "locations": [
                    {
                        "filename": hint_loc.filename(),
                        "line": hint_loc.line(),
                        "column": hint_loc.column(),
                    }
                ]
            }
        ])
    }

    #[test]
    fn compact_output_matches_expected_structure() {
        let mut buf = Vec::with_capacity(256);
        let mut render = JsonTextDiagnosticsRenderer::new(&mut buf, false);

        let diagnostics = sample_diagnostics();
        render
            .render(&diagnostics)
            .expect("can't render diagnostics");

        #[allow(clippy::drop_non_drop)]
        drop(render);

        let printed = String::from_utf8(buf).expect("Render generated non-utf8 char");

        // Compact mode: single line, no indentation.
        assert!(!printed.contains('\n'));

        let parsed: serde_json::Value =
            serde_json::from_str(&printed).expect("renderer did not produce valid JSON");

        assert_eq!(parsed, expected_value());
    }

    #[test]
    fn pretty_output_matches_expected_structure() {
        let mut buf = Vec::with_capacity(256);
        let mut render = JsonTextDiagnosticsRenderer::new(&mut buf, true);

        let diagnostics = sample_diagnostics();
        render
            .render(&diagnostics)
            .expect("can't render diagnostics");

        #[allow(clippy::drop_non_drop)]
        drop(render);

        let printed = String::from_utf8(buf).expect("Render generated non-utf8 char");

        // Pretty mode: expect indentation/newlines, but same logical content.
        assert!(printed.contains('\n'));

        let parsed: serde_json::Value =
            serde_json::from_str(&printed).expect("renderer did not produce valid JSON");

        assert_eq!(parsed, expected_value());
    }

    #[test]
    fn no_diagnostics_renders_empty_array() {
        let mut buf = Vec::new();
        let mut render = JsonTextDiagnosticsRenderer::new(&mut buf, false);

        let diagnostics = TextParserDiagnostics::default();
        render
            .render(&diagnostics)
            .expect("can't render diagnostics");

        #[allow(clippy::drop_non_drop)]
        drop(render);

        let printed = String::from_utf8(buf).expect("Render generated non-utf8 char");
        let parsed: serde_json::Value =
            serde_json::from_str(&printed).expect("renderer did not produce valid JSON");

        assert_eq!(parsed, json!([]));
    }
}
