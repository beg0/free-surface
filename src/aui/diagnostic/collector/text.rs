//! # Collect text parsing errors
//!

use fluent_message::FluentMessage;

use super::Severity;
use crate::config::textloc::TextLoc;

/// A single parsing diagnostic on a Text-based document.
///
/// A diagnostic carries a [`Severity`], a human-readable `message`
/// describing the problem, and the [`TextLoc`] pointing to where in the
/// source document the problem was found.
///
/// Collected through [`TextParserDiagnostics`].
#[derive(Debug)]
pub struct TextParserDiagnostic {
    /// How severe the diagnostic is (error, warning or hint).
    pub severity: Severity,

    /// Human-readable description of the problem.
    pub message: String,

    /// Location in the source document this diagnostic refers to.
    pub loc: TextLoc,
}

/// A diagnostic collector for Text-based document parsing
///
/// Diagnostics are appended via [`error`](Self::error),
/// [`warning`](Self::warning) and [`hint`](Self::hint) while parsing
/// proceeds. The collector keeps a running count of errors and warnings
/// so that callers can cheaply check whether parsing failed with
/// [`has_errors`](Self::has_errors), without having to walk the full list
/// of diagnostics.
#[derive(Default, Debug)]
pub struct TextParserDiagnostics {
    /// All diagnostics reported so far, in report order.
    items: Vec<TextParserDiagnostic>,

    /// Number of error encountered.
    error_count: usize,

    /// Number of warning encountered.
    warn_count: usize,
}

impl TextParserDiagnostics {
    /// Report an error at the given location.
    ///
    /// An error is to be emitted when the caller encounter an error or an
    /// unexpected situation that it can't deal with
    /// Increments the internal error count, which affects the value
    /// returned by [`error_count`](Self::error_count) and
    /// [`has_errors`](Self::has_errors).
    ///
    /// # Parameters
    /// - `message`: description of the error, accepted as anything
    ///   convertible into a `String` (e.g. `&str` or `String`).
    /// - `loc`: location in the document where the error was found.
    pub fn error(&mut self, message: impl FluentMessage, loc: TextLoc) {
        self.error_count += 1;
        self.items.push(TextParserDiagnostic {
            severity: Severity::Error,
            message: fluent_message_to_string(message),
            loc,
        });
    }

    /// Report a warning at the given location.
    ///
    /// A warning is to be emitted when the caller encounter an error or an
    /// unexpected situation but that it successfully find a solution. However
    /// given that the solution may not suite what the user wanted, a warning
    /// have to be emitted.
    ///
    /// Warnings only count towards [`error_count`](Self::error_count) /
    /// [`has_errors`](Self::has_errors) when their `warn_as_error` parameter
    /// is `true`.
    ///
    /// # Parameters
    /// - `message`: description of the warning, accepted as anything
    ///   convertible into a `String`.
    /// - `loc`: location in the document where the warning was found.
    pub fn warning(&mut self, message: impl FluentMessage, loc: TextLoc) {
        self.warn_count += 1;
        self.items.push(TextParserDiagnostic {
            severity: Severity::Warning,
            message: fluent_message_to_string(message),
            loc,
        });
    }

    /// Report a hint at the given location.
    ///
    /// Hints are purely informational: they are recorded in
    /// [`all`](Self::all) but never counted as errors, regardless of the
    /// `warn_as_error` flag used elsewhere.
    ///
    /// # Parameters
    /// - `message`: description of the hint, accepted as anything
    ///   convertible into a `String`.
    /// - `loc`: location in the document the hint relates to.
    pub fn hint(&mut self, message: impl FluentMessage, loc: TextLoc) {
        self.items.push(TextParserDiagnostic {
            severity: Severity::Hint,
            message: fluent_message_to_string(message),
            loc,
        });
    }

    /// Tell if some errors were reported.
    ///
    /// Returns `true` if at least one error was reported, `false`
    /// otherwise. If `warn_as_error` is `true`, this also returns `true`
    /// if at least one warning was reported (hints are never counted).
    ///
    /// # Examples
    /// ```
    /// use free_surface::config::textloc::TextLoc;
    /// use free_surface::aui::diagnostic::collector::TextParserDiagnostics;
    /// use fluent_message::FluentMessage;
    ///
    /// #[derive(FluentMessage)]
    /// struct WarningMessage {
    /// }
    ///
    /// let mut diag = TextParserDiagnostics::default();
    /// let loc = TextLoc::from(("a_file.txt", 42));
    /// diag.warning(WarningMessage {}, loc);
    /// assert!(!diag.has_errors(false));
    /// assert!(diag.has_errors(true));
    /// ```
    pub fn has_errors(&self, warn_as_error: bool) -> bool {
        self.error_count(warn_as_error) > 0
    }

    /// Number of errors reported.
    ///
    /// If `warn_as_error` is `true`, the count also includes warnings.
    /// Hints are never included.
    pub fn error_count(&self, warn_as_error: bool) -> usize {
        if warn_as_error {
            self.error_count + self.warn_count
        } else {
            self.error_count
        }
    }

    /// Returns all diagnostics reported so far
    ///
    /// Diagnostics are reported in the order they were
    /// added (errors, warnings and hints mixed together).
    pub fn all(&self) -> &[TextParserDiagnostic] {
        &self.items
    }

