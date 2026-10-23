// #! Render text parsing diagnostics to a terminal

use std::io::Write;

use super::super::collector::Severity;
use super::super::collector::{DiagnosticLocation, Diagnostics};
use crate::t;
use anstyle::{AnsiColor, Effects, Style};

use super::DiagnosticsRenderer;

/// How to colorize some part of the output
struct Palette {
    pub textloc: Style,
    pub error: Style,
    pub warning: Style,
    pub hint: Style,
}

impl Palette {
    /// Full color palette
    fn colored() -> Self {
        Self {
            textloc: Style::new().effects(Effects::BOLD),
            error: Style::new()
                .fg_color(Some(AnsiColor::Red.into()))
                .effects(Effects::BOLD),
            warning: Style::new()
                .fg_color(Some(AnsiColor::Magenta.into()))
                .effects(Effects::BOLD),
            hint: Style::new()
                .fg_color(Some(AnsiColor::Cyan.into()))
                .effects(Effects::BOLD),
        }
    }
}

/// Render Text Parser Diagnostics to a terminal
///
/// The output format is similar to what GCC diagnostics
/// may output.
/// This is both human-readable & machine readable.
///
/// It output colorized text. Use [anstream::AutoStream]
/// to filter colors if needed.
///
pub struct TerminalDiagnosticsRenderer<W: Write> {
    writer: W,
}

impl<W: Write> DiagnosticsRenderer for TerminalDiagnosticsRenderer<W> {
    fn render(&mut self, diagnostics: &Diagnostics) -> Result<(), std::io::Error> {
        let palette = Palette::colored();

        for d in diagnostics.all() {
            match d.loc {
                DiagnosticLocation::Filename(_) | DiagnosticLocation::TextLoc(_) => {
                    palette.textloc.write_to(&mut self.writer)?;
                    write!(self.writer, "{}:", d.loc)?;
                    palette.textloc.write_reset_to(&mut self.writer)?;
                }
                DiagnosticLocation::None => {}
            }
            write!(self.writer, " ")?;

            self.render_severity(&d.severity, &palette)?;

            writeln!(self.writer, " {}", d.message)?;
        }

        Ok(())
    }
}

impl<W: Write> TerminalDiagnosticsRenderer<W> {
    pub fn new(w: W) -> Self {
        Self { writer: w }
    }

    fn render_severity(
        &mut self,
        severity: &Severity,
        palette: &Palette,
    ) -> Result<(), std::io::Error> {
        match severity {
            Severity::Error => {
                palette.error.write_to(&mut self.writer)?;
                write!(self.writer, "{}:", t!("diag-label-error"))?;
                palette.error.write_reset_to(&mut self.writer)?;
            }
            Severity::Warning => {
                palette.warning.write_to(&mut self.writer)?;
                write!(self.writer, "{}:", t!("diag-label-warning"))?;
                palette.warning.write_reset_to(&mut self.writer)?;
            }
            Severity::Hint => {
                palette.hint.write_to(&mut self.writer)?;
                write!(self.writer, "{}:", t!("diag-label-hint"))?;
                palette.hint.write_reset_to(&mut self.writer)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aui::diagnostic::collector::Diagnostics;
    use crate::config::textloc::TextLoc;

    const FILE_WITH_ERROR: &str = "corrupted_file.txt";
    const ERROR_LINE: usize = 666;

    const FILE_WITH_WARNING: &str = "strange_file.txt";
    const WARNING_LINE: usize = 42;

    const FILE_WITH_HINT: &str = "no_clue_file.txt";
    const HINT_LINE: usize = 777;

    #[derive(fluent_message::FluentMessage)]
    #[fluent(id = "test-error-message")]
    struct TestErrorMessage();

    #[derive(fluent_message::FluentMessage)]
    #[fluent(id = "test-warning-message")]
    struct TestWarningMessage();

    #[derive(fluent_message::FluentMessage)]
    #[fluent(id = "test-hint-message")]
    struct TestHintMessage();

    fn sample_diagnostics() -> Diagnostics {
        let mut diagnostics = Diagnostics::default();
        diagnostics
            .error(TestErrorMessage())
            .with_textloc(TextLoc::from((FILE_WITH_ERROR, ERROR_LINE)));
        diagnostics
            .warning(TestWarningMessage())
            .with_textloc(TextLoc::from((FILE_WITH_WARNING, WARNING_LINE)));
        diagnostics
            .hint(TestHintMessage())
            .with_textloc(TextLoc::from((FILE_WITH_HINT, HINT_LINE)));
        diagnostics
    }

    #[test]
    fn basic() {
        let mut buf = Vec::with_capacity(128);

        let mut render = TerminalDiagnosticsRenderer::new(&mut buf);

        let diagnostics = sample_diagnostics();
        render.render(&diagnostics).expect("can't render msg");

        #[allow(clippy::drop_non_drop)]
        drop(render);

        let printed = String::from_utf8(buf).expect("Render generated non-utf8 char");

        let lines: Vec<&str> = printed.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with(
            format!(
                "\x1b[1m{}:{}:\x1b[0m \x1b[1m\x1b[31m{}:\x1b[0m ",
                FILE_WITH_ERROR, ERROR_LINE, "error"
            )
            .as_str()
        ));
        assert!(lines[0].contains("test-error-message"));
        assert!(lines[1].starts_with(
            format!(
                "\x1b[1m{}:{}:\x1b[0m \x1b[1m\x1b[35m{}:\x1b[0m ",
                FILE_WITH_WARNING, WARNING_LINE, "warning"
            )
            .as_str()
        ));
        assert!(lines[1].contains("test-warning-message"));
        assert!(lines[2].starts_with(
            format!(
                "\x1b[1m{}:{}:\x1b[0m \x1b[1m\x1b[36m{}:\x1b[0m ",
                FILE_WITH_HINT, HINT_LINE, "hint"
            )
            .as_str()
        ));
        assert!(lines[2].contains("test-hint-message"));
    }

    #[test]
    fn no_diagnostics_renders_empty_string() {
        let mut buf = Vec::new();
        let mut render = TerminalDiagnosticsRenderer::new(&mut buf);

        let diagnostics = Diagnostics::default();
        render
            .render(&diagnostics)
            .expect("can't render diagnostics");

        #[allow(clippy::drop_non_drop)]
        drop(render);

        let printed = String::from_utf8(buf).expect("Render generated non-utf8 char");
        assert!(printed.is_empty());
    }
}
