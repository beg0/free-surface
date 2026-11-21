//! # "Condition Limites" - CLI - file format
//!
//! CLI files prescribes inflow and outflow conditions
//! at the boundary of the mesh.
//!
//! It is define
//! Details on the file format can be found at:
//! - Section 4.2 of [Telemac 2D user manual](https://gitlab.pam-retd.fr/otm/telemac-mascaret/-/blob/v9.0.0/documentation/telemac2d/user/telemac2d_user_9.0.pdf)
//! - [hydro-informatics](https://hydro-informatics.com/boundaries/#tab-bc-overview) tutorial on boundaries file

use fluent_message::FluentMessage;
use std::path::Path;
use std::{fmt, str::FromStr};
use strum::FromRepr;

use crate::aui::diagnostic::collector::Diagnostics;
use crate::config::textloc::TextLoc;

/// Number of whitespace-separated columns expected on each data line.
///
/// Column order for selafin .cli files (as per telemac implementation):
/// `LIHBOR LIUBOR LIVBOR HBOR UBOR VBOR AUBOR LITBOR TBOR ATBOR BTBOR N NBOR`
const FIELD_COUNT: usize = 13;

#[derive(FluentMessage)]
pub enum CliParseDiagnostic {
    ReadFileFailure {
        #[fluent(display)]
        error: std::io::Error,
    },
    MissingFields {
        expected: usize,
        got: usize,
    },
    ExtraFields {
        expected: usize,
        got: usize,
    },
    ValueParsingError {
        name: String,
        value: String,
        error: String,
    },
    BoundaryTypeParsingError {
        name: String,
        value: String,
    },
}

/// What to prescribe at mesh boundaries
#[derive(Debug, Clone, PartialEq, FromRepr)]
pub enum BoundaryType {
    /// Zero velocity (no-slip / adhesion)
    ///
    /// Forces velocity to zero at the boundary node.
    ///
    /// Apply to u (x flow rate) and v (y flow rate) (occasionally water
    /// depth - h - for specific internal walls).
    ///
    /// Known as KADH in telemac.
    None = 0,

    /// Open boundary with incident wave.
    ///
    /// Known as KINC in telemac.
    IncidentWave = 1,

    /// Solid boundary (wall)
    ///
    /// Impervious boundary with free slip or wall friction (governed by AUBOR and friction laws).
    ///
    /// Apply to h (water depth), u (x flow rate) and v (y flow rate).
    ///
    /// Known as KLOG in telemac.
    Wall = 2,

    /// Free drainage
    ///
    /// Known as KDRAIN in telemac.
    Drain = 3,

    /// Free liquid boundary
    ///
    /// The variable is not imposed and is computed freely by the solver (often
    /// used at outlets or where another variable controls the flow).
    ///
    /// Apply to h (water depth), u (x flow rate) and v (y flow rate).
    ///
    /// Known as KSORT in telemac.
    Output = 4,

    /// Prescribed value (or flowrate)
    ///
    /// When applied to water depth, the water depth/elevation is prescribed.
    /// When applied to x flow rate (u) or y flow rate (v), the discharge (Q)
    /// is prescribed (velocities are derived from Q).
    ///
    /// Apply to h (water depth), u (x flow rate) and v (y flow rate).
    ///
    /// Known as KENT in telemac.
    Input = 5,

    /// Prescribed velocity
    ///
    /// Directly imposes the velocity components U and V (either via UBOR/VBOR
    /// in the .cli file, the steering .cas file, or a liquid boundary file).
    ///
    /// Apply only to u (x flow rate) and v (y flow rate).
    ///
    /// Known as KENTU in telemac.
    InputVelocity = 6,
}

