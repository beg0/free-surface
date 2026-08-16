use super::celliterator::{CellData, CellIterator};
use crate::math::Point3f;
use crate::storage::selafin::geometry::SlfGeometry;

/// Get coordinates of points of each cell
pub struct PerCellCoords {
    pub points: Vec<Point3f>,
    pub point_per_cell: usize,
}

pub type PerCellCoordsIterator<'a> = CellIterator<'a, PerCellCoords>;

impl PerCellCoords {
    pub fn iter<'a>(&'a self) -> PerCellCoordsIterator<'a> {
        PerCellCoordsIterator::new(self)
    }

    pub fn from_selafin(geometry: &SlfGeometry) -> Self {
        let (x_coords, y_coords) = geometry.points_raw().to_vec();

        let points: Vec<Point3f> = geometry
            .ikle3()
            .iter()
            .map(|point_idx| Point3f {
                x: x_coords[*point_idx as usize],
                y: y_coords[*point_idx as usize],
                z: 0.0,
            })
            .collect();

        Self {
            points,
            point_per_cell: geometry.point_per_element(),
        }
    }
}

impl CellData for PerCellCoords {
    type Item = Point3f;

    fn point_per_cell(&self) -> usize {
        self.point_per_cell
    }

    fn data_len(&self) -> usize {
        self.points.len()
    }

    fn point_data(&self, range: std::ops::Range<usize>) -> &[Point3f] {
        &self.points[range]
    }
}
