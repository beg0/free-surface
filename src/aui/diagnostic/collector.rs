//! # Parsing error collection
//!

use fluent_message::FluentMessage;
use std::fmt;

use crate::config::textloc::TextLoc;

/// Diagnostic severity
#[derive(Clone, PartialEq, Debug)]
pub enum Severity {
    /// An error that the system cannot recover from
    Error,

    /// A warning can be two different things:
    /// 1. Something that the system may have interpreted in some way
    ///    and that the user may not expect
    /// 2. or an error that the system has recover in a certain way but
    ///    which may lead to something unexpected for the user
    Warning,

    /// Something that can help to resolve an error or a warning
    Hint,
}

/// Location associated to a diagnostic (e.g. where it occurs)
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticLocation {
    TextLoc(TextLoc),
    Filename(String),
    None,
}

impl fmt::Display for DiagnosticLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLocation::TextLoc(loc) => loc.fmt(f),
            DiagnosticLocation::Filename(filename) => filename.fmt(f),
            DiagnosticLocation::None => write!(f, ""),
        }
    }
}

/// A single diagnostic.
///
/// A diagnostic carries a [`Severity`], a human-readable `message`
/// describing the problem, and the [`DiagnosticLocation`] pointing to
/// where in the source document the problem was found.
///
/// Collected through [`Diagnostics`].
#[derive(Debug)]
pub struct Diagnostic {
    /// How severe the diagnostic is (error, warning or hint).
    pub severity: Severity,

    /// Human-readable description of the problem.
    pub message: String,

    /// To which file (and potentially position) this diagnostic refers to.
    pub loc: DiagnosticLocation,
}

impl Diagnostic {
    /// Add [TextLoc] info to a diagnostic
    /// # Parameters
    /// - `loc`: location in the document where the diagnostic is.
    pub fn with_textloc(&mut self, loc: TextLoc) -> &mut Self {
        self.loc = DiagnosticLocation::TextLoc(loc);
        self
    }

    /// Add filename info to a diagnostic
    pub fn with_filename(&mut self, filename: impl Into<String>) -> &mut Self {
        self.loc = DiagnosticLocation::Filename(filename.into());
        self
    }
}

/// Diagnostic collector
///
/// Diagnostics are appended via [`error`](Self::error),
/// [`warning`](Self::warning) and [`hint`](Self::hint) while parsing
/// proceeds. The collector keeps a running count of errors and warnings
/// so that callers can cheaply check whether parsing failed with
/// [`has_errors`](Self::has_errors), without having to walk the full list
/// of diagnostics.
#[derive(Default, Debug)]
pub struct Diagnostics {
    /// All diagnostics reported so far, in report order.
    items: Vec<Diagnostic>,

    /// Number of error encountered.
    error_count: usize,

    /// Number of warning encountered.
    warn_count: usize,
}

