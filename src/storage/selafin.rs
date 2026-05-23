//! # Selafin file format
//!
//! Selafin is used to store geometry and results.
//!
//! Selafin is sometimes spelled Serafin, or even Selaphin.
//!

pub mod container;
mod parser;
mod variable;

use chrono::NaiveDateTime;
use container::SlfArray2D;
use variable::{SlfVariable, TimeSerie};

pub use parser::{parse, parse_file};

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

    nelem3: usize,
    npoin3: usize,

    /// Number of points per elements
    /// Typically 3 in 2D and 6 in 3D
    npd3: usize,

    /// Number of planes (3D computation)
    nplan: u32,

    /// The connectivity table, size of 'nelem3' * 'npd3'
    /// Indexes of nodes to connect each nodes together (0-based indexes)
    mesh: Vec<u32>,

    /// Indexes of nodes at the boundary (0-based indexes)
    /// The value of an element is 0 for an inner point and yields the edge
    /// point numbers for the others),
    ipob3: Vec<u32>,

    /// Linear variables stored in history results
    var: Vec<SlfVariable>,

    /// Quadratic variables stored in history results
    cld: Vec<SlfVariable>,

    /// Coordinates of each points of the mesh
    points: SlfArray2D,

    results: TimeSerie,

    /// Date & time of creation of the Selafin
    pub datetime: Option<NaiveDateTime>,
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

            nelem3: 0,
            npoin3: 0,
            npd3: 0,
            nplan: 1,
            mesh: vec![],
            ipob3: vec![],

            var: vec![],
            cld: vec![],
            points: SlfArray2D::Float {
                x: vec![],
                y: vec![],
            },
            datetime: None,
            results: TimeSerie::default(),
        }
    }
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

    /// Tell if the Selafin is for 2D of 3D computation
    pub fn dimension(&self) -> u32 {
        if self.nplan > 1 {
            3
        } else {
            2
        }
    }

    /// Number of plane (layer) in  the Selafin file
    /// Always 1 for 2D files, always >= 2 for 3D files
    pub fn planes_cnt(&self) -> u32 {
        self.nplan
    }

    /// Number of points in a 2D plane (layer)
    ///
    /// For 2D selafin, this is the same as [self.points_count]
    /// For 3D Selafin, this is the total number of points ([self.points_count]) divided by the number of
    /// layer as there is the same number of points for each layer.
    pub fn points_per_layer(&self) -> usize {
        if self.nplan > 1 {
            // The number of points is the same for every layer (regular mesh)
            self.points.len() / (self.nplan as usize)
        } else {
            self.points.len()
        }
    }

    /// Alias for [self.points_per_layer] for Telemac compatibility
    pub fn npoin2(&self) -> usize {
        self.points_per_layer()
    }

    /// Total number of points in the mesh
    pub fn points_count(&self) -> usize {
        self.points.len()
    }

    /// Alias for [self.points_count] for Telemac compatibility
    pub fn npoin3(&self) -> usize {
        self.points.len()
    }

    /// Number of triangular in a 2D plane (layer)
    ///
    /// For 2D selafin, this is the same as [self.elements_count]
    /// For 3D Selafin, this is the total number of elements ([self.elements_count]) divided by the number of
    /// layer as there is the same number of element for each layer.
    pub fn elements_per_layer(&self) -> usize {
        if self.nplan > 1 {
            // The number of points is the same for every layer (regular mesh)
            self.nelem3 / (self.nplan as usize - 1)
        } else {
            self.nelem3
        }
    }

    /// Alias for [self.elements_per_layer] for Telemac compatibility
    pub fn nelem2(&self) -> usize {
        self.elements_per_layer()
    }

    /// Total number of element (triangular or prism) or  in the mesh
    pub fn elements_count(&self) -> usize {
        self.nelem3
    }

    /// Number of points per element on a layer
    ///
    /// In 2D, this is the same as the number of points per layer
    /// In 3D, it's half the number of point per element as there is no need
    /// to connect to upper layer
    pub fn point_per_layer_element(&self) -> usize {
        if self.nplan > 1 {
            self.npd3 / 2 // triangular prism: 6 nodes → 3 in 2-D
        } else {
            self.npd3
        }
    }

    /// Alias for [self.point_per_layer_element] for Telemac compatibility
    pub fn npd2(&self) -> usize {
        self.point_per_layer_element()
    }

    pub fn point_per_element(&self) -> usize {
        self.npd3
    }

    /// Return elements of a single layer
    ///
    /// Layer 0 is the bottom layer, layer `self.planes_cnt() - 1` is the upper layer
    /// For 2D selafin, only layer 0 is valid
    pub fn ikle2(&self, layer: usize) -> Option<&[u32]> {
        let points_per_layer = self.elements_per_layer() * self.point_per_layer_element();
        if layer >= self.nplan as usize {
            None
        } else {
            Some(&self.mesh[layer * points_per_layer..(layer + 1) * points_per_layer])
        }
    }

    /// Return all elements of the selafin
    pub fn ikle3<const N: usize>(&self) -> Option<&[u32; N]> {
        self.mesh.as_array()
    }

    /// Return elements of a single layer
    ///
    /// Layer 0 is the bottom layer, layer `self.planes_cnt() - 1` is the upper layer
    /// For 2D selafin, only layer 0 is valid
    pub fn ipob2(&self, layer: usize) -> Option<&[u32]> {
        let points_per_layer = self.elements_per_layer() * self.point_per_layer_element();
        if layer >= self.nplan as usize {
            None
        } else {
            Some(&self.mesh[layer * points_per_layer..(layer + 1) * points_per_layer])
        }
    }

    /// Return all elements of the selafin
    pub fn ipob3<const N: usize>(&self) -> Option<&[u32; N]> {
        self.ipob3.as_array()
    }

    pub fn results(&self) -> &TimeSerie {
        &self.results
    }
}

// cSpell:ignore Selaphin
