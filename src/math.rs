//! # Mathematical operations for free-surface
//!

use std::fmt;
pub mod backend;
pub mod bief;

/// Coordinate of a 2D points
#[derive(Clone)]
pub struct Point2f {
    pub x: f64,
    pub y: f64,
}

impl fmt::Display for Point2f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Debug for Point2f {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Point2f {
    /// Compute the distance from another point
    pub fn distance(&self, other: &Point2f) -> f64 {
        self.distance2(other).sqrt()
    }

    /// Compute the square of the distance from another point
    pub fn distance2(&self, other: &Point2f) -> f64 {
        let delta_x = self.x - other.x;
        let delta_y = self.y - other.y;

        delta_x * delta_x + delta_y * delta_y
    }

    pub fn as_point3f(&self) -> Point3f {
        Point3f {
            x: self.x,
            y: self.y,
            z: 0.0,
        }
    }
}

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