impl Diagnostics {
    /// Report an error at the given location.
    ///
    /// An error is to be emitted when the caller encounter an error or an
    /// unexpected situation that it can't deal with
    /// Increments the internal error count, which affects the value
    /// returned by [`error_count`](Self::error_count) and
    /// [`has_errors`](Self::has_errors).
    ///
    /// # Parameters
    /// - `message`: description of the error.
    pub fn error(&mut self, message: impl FluentMessage) -> &mut Diagnostic {
        self.error_count += 1;
        self.items.push_mut(Diagnostic {
            severity: Severity::Error,
            message: fluent_message_to_string(message),
            loc: DiagnosticLocation::None,
        })
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
    /// - `message`: description of the warning.
    pub fn warning(&mut self, message: impl FluentMessage) -> &mut Diagnostic {
        self.warn_count += 1;
        self.items.push_mut(Diagnostic {
            severity: Severity::Warning,
            message: fluent_message_to_string(message),
            loc: DiagnosticLocation::None,
        })
    }

    /// Report a hint at the given location.
    ///
    /// Hints are purely informational: they are recorded in
    /// [`all`](Self::all) but never counted as errors, regardless of the
    /// `warn_as_error` flag used elsewhere.
    ///
    /// # Parameters
    /// - `message`: description of the hint.
    pub fn hint(&mut self, message: impl FluentMessage) -> &mut Diagnostic {
        self.items.push_mut(Diagnostic {
            severity: Severity::Hint,
            message: fluent_message_to_string(message),
            loc: DiagnosticLocation::None,
        })
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
    /// use free_surface::aui::diagnostic::collector::Diagnostics;
    /// use fluent_message::FluentMessage;
    ///
    /// #[derive(FluentMessage)]
    /// struct WarningMessage {
    /// }
    ///
    /// let mut diag = Diagnostics::default();
    /// let loc = TextLoc::from(("a_file.txt", 42));
    /// diag.warning(WarningMessage {})
    ///     .with_textloc(loc);
    /// assert!(!diag.has_errors(false));
    /// assert!(diag.has_errors(true));
    /// ```
    #[must_use]
    pub fn has_errors(&self, warn_as_error: bool) -> bool {
        self.error_count(warn_as_error) > 0
    }

    /// Number of errors reported.
    ///
    /// If `warn_as_error` is `true`, the count also includes warnings.
    /// Hints are never included.
    #[must_use]
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
    #[must_use]
    pub fn all(&self) -> &[Diagnostic] {
        &self.items
    }

    /// Helper to forge a [`Diagnostics`] containing a single
    /// error.
    ///
    /// Equivalent to creating a default collector and calling
    /// [`error`](Self::error) on it once; convenient for building an
    /// early-return diagnostics value from a single failure.
    #[must_use]
    pub fn from_single_error(message: impl FluentMessage, loc: TextLoc) -> Self {
        let mut diag = Self::default();
        diag.error(message).with_textloc(loc);
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
    fn filename1() -> String {
        String::from("the_ultimate_test.txt")
    }

    fn filename2() -> String {
        String::from("strange_file.txt")
    }

    #[test]
    fn default_diagnostics_is_empty() {
        let diag = Diagnostics::default();

        assert!(diag.all().is_empty());
        assert_eq!(diag.error_count(false), 0);
        assert_eq!(diag.error_count(true), 0);
        assert!(!diag.has_errors(false));
        assert!(!diag.has_errors(true));
    }

    #[test]
    fn error_is_recorded() {
        let mut diag = Diagnostics::default();

        diag.error(TestMessage::Error {
            arg: "something bad",
        })
        .with_textloc(loc1());

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Error);
        assert!(diag.all()[0].message.contains("test-message-error"));
        assert_eq!(diag.all()[0].loc, DiagnosticLocation::TextLoc(loc1()));

        assert_eq!(diag.error_count(false), 1);
        assert_eq!(diag.error_count(true), 1);
        assert!(diag.has_errors(false));
        assert!(diag.has_errors(true));
    }

    #[test]
    fn warning_is_recorded_but_not_counted_as_error_by_default() {
        let mut diag = Diagnostics::default();
        diag.warning(TestMessage::Warning { arg: "careful" })
            .with_textloc(loc2());

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Warning);
        assert!(diag.all()[0].message.contains("test-message-warning"));
        assert_eq!(diag.all()[0].loc, DiagnosticLocation::TextLoc(loc2()));

        // Not counted as an error unless warn_as_error is set
        assert_eq!(diag.error_count(false), 0);
        assert!(!diag.has_errors(false));

        // Counted when warn_as_error is true
        assert_eq!(diag.error_count(true), 1);
        assert!(diag.has_errors(true));
    }

    #[test]
    fn hint_is_recorded_but_never_counted_as_error() {
        let mut diag = Diagnostics::default();
        diag.hint(TestMessage::Hint { arg: "fyi" })
            .with_textloc(loc1());

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Hint);
        assert!(diag.all()[0].message.contains("test-message-hint"));
        assert_eq!(diag.all()[0].loc, DiagnosticLocation::TextLoc(loc1()));

        assert_eq!(diag.error_count(false), 0);
        assert_eq!(diag.error_count(true), 0);
        assert!(!diag.has_errors(false));
        assert!(!diag.has_errors(true));
    }

