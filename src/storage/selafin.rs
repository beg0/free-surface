//! # Selafin file format
//!
//! Selafin is used to store geometry and results
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
    pub title: String, // Title of the study

    //pub nvar: u32,
    //pub varindex: [u32]
    pub origin: (u32, u32), // (X,Y) coordinate of origin

    pub boundaries_count: u32, // Number of boundaries (for parallel computation)
    pub interfaces_count: u32, // Number of interfaces (for parallel computation)

    pub nelem2: u32,
    pub npoin2: u32,
    pub npd2: i32,
    pub ikle2: Vec<u32>,
    pub ipob2: Vec<i32>,

    pub nelem3: u32,
    pub npoin3: u32,
    pub npd3: u32,
    pub nplan: u32, // Number of planes (3D computation)
    pub ikle3: Vec<u32>,
    pub ipob3: Vec<i32>,

    pub var: Vec<SlfVariable>, // Variables stored in history results
    pub cld: Vec<SlfVariable>, // Variables stored in history results
    pub mesh: SlfMesh,         // Geometry
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
    pub fn nbvar(&self) -> usize {
        self.var.len() + self.cld.len()
    }

    pub fn nbvar1(&self) -> usize {
        self.var.len()
    }

    pub fn nbvar2(&self) -> usize {
        self.cld.len()
    }
}
