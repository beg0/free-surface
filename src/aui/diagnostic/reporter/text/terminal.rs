// #! Render text parsing diagnostics to a terminal

use std::io::Write;

use super::super::super::collector::Severity;
use super::super::super::collector::TextParserDiagnostics;
use anstyle::{AnsiColor, Effects, Style};

use super::TextDiagnosticsRenderer;

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
pub struct TerminalTextDiagnosticsRenderer<W: Write> {
    writer: W,
}

impl<W: Write> TextDiagnosticsRenderer for TerminalTextDiagnosticsRenderer<W> {
    fn render(&mut self, diagnostics: &TextParserDiagnostics) -> Result<(), std::io::Error> {
        let palette = Palette::colored();

        for d in diagnostics.all() {
            palette.textloc.write_to(&mut self.writer)?;
            write!(self.writer, "{}:", d.loc)?;
            palette.textloc.write_reset_to(&mut self.writer)?;
            write!(self.writer, " ")?;

            self.render_severity(&d.severity, &palette)?;

            writeln!(self.writer, " {}", d.message)?;
        }

        Ok(())
    }
}

impl<W: Write> TerminalTextDiagnosticsRenderer<W> {
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
                write!(self.writer, "error:")?;
                palette.error.write_reset_to(&mut self.writer)?;
            }
            Severity::Warning => {
                palette.warning.write_to(&mut self.writer)?;
                write!(self.writer, "warning:")?;
                palette.warning.write_reset_to(&mut self.writer)?;
            }
            Severity::Hint => {
                palette.hint.write_to(&mut self.writer)?;
                write!(self.writer, "hint:")?;
                palette.hint.write_reset_to(&mut self.writer)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aui::diagnostic::collector::TextParserDiagnostics;
    use crate::config::textloc::TextLoc;

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

    #[test]
    fn basic() {
        let mut buf = Vec::with_capacity(128);

        let mut render = TerminalTextDiagnosticsRenderer::new(&mut buf);

        let diagnostics = sample_diagnostics();
        render.render(&diagnostics).expect("can't render msg");

        #[allow(clippy::drop_non_drop)]
        drop(render);

        let printed = String::from_utf8(buf).expect("Render generated non-utf8 char");

        let lines: Vec<&str> = printed.lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[0],
            format!(
                "\x1b[1m{}:{}:\x1b[0m \x1b[1m\x1b[31m{}:\x1b[0m {}",
                FILE_WITH_ERROR, ERROR_LINE, "error", ERROR_MSG
            )
        );
        assert_eq!(
            lines[1],
            format!(
                "\x1b[1m{}:{}:\x1b[0m \x1b[1m\x1b[35m{}:\x1b[0m {}",
                FILE_WITH_WARNING, WARNING_LINE, "warning", WARNING_MSG
            )
        );
        assert_eq!(
            lines[2],
            format!(
                "\x1b[1m{}:{}:\x1b[0m \x1b[1m\x1b[36m{}:\x1b[0m {}",
                FILE_WITH_HINT, HINT_LINE, "hint", HINT_MSG
            )
        );
    }

    #[test]
    fn no_diagnostics_renders_empty_string() {
        let mut buf = Vec::new();
        let mut render = TerminalTextDiagnosticsRenderer::new(&mut buf);

        let diagnostics = TextParserDiagnostics::default();
        render
            .render(&diagnostics)
            .expect("can't render diagnostics");

        #[allow(clippy::drop_non_drop)]
        drop(render);

        let printed = String::from_utf8(buf).expect("Render generated non-utf8 char");
        assert!(printed.is_empty());
    }
}
