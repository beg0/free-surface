//! # Basic mathematical objects for linear algebra
//!

use crate::mesh::{neighbors::Neighbors, percellcoords::PerCellCoords};
// use nalgebra::{
//     base::{DMatrix, DVector},
//     Scalar,
// };

// enum InnerBiefObject<T> {
//     Vector {
//         values: DVector<T>,
//         errors: DVector<T>,
//     },
//     Matrix {
//         values: DMatrix<T>,
//         errors: DMatrix<T>,
//     },
// }
// pub struct BiefObject<T> {
//     name: String,
//     parent: String,
//     obj: InnerBiefObject<T>,
// }

// impl<T: Scalar> BiefObject<T> {
//     pub fn new(name: String, parent: String) -> Self {
//         Self {
//             name,
//             parent,
//             obj: InnerBiefObject::Vector {
//                 values: DVector::<T>::default(),
//                 errors: DVector::<T>::default(),
//             },
//         }
//     }

//     pub fn name(&self) -> &String {
//         &self.name
//     }
// }

#[allow(dead_code)]
enum CellSize {
    Triangle = 11, // 3 elements per cell
    Quadrilateral = 21,
    Tetrahedra = 31,              // for 3D, 4 elements per cell
    Prism = 41,                   // For 3D, 6 elements per cell
    PrismsCutIntoTetrahedra = 51, // For 3D, 6 elements per cell?
}

/// Digital Terrain Model
///
/// Gather all simulation data
pub struct DTM {
    /// The mesh info, as stored in Selafin file
    //pub geometry: SlfGeometry,

    /// Neighbors cell of each edge of each cell
    pub neighbors: Neighbors,

    /// Coordinates of the vertices of each cell
    ///
    /// E.g. there is one entry per cell, for each cell, there are `n` coordinates.
    /// `n` is the same for every cells and is the number of points per cell. See `geometry.npd3`.
    pub coords_per_cell: PerCellCoords,

    /// surface (or volume) of each cell
    ///
    /// Note: This is constant over all the simulation, thus it is computed once at the begining of the simulation.
    pub surface: Vec<f64>,

    /// 1/det(cell) for each cell
    ///
    /// Note: This is constant over all the simulation, thus it is computed once at the begining of the simulation.
    pub det_inverse: Vec<f64>,
}
