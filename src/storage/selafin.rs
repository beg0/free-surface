//! # Selafin file format
//!
//! Selafin is used to store geometry and results.
//!
//! Selafin is sometimes spelled Serafin, or even Selaphin.
//!

pub mod container;
pub mod geometry;
mod parser;
mod variable;
mod writer;

use chrono::NaiveDateTime;
use geometry::SlfGeometry;
use variable::{SlfVariable, TimeSerie};

pub use parser::{parse, parse_file};
pub use writer::{write, write_file};

#[derive(Debug, Default)]
pub struct Selafin {
    /// Title of the study
    title: String,

    /// (X,Y) coordinate of origin
    origin: (u32, u32),

    geo: SlfGeometry,

    /// Linear variables stored in history results
    var: Vec<SlfVariable>,

    /// Quadratic variables stored in history results
    cld: Vec<SlfVariable>,

    /// Value of each variable at each node and each time step
    results: TimeSerie,

    /// Date & time of creation of the Selafin
    datetime: Option<NaiveDateTime>,
}

impl Selafin {
    /// Title of the study
    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn origin(&self) -> (u32, u32) {
        self.origin
    }

    /// Return total number of variable in Selafin file
    pub fn nbvar(&self) -> usize {
        self.var.len() + self.cld.len()
    }

    /// Return number of linear variables
    pub fn nbvar1(&self) -> usize {
        self.var.len()
    }

    /// Return number of quadratic variables
    pub fn nbvar2(&self) -> usize {
        self.cld.len()
    }

    /// The ordered list of linear variable definitions.
    pub fn var_defs(&self) -> &[SlfVariable] {
        &self.var
    }

    /// The ordered list of quadratic variable definitions.
    pub fn cld_defs(&self) -> &[SlfVariable] {
        &self.cld
    }

    pub fn results(&self) -> &TimeSerie {
        &self.results
    }

    pub fn geometry(&self) -> &SlfGeometry {
        &self.geo
    }

    pub fn datetime(&self) -> Option<NaiveDateTime> {
        self.datetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    // -----------------------------------------------------------------------
    // Test helper
    //
    // All fields are private, so we build instances through Default and
    // direct field mutation inside this cfg(test) block, which lives in the
    // same module as the struct and therefore has access to private fields.
    // -----------------------------------------------------------------------

    struct SelafinBuilder {
        inner: Selafin,
    }

    impl SelafinBuilder {
        fn new() -> Self {
            Self {
                inner: Selafin::default(),
            }
        }

        fn title(mut self, t: &str) -> Self {
            self.inner.title = t.to_string();
            self
        }

        fn var(mut self, vars: Vec<SlfVariable>) -> Self {
            self.inner.var = vars;
            self
        }

        fn cld(mut self, cld: Vec<SlfVariable>) -> Self {
            self.inner.cld = cld;
            self
        }

        fn datetime(mut self, dt: NaiveDateTime) -> Self {
            self.inner.datetime = Some(dt);
            self
        }

        fn build(self) -> Selafin {
            self.inner
        }
    }

    fn make_var(name: &str) -> SlfVariable {
        SlfVariable::new(name, "M")
    }

    fn make_datetime() -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(1972, 7, 13)
            .unwrap()
            .and_hms_opt(17, 15, 13)
            .unwrap()
    }

    // -----------------------------------------------------------------------
    // Default
    // -----------------------------------------------------------------------

    #[test]
    fn default_title_is_empty() {
        assert!(Selafin::default().title().is_empty());
    }

    #[test]
    fn default_has_no_variables() {
        let s = Selafin::default();
        assert_eq!(s.nbvar(), 0);
        assert_eq!(s.nbvar1(), 0);
        assert_eq!(s.nbvar2(), 0);
    }

    #[test]
    fn default_datetime_is_none() {
        assert!(Selafin::default().datetime.is_none());
    }

    #[test]
    fn default_origin_is_zero() {
        let s = Selafin::default();
        assert_eq!(s.origin(), (0, 0));
    }

    // -----------------------------------------------------------------------
    // title
    // -----------------------------------------------------------------------

    #[test]
    fn title_returns_set_value() {
        let s = SelafinBuilder::new().title("TEST STUDY").build();
        assert_eq!(s.title(), "TEST STUDY");
    }

    #[test]
    fn title_returns_empty_string_by_default() {
        assert_eq!(Selafin::default().title(), "");
    }

    // -----------------------------------------------------------------------
    // nbvar / nbvar1 / nbvar2
    // -----------------------------------------------------------------------

    #[test]
    fn nbvar1_counts_linear_variables() {
        let s = SelafinBuilder::new()
            .var(vec![make_var("DEPTH"), make_var("VELOCITY U")])
            .build();
        assert_eq!(s.nbvar1(), 2);
    }

    #[test]
    fn nbvar2_counts_quadratic_variables() {
        let s = SelafinBuilder::new()
            .cld(vec![make_var("K"), make_var("EPSILON"), make_var("NU_T")])
            .build();
        assert_eq!(s.nbvar2(), 3);
    }

    #[test]
    fn nbvar_is_sum_of_nbvar1_and_nbvar2() {
        let s = SelafinBuilder::new()
            .var(vec![make_var("DEPTH"), make_var("VELOCITY U")])
            .cld(vec![make_var("K")])
            .build();
        assert_eq!(s.nbvar(), s.nbvar1() + s.nbvar2());
        assert_eq!(s.nbvar(), 3);
    }

    #[test]
    fn nbvar_is_zero_with_no_variables() {
        assert_eq!(Selafin::default().nbvar(), 0);
    }

    #[test]
    fn nbvar_with_only_linear_variables() {
        let s = SelafinBuilder::new().var(vec![make_var("DEPTH")]).build();
        assert_eq!(s.nbvar(), 1);
        assert_eq!(s.nbvar2(), 0);
    }

    #[test]
    fn nbvar_with_only_quadratic_variables() {
        let s = SelafinBuilder::new().cld(vec![make_var("K")]).build();
        assert_eq!(s.nbvar(), 1);
        assert_eq!(s.nbvar1(), 0);
    }

    // -----------------------------------------------------------------------
    // datetime
    // -----------------------------------------------------------------------

    #[test]
    fn datetime_is_none_when_not_set() {
        assert!(Selafin::default().datetime().is_none());
    }

    #[test]
    fn datetime_is_some_when_set() {
        let s = SelafinBuilder::new().datetime(make_datetime()).build();
        assert!(s.datetime().is_some());
    }

    #[test]
    fn datetime_stores_correct_value() {
        let dt = make_datetime();
        let s = SelafinBuilder::new().datetime(dt).build();
        assert_eq!(s.datetime().unwrap(), dt);
    }

    #[test]
    fn datetime_is_correct() {
        let dt = make_datetime();
        let s = SelafinBuilder::new().datetime(dt).build();
        let stored = s.datetime().unwrap();
        assert_eq!(
            format!("{}", stored.date().format("%Y-%m-%d")),
            "1972-07-13"
        );
        assert_eq!(format!("{}", stored.time().format("%H:%M:%S")), "17:15:13");
    }
}

// cSpell:ignore Selaphin