/// One entry (one line) of the boundary condition .cli file
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryCondition {
    /// Type of boundary conditions on H (water depth)
    pub h_boundary_type: BoundaryType,

    /// Type of boundary conditions on U
    pub u_boundary_type: BoundaryType,

    /// Type of boundary conditions on V
    pub v_boundary_type: BoundaryType,

    /// Prescribed boundary condition on H (water depth)
    pub h_prescription: f64,

    /// Prescribed boundary condition on velocity U (x flow rate)
    pub u_prescription: f64,

    /// Prescribed boundary condition on velocity V (y flow rate)
    pub v_prescription: f64,

    /// Friction coefficient at boundary
    pub friction_coefficient: f64,

    /// Physical boundary conditions for tracers
    pub tracer_boundary_type: BoundaryType,

    /// Prescribed boundary condition on tracer
    pub tracer_prescription: f64,

    /// Thermal exchange coefficients
    pub at_prescription: f64,

    /// Thermal exchange coefficients
    pub bt_prescription: f64,

    /// Index of the vertice (point) in the selafin file (a.k.a global index)
    pub vertice_idx: usize,

    /// Boundary color (a.k.a local index)
    pub color: usize,
}

/// Collection of boundary condition records, in file order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoundariesConditions(Vec<BoundaryCondition>);

impl BoundariesConditions {
    /// Parses the content of a telemac `.cli` file for selafin mesh.
    ///
    /// Blank lines and comments are skipped. A comment starts at the first
    /// `#` or `/` on a line and runs to the end of that line.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostics`] pointing at the offending line and columns
    /// for a wrong field count, a value that does not parse, or a boundary
    /// type that `BoundaryType::try_from` rejects.
    ///
    /// # Example
    ///
    /// ```rust
    /// use free_surface::config::cli::BoundariesConditions;
    ///
    /// let input = r"
    /// // inlet and outlet
    /// 4 4 4 1.5 0.0 0.0 0.025 2 0.0 0.0 0.0 1 1
    /// 2 2 2 0.0 0.0 0.0 0.0 0 0.0 0.0 0.0 25 2 # wall
    /// ";
    ///
    /// let bc = BoundariesConditions::parse(input).unwrap();
    /// assert_eq!(bc.len(), 2);
    /// ```
    pub fn parse(input: &str) -> Result<Self, Diagnostics> {
        let (ret, diag) = parse_from_content_and_textloc(input, TextLoc::default());

        if diag.has_errors(false) {
            Err(diag)
        } else {
            Ok(ret)
        }
    }

    /// Parses the content of a telemac `.cli` file with best efforts
    ///
    /// This function is similar to [BoundariesConditions::parse] but does
    /// not return an `Err()` if it fails. Instead it returns a
    /// [BoundariesConditions] which match as much as possible the input
    /// and also the [Diagnostics] that contains errors, warnings and
    /// hint messages.
    ///
    /// If no error are reported, then the returned [BoundariesConditions]
    /// would match what would be returned by a successful
    /// [BoundariesConditions::parse]
    ///
    /// # Example
    ///
    /// ```rust
    /// use free_surface::config::cli::BoundariesConditions;
    ///
    /// let boggy_input = r"
    /// // inlet and outlet
    /// 4 4 4 1.5 0.0 0.0 0.025 2 0.0 0.0 0.0 1 1
    /// 2 2 2 bad input 0.0 0.0 0 0.0 0.0 0.0 25 2 # wall
    /// ";
    ///
    /// let (bc, diag) = BoundariesConditions::parse_best_effort(boggy_input);
    /// assert_eq!(bc.len(), 2);
    /// assert!(diag.has_errors(false))
    /// ```
    pub fn parse_best_effort(input: &str) -> (Self, Diagnostics) {
        parse_from_content_and_textloc(input, TextLoc::default())
    }

    /// Parses a telemac `.cli` file from filename
    ///
    /// Blank lines and comments are skipped. A comment starts at the first
    /// `#` or `/` on a line and runs to the end of that line.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostics`] pointing at the offending line and columns
    /// for a wrong field count, a value that does not parse, or a boundary
    /// type that `BoundaryType::try_from` rejects.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use free_surface::config::cli::BoundariesConditions;
    ///
    /// let bc = BoundariesConditions::parse("path/to/boundary_file.cli").unwrap();
    /// assert_eq!(bc.len(), 50);
    /// ```
    pub fn parse_file<P: AsRef<Path>>(filename: P) -> Result<Self, Diagnostics> {
        let (ret, diag) = Self::parse_file_best_effort(filename);

        if diag.has_errors(false) {
            Err(diag)
        } else {
            Ok(ret)
        }
    }