    /// Helper to forge a [`TextParserDiagnostics`] containing a single
    /// error.
    ///
    /// Equivalent to creating a default collector and calling
    /// [`error`](Self::error) on it once; convenient for building an
    /// early-return diagnostics value from a single failure.
    pub fn from_single_error(message: impl FluentMessage, loc: TextLoc) -> Self {
        let mut diag = Self::default();
        diag.error(message, loc);
        diag
    }
}

fn fluent_message_to_string(message: impl FluentMessage) -> String {
    crate::i18n::lookup_with_args(
        &fluent_i18n::get_locale(),
        message.msg_id(),
        &message.args(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(fluent_message::FluentMessage)]
    #[fluent(prefix = "test-message")]
    enum TestMessage {
        Error { arg: &'static str },
        Warning { arg: &'static str },
        Hint { arg: &'static str },
    }

    fn loc1() -> TextLoc {
        TextLoc::from(("test.txt", 1))
    }
    fn loc2() -> TextLoc {
        TextLoc::from(("another_test.txt", 3))
    }
    fn loc3() -> TextLoc {
        TextLoc::from(("test.txt", 5))
    }

    #[test]
    fn default_diagnostics_is_empty() {
        let diag = TextParserDiagnostics::default();

        assert!(diag.all().is_empty());
        assert_eq!(diag.error_count(false), 0);
        assert_eq!(diag.error_count(true), 0);
        assert!(!diag.has_errors(false));
        assert!(!diag.has_errors(true));
    }

    #[test]
    fn error_is_recorded() {
        let mut diag = TextParserDiagnostics::default();

        diag.error(
            TestMessage::Error {
                arg: "something bad",
            },
            loc1(),
        );

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Error);
        assert!(diag.all()[0].message.contains("test-message-error"));
        assert_eq!(diag.all()[0].loc, loc1());

        assert_eq!(diag.error_count(false), 1);
        assert_eq!(diag.error_count(true), 1);
        assert!(diag.has_errors(false));
        assert!(diag.has_errors(true));
    }

    #[test]
    fn warning_is_recorded_but_not_counted_as_error_by_default() {
        let mut diag = TextParserDiagnostics::default();
        diag.warning(TestMessage::Warning { arg: "careful" }, loc2());

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Warning);
        assert!(diag.all()[0].message.contains("test-message-warning"));
        assert_eq!(diag.all()[0].loc, loc2());

        // Not counted as an error unless warn_as_error is set
        assert_eq!(diag.error_count(false), 0);
        assert!(!diag.has_errors(false));

        // Counted when warn_as_error is true
        assert_eq!(diag.error_count(true), 1);
        assert!(diag.has_errors(true));
    }

    #[test]
    fn hint_is_recorded_but_never_counted_as_error() {
        let mut diag = TextParserDiagnostics::default();
        diag.hint(TestMessage::Hint { arg: "fyi" }, loc1());

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Hint);
        assert!(diag.all()[0].message.contains("test-message-hint"));
        assert_eq!(diag.all()[0].loc, loc1());

        assert_eq!(diag.error_count(false), 0);
        assert_eq!(diag.error_count(true), 0);
        assert!(!diag.has_errors(false));
        assert!(!diag.has_errors(true));
    }

    #[test]
    fn mixed_diagnostics_counts_and_preserves_order() {
        let mut diag = TextParserDiagnostics::default();
        diag.error(TestMessage::Error { arg: "e1" }, loc1());
        diag.warning(TestMessage::Warning { arg: "w1" }, loc2());
        diag.hint(TestMessage::Hint { arg: "h1" }, loc2());
        diag.error(TestMessage::Error { arg: "e2" }, loc3());
        diag.warning(TestMessage::Warning { arg: "w2" }, loc3());

        // Order is preserved as inserted
        let messages: Vec<&str> = diag.all().iter().map(|d| d.message.as_str()).collect();
        let expected_messages = vec![
            "test-message-error",
            "test-message-warning",
            "test-message-hint",
            "test-message-error",
            "test-message-warning",
        ];

        for (actual, expected) in std::iter::zip(messages, expected_messages) {
            assert!(actual.contains(expected));
        }

        let locs: Vec<TextLoc> = diag.all().iter().map(|d| d.loc.clone()).collect();
        assert_eq!(locs, vec![loc1(), loc2(), loc2(), loc3(), loc3()]);

        assert_eq!(diag.error_count(false), 2); // only errors
        assert_eq!(diag.error_count(true), 4); // errors + warnings
        assert!(diag.has_errors(false));
        assert!(diag.has_errors(true));
    }

    #[test]
    fn from_single_error_builds_expected_state() {
        let diag =
            TextParserDiagnostics::from_single_error(TestMessage::Error { arg: "boom" }, loc1());

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Error);
        assert!(diag.all()[0].message.contains("test-message-error"));
        assert_eq!(diag.error_count(false), 1);
        assert!(diag.has_errors(false));
    }

    #[test]
    fn only_warnings_does_not_trigger_has_errors_without_warn_as_error() {
        let mut diag = TextParserDiagnostics::default();
        diag.warning(TestMessage::Warning { arg: "w1" }, loc1());
        diag.warning(TestMessage::Warning { arg: "w2" }, loc1());

        assert!(!diag.has_errors(false));
        assert!(diag.has_errors(true));
        assert_eq!(diag.error_count(true), 2);
    }
}
