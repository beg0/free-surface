//! # Mathematical operations for free-surface
//!

use std::fmt;
pub mod backend;
pub mod bief;

/// Coordinate of a 3D points
#[derive(Clone)]
pub struct Point3f {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl fmt::Display for Point3f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl fmt::Debug for Point3f {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
