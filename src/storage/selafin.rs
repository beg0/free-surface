//! # Selafin file format
//!
//! Selafin is used to store geometry and results.
//!
//! Selafin is sometimes spelled Serafin, or even Selaphin.
//!

use chrono::NaiveDateTime;

mod parser;
pub use parser::parse_file;

#[derive(Debug, Clone)]
pub struct SlfVariable {
    pub name: String,
    pub unit: String,
}

impl SlfVariable {
    pub fn new(name: &str, unit: &str) -> Self {
        SlfVariable {
            name: name.to_string(),
            unit: unit.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum SlfMesh {
    Float { x: Vec<f32>, y: Vec<f32> },
    Double { x: Vec<f64>, y: Vec<f64> },
}
#[derive(Debug)]
pub struct Selafin {
    /// Title of the study
    title: String,

    /// (X,Y) coordinate of origin
    pub origin: (u32, u32),

    /// Number of boundaries (for parallel computation)
    pub boundaries_count: u32,

    /// Number of interfaces (for parallel computation)
    pub interfaces_count: u32,

    pub nelem2: u32,
    pub npoin2: u32,
    pub npd2: i32,
    pub ikle2: Vec<u32>,
    pub ipob2: Vec<i32>,

    nelem3: u32,
    npoin3: u32,

    /// Number of points per elements
    /// Typically 3 in 2D and 6 in 3D
    npd3: u32,

    /// Number of planes (3D computation)
    nplan: u32,

    /// The connectivity table, size of 'nelem3' * 'npd3'
    /// Indexes of nodes to connect each nodes together (0-based indexes)
    ikle3: Vec<u32>,

    /// Indexes of nodes at the boundary (0-based indexes)
    /// The value of an element is 0 for an inner point and yields the edge
    /// point numbers for the others),
    ipob3: Vec<i32>,

    /// Linear variables stored in history results
    var: Vec<SlfVariable>,

    /// Quadratic variables stored in history results
    cld: Vec<SlfVariable>,

    /// Coordinates of each points of the mesh
    mesh: SlfMesh,

    /// Date & time of creation of the Selafin
    pub datetime: NaiveDateTime,
}

impl Default for Selafin {
    fn default() -> Self {
        Selafin {
            title: String::new(),

            //pub nvar: u32,
            //pub varindex: [u32]
            origin: (0, 0),

            boundaries_count: 0,
            interfaces_count: 0,

            nelem2: 0,
            npoin2: 0,
            npd2: 0,
            ikle2: vec![],
            ipob2: vec![],
            nelem3: 0,
            npoin3: 0,
            npd3: 0,
            nplan: 1,
            ikle3: vec![],
            ipob3: vec![],

            var: vec![],
            // pub nbv1: u32 = 0,
            // pub varnames: [String],
            // pub varunits: [String],
            cld: vec![],
            // pub nbv2: u32 = 0,
            // pub cldnames: [String],
            // pub cldunits: [String],
            mesh: SlfMesh::Float {
                x: vec![],
                y: vec![],
            },
            datetime: NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(1972, 7, 13).unwrap(),
                chrono::NaiveTime::from_hms_opt(17, 15, 13).unwrap(),
            ),
        }
    }
}

impl Selafin {
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
}

// cSpell:ignore Selaphin