    /// Parses a telemac `.cli` file from filename with best efforts
    ///
    /// This function is similar to [BoundariesConditions::parse_file] but does
    /// not return an `Err()` if it fails. Instead it returns a
    /// [BoundariesConditions] which match as much as possible the input and
    /// also the [Diagnostics] that contains errors, warnings and hint
    /// messages.
    ///
    /// If no error are reported, then the returned [BoundariesConditions]
    /// would match what would be returned by a successful
    /// [BoundariesConditions::parse_file]
    pub fn parse_file_best_effort<P: AsRef<Path>>(filename: P) -> (Self, Diagnostics) {
        let file_pos = TextLoc::from((&filename, 0));

        match std::fs::read_to_string(&filename) {
            Ok(content) => parse_from_content_and_textloc(&content, file_pos),
            Err(error) => {
                let diag = Diagnostics::from_single_error(
                    CliParseDiagnostic::ReadFileFailure { error },
                    file_pos.clone(),
                );
                (Self::default(), diag)
            }
        }
    }

    /// Number of boundary records.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no records.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Nombre PoinT FRontier - boundary points count
    ///
    /// Alias to [Self.len] for compatibility with telemac
    pub fn nptfr(&self) -> usize {
        self.len()
    }

    pub fn to_nbor(&self) -> Vec<usize> {
        self.iter_nbor().collect()
    }

    pub fn iter_nbor(&self) -> impl Iterator<Item = usize> {
        self.iter().map(|entry| entry.vertice_idx)
    }

    /// Iterates over the records in file order.
    pub fn iter(&self) -> impl Iterator<Item = &BoundaryCondition> {
        self.0.iter()
    }
}