    #[test]
    fn mixed_diagnostics_counts_and_preserves_order() {
        let mut diag = Diagnostics::default();
        diag.error(TestMessage::Error { arg: "e1" })
            .with_textloc(loc1());
        diag.warning(TestMessage::Warning { arg: "w1" })
            .with_textloc(loc2());
        diag.hint(TestMessage::Hint { arg: "h1" })
            .with_textloc(loc2());
        diag.error(TestMessage::Error { arg: "e2" })
            .with_textloc(loc3());
        diag.warning(TestMessage::Warning { arg: "w2" })
            .with_textloc(loc3());

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

        let locs: Vec<DiagnosticLocation> = diag.all().iter().map(|d| d.loc.clone()).collect();
        assert_eq!(
            locs,
            vec![
                DiagnosticLocation::TextLoc(loc1()),
                DiagnosticLocation::TextLoc(loc2()),
                DiagnosticLocation::TextLoc(loc2()),
                DiagnosticLocation::TextLoc(loc3()),
                DiagnosticLocation::TextLoc(loc3())
            ]
        );

        assert_eq!(diag.error_count(false), 2); // only errors
        assert_eq!(diag.error_count(true), 4); // errors + warnings
        assert!(diag.has_errors(false));
        assert!(diag.has_errors(true));
    }

    #[test]
    fn diags_with_filename() {
        let mut diag = Diagnostics::default();

        diag.error(TestMessage::Error {
            arg: "something bad",
        })
        .with_filename(filename1());

        diag.warning(TestMessage::Warning {
            arg: "something strange",
        })
        .with_filename(filename2());

        diag.hint(TestMessage::Hint { arg: "take a note" })
            .with_filename(filename1());

        diag.error(TestMessage::Error {
            arg: "error with loc",
        })
        .with_textloc(loc1());

        assert_eq!(diag.all().len(), 4);
        let severities: Vec<Severity> = diag.all().iter().map(|d| d.severity.clone()).collect();
        assert_eq!(
            severities,
            vec![
                Severity::Error,
                Severity::Warning,
                Severity::Hint,
                Severity::Error,
            ]
        );

        let locs: Vec<DiagnosticLocation> = diag.all().iter().map(|d| d.loc.clone()).collect();
        assert_eq!(
            locs,
            vec![
                DiagnosticLocation::Filename(filename1()),
                DiagnosticLocation::Filename(filename2()),
                DiagnosticLocation::Filename(filename1()),
                DiagnosticLocation::TextLoc(loc1()),
            ]
        );
    }
    #[test]
    fn from_single_error_builds_expected_state() {
        let diag = Diagnostics::from_single_error(TestMessage::Error { arg: "boom" }, loc1());

        assert_eq!(diag.all().len(), 1);
        assert_eq!(diag.all()[0].severity, Severity::Error);
        assert!(diag.all()[0].message.contains("test-message-error"));
        assert_eq!(diag.error_count(false), 1);
        assert!(diag.has_errors(false));
    }

    #[test]
    fn only_warnings_does_not_trigger_has_errors_without_warn_as_error() {
        let mut diag = Diagnostics::default();
        diag.warning(TestMessage::Warning { arg: "w1" })
            .with_textloc(loc1());
        diag.warning(TestMessage::Warning { arg: "w2" })
            .with_textloc(loc1());

        assert!(!diag.has_errors(false));
        assert!(diag.has_errors(true));
        assert_eq!(diag.error_count(true), 2);
    }
}
