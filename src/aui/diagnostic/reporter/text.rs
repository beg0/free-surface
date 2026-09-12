/// # Parser error reporting mechanism
///
/// This module offers a way to create a reporting function for [TextParserDiagnostics]
///
use anstream::stream::{AsLockedWrite, RawStream};
use clap::ColorChoice;

use super::super::collector::TextParserDiagnostics;

use crate::aui::helpers::color_choice_to_stream;

mod json;
mod terminal;

pub trait TextDiagnosticsRenderer {
    /// Display [TextParserDiagnostics] to outside world
    fn render(&mut self, diagnostics: &TextParserDiagnostics) -> Result<(), std::io::Error>;
}

/// Options to create a [TextDiagnosticsRenderer] via [text_diagnostic_renderer]
pub enum TextDiagnosticsRendererOptions {
    Terminal { color: ColorChoice },
    Json { pretty: bool },
}

/// Factory to create one of the supported format
///
/// # Example
/// ```
/// use free_surface::config::textloc::TextLoc;
/// use free_surface::aui::diagnostic::collector::TextParserDiagnostics;
/// use free_surface::aui::diagnostic::reporter::{TextDiagnosticsRendererOptions, create_text_diagnostic_renderer};
/// use clap::ColorChoice;
///
/// let mut stdio = std::io::stdout();
/// let options =  TextDiagnosticsRendererOptions::Terminal { color: ColorChoice::Auto };
/// let mut renderer = create_text_diagnostic_renderer(stdio, options);
/// let diagnostics = TextParserDiagnostics::from_single_error("This is an error", TextLoc::default());
/// renderer.as_mut().render(&diagnostics).expect("writing error")
/// ```
pub fn create_text_diagnostic_renderer<'a, W>(
    out: W,
    options: TextDiagnosticsRendererOptions,
) -> Box<dyn TextDiagnosticsRenderer + 'a>
where
    W: RawStream + AsLockedWrite + 'a,
{
    match options {
        TextDiagnosticsRendererOptions::Terminal { color } => {
            let stream = color_choice_to_stream(out, color);
            Box::new(terminal::TerminalTextDiagnosticsRenderer::new(stream))
        }
        TextDiagnosticsRendererOptions::Json { pretty } => {
            Box::new(json::JsonTextDiagnosticsRenderer::new(out, pretty))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aui::diagnostic::collector::TextParserDiagnostics;
    use crate::config::textloc::TextLoc;
    use serde_json::json;

    const ERROR_MSG: &str = "this is an error msg";
    const BUGGY_FILE: &str = "corrupted_file.txt";
    const BUGGY_LINE: usize = 42;

    fn get_rendered_diagnostics(options: TextDiagnosticsRendererOptions) -> String {
        let mut buf = Vec::with_capacity(128);
        let mut renderer = create_text_diagnostic_renderer(&mut buf, options);

        let diagnostics = TextParserDiagnostics::from_single_error(
            ERROR_MSG,
            TextLoc::from((BUGGY_FILE, BUGGY_LINE)),
        );

        renderer
            .as_mut()
            .render(&diagnostics)
            .expect("no rendering error");

        drop(renderer);

        String::from_utf8(buf).expect("invalid utf8 from renderer")
    }

    #[test]
    fn terminal_color_always() {
        let printed = get_rendered_diagnostics(TextDiagnosticsRendererOptions::Terminal {
            color: ColorChoice::Always,
        });

        assert_eq!(
            printed,
            format!(
                "\x1b[1m{}:{}:\x1b[0m \x1b[1m\x1b[31merror:\x1b[0m {}\n",
                BUGGY_FILE, BUGGY_LINE, ERROR_MSG
            )
        );
    }

    #[test]
    fn terminal_color_never() {
        let printed = get_rendered_diagnostics(TextDiagnosticsRendererOptions::Terminal {
            color: ColorChoice::Never,
        });

        assert_eq!(
            printed,
            format!("{}:{}: error: {}\n", BUGGY_FILE, BUGGY_LINE, ERROR_MSG)
        );
    }

    #[test]
    fn terminal_color_auto() {
        let printed = get_rendered_diagnostics(TextDiagnosticsRendererOptions::Terminal {
            color: ColorChoice::Auto,
        });

        assert_eq!(
            printed,
            format!("{}:{}: error: {}\n", BUGGY_FILE, BUGGY_LINE, ERROR_MSG)
        );
    }

    #[test]
    fn json_compact() {
        let printed =
            get_rendered_diagnostics(TextDiagnosticsRendererOptions::Json { pretty: false });

        let expected = json!([
        {
            "message": ERROR_MSG,
            "kind": "error",
            "locations": [
                {
                    "filename": BUGGY_FILE,
                    "line": BUGGY_LINE,
                    "column": 0,
                }
            ]
        }]);

        let parsed: serde_json::Value =
            serde_json::from_str(&printed).expect("renderer did not produce valid JSON");

        assert_eq!(parsed, expected);
    }
}
