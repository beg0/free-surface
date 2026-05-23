//! # Selafin file format
//!
//! Selafin is used to store geometry and results.
//!
//! Selafin is sometimes spelled Serafin, or even Selaphin.
//!

pub mod container;
mod geometry;
mod parser;
mod variable;

use chrono::NaiveDateTime;
use geometry::SlfGeometry;
use variable::{SlfVariable, TimeSerie};

pub use parser::{parse, parse_file};

#[derive(Debug, Default)]
pub struct Selafin {
    /// Title of the study
    title: String,

    /// (X,Y) coordinate of origin
    pub origin: (u32, u32),

    geo: SlfGeometry,

    /// Linear variables stored in history results
    var: Vec<SlfVariable>,

    /// Quadratic variables stored in history results
    cld: Vec<SlfVariable>,

    /// Value of each variable at each node and each time step
    results: TimeSerie,

    /// Date & time of creation of the Selafin
    pub datetime: Option<NaiveDateTime>,
}

impl Selafin {
    /// Title of the study
    pub fn title(&self) -> &String {
        &self.title
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

    pub fn results(&self) -> &TimeSerie {
        &self.results
    }

    pub fn geometry(&self) -> &SlfGeometry {
        &self.geo
    }
}

// cSpell:ignore Selaphin
