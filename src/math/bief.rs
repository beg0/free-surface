//! # Basic mathematical objects for linear algebra
//!

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