impl FromStr for BoundariesConditions {
    type Err = Diagnostics;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

fn parse_from_content_and_textloc(
    input: &str,
    top_pos: TextLoc,
) -> (BoundariesConditions, Diagnostics) {
    let mut records = Vec::new();
    let mut diag = Diagnostics::default();

    for (idx, raw) in input.lines().enumerate() {
        let line_pos = top_pos.clone_with_line(idx + 1);
        if let Some(record) = parse_line(raw, &line_pos, &mut diag) {
            records.push(record);
        }
    }

    (BoundariesConditions(records), diag)
}

/// A whitespace-delimited token with its byte span in the line.
#[derive(Debug, Clone, Copy)]
struct Token<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

fn tokenize(line: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::with_capacity(FIELD_COUNT);
    let mut current: Option<usize> = None;

    for (i, c) in line.char_indices() {
        match (c.is_whitespace(), current) {
            (false, None) => current = Some(i),
            (true, Some(s)) => {
                tokens.push(Token {
                    start: s,
                    end: i,
                    text: &line[s..i],
                });
                current = None;
            }
            _ => {}
        }
    }

    if let Some(s) = current {
        tokens.push(Token {
            start: s,
            end: line.len(),
            text: &line[s..],
        });
    }

    tokens
}

/// Parses one line. Returns `Ok(None)` for blank and comment-only lines.
fn parse_line(raw: &str, pos: &TextLoc, diag: &mut Diagnostics) -> Option<BoundaryCondition> {
    // Everything from the first '#' or '/' is a comment.
    let content_end = raw.find(['#', '/']).unwrap_or(raw.len());
    let content = &raw[..content_end];

    let tokens = tokenize(content);
    if tokens.is_empty() {
        return None;
    }

    if tokens.len() < FIELD_COUNT {
        let last = tokens[tokens.len() - 1];
        let last_token_pos = pos.clone_with_line_offset_col(0, last.end + 1);

        diag.error(CliParseDiagnostic::MissingFields {
            expected: FIELD_COUNT,
            got: tokens.len(),
        })
        .with_textloc(last_token_pos);
        return None;
    }

    if tokens.len() > FIELD_COUNT {
        let extra_start = tokens[FIELD_COUNT].start;
        //let extra_end = tokens[tokens.len() - 1].end;
        let extra_tokens_pos = pos.clone_with_line_offset_col(0, extra_start + 1);
        diag.warning(CliParseDiagnostic::ExtraFields {
            expected: FIELD_COUNT,
            got: tokens.len(),
        })
        .with_textloc(extra_tokens_pos);
    }

    let t = &tokens;
    Some(BoundaryCondition {
        h_boundary_type: parse_boundary_type(&t[0], "LIHBOR", pos, diag),
        u_boundary_type: parse_boundary_type(&t[1], "LIUBOR", pos, diag),
        v_boundary_type: parse_boundary_type(&t[2], "LIVBOR", pos, diag),
        h_prescription: parse_value(&t[3], "HBOR", pos, diag),
        u_prescription: parse_value(&t[4], "UBOR", pos, diag),
        v_prescription: parse_value(&t[5], "VBOR", pos, diag),
        friction_coefficient: parse_value(&t[6], "AUBOR", pos, diag),
        tracer_boundary_type: parse_boundary_type(&t[7], "LITBOR", pos, diag),
        tracer_prescription: parse_value(&t[8], "TBOR", pos, diag),
        at_prescription: parse_value(&t[9], "ATBOR", pos, diag),
        bt_prescription: parse_value(&t[10], "BTBOR", pos, diag),
        vertice_idx: parse_value(&t[11], "N", pos, diag),
        color: parse_value(&t[12], "NBOR", pos, diag),
    })
}

/// Parses a plain value (`f64`, `u32`, `isize`, ...) from a token.
fn parse_value<T>(token: &Token<'_>, name: &str, pos: &TextLoc, diag: &mut Diagnostics) -> T
where
    T: Default + FromStr,
    T::Err: fmt::Display,
{
    match token.text.parse::<T>() {
        Ok(value) => value,
        Err(e) => {
            let error_pos = pos.clone_with_line_offset_col(0, token.start + 1);
            let error_msg = format!("{}", e);
            diag.error(CliParseDiagnostic::ValueParsingError {
                name: name.into(),
                value: token.text.into(),
                error: error_msg,
            })
            .with_textloc(error_pos);
            T::default()
        }
    }
}

/// Parses a boundary type code and converts it with `BoundaryType::try_from`.
fn parse_boundary_type(
    token: &Token<'_>,
    name: &str,
    pos: &TextLoc,
    diag: &mut Diagnostics,
) -> BoundaryType {
    let code: usize = parse_value(token, name, pos, diag);

    BoundaryType::from_repr(code).unwrap_or_else(|| {
        let error_pos = pos.clone_with_line_offset_col(0, token.start + 1);
        diag.error(CliParseDiagnostic::BoundaryTypeParsingError {
            name: name.into(),
            value: token.text.into(),
        })
        .with_textloc(error_pos);
        BoundaryType::None
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::aui::diagnostic::collector::{DiagnosticLocation, Severity};

    /// Valid line: LIHBOR=2 LIUBOR=2 LIVBOR=2 HBOR=1.5 UBOR=0.0 VBOR=0.0
    /// AUBOR=0.025 LITBOR=2 TBOR=0.0 ATBOR=0.0 BTBOR=0.0 N=10 NBOR=1
    const VALID_LINE: &str = "2 2 2 1.5 0.0 0.0 0.025 2 0.0 0.0 0.0 10 1";

    fn bt(code: usize) -> BoundaryType {
        BoundaryType::from_repr(code).expect("valid boundary type code in test")
    }

    #[test]
    fn parses_single_valid_record() {
        let bc = BoundariesConditions::parse(VALID_LINE).expect("should parse");
        assert_eq!(bc.len(), 1);

        let rec = bc.iter().next().unwrap();
        assert_eq!(rec.h_boundary_type, bt(2));
        assert_eq!(rec.u_boundary_type, bt(2));
        assert_eq!(rec.v_boundary_type, bt(2));
        assert_eq!(rec.h_prescription, 1.5);
        assert_eq!(rec.u_prescription, 0.0);
        assert_eq!(rec.v_prescription, 0.0);
        assert_eq!(rec.friction_coefficient, 0.025);
        assert_eq!(rec.tracer_boundary_type, bt(2));
        assert_eq!(rec.tracer_prescription, 0.0);
        assert_eq!(rec.at_prescription, 0.0);
        assert_eq!(rec.bt_prescription, 0.0);
        assert_eq!(rec.vertice_idx, 10);
        assert_eq!(rec.color, 1);
    }

    #[test]
    fn skips_blank_lines_and_comments() {
        let input = format!(
            "\n   \n# header comment\n/ slash comment\n\t\n{VALID_LINE}\n   # indented comment\n"
        );
        let bc = BoundariesConditions::parse(&input).expect("should parse");
        assert_eq!(bc.len(), 1);
    }

    #[test]
    fn strips_trailing_comments() {
        let input = format!("{VALID_LINE} # inlet\n{VALID_LINE} / outlet\n");
        let bc = BoundariesConditions::parse(&input).expect("should parse");
        assert_eq!(bc.len(), 2);
    }

    #[test]
    fn empty_input_gives_no_records() {
        let bc = BoundariesConditions::parse("").expect("should parse");
        assert!(bc.is_empty());

        let bc =
            BoundariesConditions::parse("# only comments\n\n/ nothing\n").expect("should parse");
        assert!(bc.is_empty());
    }

    #[test]
    fn preserves_record_order() {
        let input = "2 2 2 1.0 0.0 0.0 0.0 2 0.0 0.0 0.0 1 1\n\
                     2 2 2 2.0 0.0 0.0 0.0 2 0.0 0.0 0.0 2 2\n\
                     2 2 2 3.0 0.0 0.0 0.0 2 0.0 0.0 0.0 3 3\n";
        let bc = BoundariesConditions::parse(input).expect("should parse");
        let heights: Vec<f64> = bc.iter().map(|r| r.h_prescription).collect();
        let indices: Vec<usize> = bc.iter().map(|r| r.vertice_idx).collect();
        assert_eq!(heights, vec![1.0, 2.0, 3.0]);
        assert_eq!(indices, vec![1, 2, 3]);
    }

    #[test]
    fn handles_crlf_line_endings_and_tabs() {
        let input = "2\t2\t2\t1.5\t0.0\t0.0\t0.025\t2\t0.0\t0.0\t0.0\t10\t1\r\n";
        let bc = BoundariesConditions::parse(input).expect("should parse");
        assert_eq!(bc.len(), 1);
        assert_eq!(bc.iter().next().unwrap().vertice_idx, 10);
    }

    #[test]
    fn reports_invalid_float_with_columns() {
        // "2 2 2 " is 6 characters, so "abc" occupies columns 7..=9.
        let input = "2 2 2 abc 0.0 0.0 0.025 2 0.0 0.0 0.0 10 1\n";
        let diag = BoundariesConditions::parse(input).unwrap_err();
        let all_diags = diag.all();
        assert_eq!(all_diags.len(), 1);
        let diag0 = &all_diags[0];

        assert_eq!(diag0.severity, Severity::Error);
        assert_eq!(
            diag0.loc,
            DiagnosticLocation::TextLoc(TextLoc::default().clone_with_line_col(1, 7))
        );
        // assert_eq!(err.start, 7);
        // assert_eq!(err.end, 9);
        assert!(diag0.message.contains("HBOR"), "message: {}", diag0.message);
    }

    #[test]
    fn line_number_counts_skipped_lines() {
        let input = format!("# comment\n\n{VALID_LINE}\n2 2 2 x 0 0 0 2 0 0 0 1 1\n");
        let diag = BoundariesConditions::parse(input.as_str()).unwrap_err();
        let all_diags = diag.all();
        assert_eq!(all_diags.len(), 1);
        let diag0 = &all_diags[0];

        assert_eq!(diag0.severity, Severity::Error);
        assert_eq!(
            diag0.loc,
            DiagnosticLocation::TextLoc(TextLoc::default().clone_with_line_col(4, 7))
        );
        // assert_eq!(err.start, 7);
        // assert_eq!(err.end, 7);
    }

    #[test]
    fn reports_too_few_fields() {
        let input = "2 2 2 1.5 0.0 0.0\n";
        let diag = BoundariesConditions::parse(input).unwrap_err();
        let all_diags = diag.all();
        assert_eq!(all_diags.len(), 1);
        let diag0 = &all_diags[0];

        assert_eq!(diag0.severity, Severity::Error);
        assert_eq!(
            diag0.loc,
            DiagnosticLocation::TextLoc(TextLoc::default().clone_with_line_col(1, 18))
        );

        assert!(
            diag0.message.contains("expected 13 fields, found 6"),
            "message: {}",
            diag0.message
        );
        // Points at the last token, "0.0", which is columns 15..=17.
        // assert_eq!(err.start, 15);
        // assert_eq!(err.end, 17);
    }

    #[test]
    fn reports_too_many_fields() {
        let input = format!("{VALID_LINE} 99\n");
        let (bc, diag) = BoundariesConditions::parse_best_effort(&input);

        assert_eq!(bc.len(), 1);

        let all_diags = diag.all();
        assert_eq!(all_diags.len(), 1);
        let diag0 = &all_diags[0];

        assert_eq!(diag0.severity, Severity::Warning);

        // Extra token "99" starts after the 13 valid columns plus one space.
        let expected_start = VALID_LINE.len() + 2;
        // assert_eq!(err.start, expected_start);
        // assert_eq!(err.end, expected_start + 1);

        assert_eq!(
            diag0.loc,
            DiagnosticLocation::TextLoc(TextLoc::default().clone_with_line_col(1, expected_start))
        );

        assert!(
            diag0.message.contains("found 14"),
            "message: {}",
            diag0.message
        );
    }

    #[test]
    fn reports_invalid_boundary_type() {
        // 999 is assumed not to be a valid BoundaryType code.
        let input = "2 2 999 1.5 0.0 0.0 0.025 2 0.0 0.0 0.0 10 1\n";
        let diag = BoundariesConditions::parse(input).unwrap_err();
        let all_diags = diag.all();
        assert_eq!(all_diags.len(), 1);
        let diag0 = &all_diags[0];

        assert_eq!(diag0.severity, Severity::Error);

        // "2 2 " is 4 characters, so "999" occupies columns 5..=7.
        assert_eq!(
            diag0.loc,
            DiagnosticLocation::TextLoc(TextLoc::default().clone_with_line_col(1, 5))
        );
        // assert_eq!(err.start, 5);
        // assert_eq!(err.end, 7);
        assert!(
            diag0.message.contains("LIVBOR"),
            "message: {}",
            diag0.message
        );
    }

    #[test]
    fn rejects_non_integer_node_index() {
        let input = "2 2 2 1.5 0.0 0.0 0.025 2 0.0 0.0 0.0 1.5 1\n";
        let diag = BoundariesConditions::parse(input).unwrap_err();
        let all_diags = diag.all();
        assert_eq!(all_diags.len(), 1);
        let diag0 = &all_diags[0];

        assert!(
            diag0.message.contains("'N'") || diag0.message.contains(" N "),
            "message: {}",
            diag0.message
        );
    }

    #[test]
    fn rejects_negative_color() {
        let input = "2 2 2 1.5 0.0 0.0 0.025 2 0.0 0.0 0.0 10 -1\n";
        let diag = BoundariesConditions::parse(input).unwrap_err();
        let all_diags = diag.all();
        assert_eq!(all_diags.len(), 1);
        let diag0 = &all_diags[0];

        assert_eq!(
            diag0.loc,
            DiagnosticLocation::TextLoc(TextLoc::default().clone_with_line_col(1, 42))
        );
        assert!(diag0.message.contains("NBOR"));
    }

    #[test]
    fn from_str_matches_parse() {
        let bc: BoundariesConditions = VALID_LINE.parse().expect("should parse");
        assert_eq!(bc.len(), 1);
    }
}

// Ignore spelling some french sentences from telemac
// cSpell:ignore Condition Limites
// Also ignore telemac variables
// cSpell:ignore NBOR LIHBOR LIUBOR LIVBOR HBOR UBOR VBOR AUBOR LITBOR TBOR ATBOR BTBOR NBOR
// cSpell:ignore Nombre PoinT FRontier nptfr
