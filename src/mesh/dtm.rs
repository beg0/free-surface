use crate::math::Point3f;
use crate::mesh::neighbors::Neighbors;
use crate::mesh::percellcoords::PerCellCoords;
use crate::storage::selafin::geometry::SlfGeometry;

use super::sanity::initial_sanity_checks;

/// Errors encountered on DTM computation
#[derive(Debug, thiserror::Error)]
pub enum DtmError {
    #[error("Negative or null determinant for cell #{cell_idx}: {coords:?}")]
    NegativeDeterminant {
        cell_idx: usize,
        coords: Vec<Point3f>,
    },
    #[error("Unsupported number of point per cell: {0}")]
    UnsupportedPointPerCellCnt(usize),
}

/// Digital Terrain Model
///
/// Gather all simulation data
pub struct DTM {
    /// The mesh info, as stored in Selafin file
    pub geometry: SlfGeometry,

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

// fn compute_coords_per_cell(geometry: &SlfGeometry) -> Vec<PerCellCoords>
// {
//     let mut coords: Vec<PerCellCoords> = Vec::with_capacity(geometry.elements_count());
//     let (x_coords, y_coords) = geometry.points_raw().to_vec();

//     for points_idx_in_cell in geometry.ikle3.chunks(geometry.point_per_element()) {
//         let coords_this_cell: PerCellCoords = Vec::with_capacity(geometry.point_per_element());
//         for point_idx in points_idx_in_cell {
//             let x = x_coords[point_idx];
//             let y = y_coords[point_idx];
//             let z = 0_f64;

//             coords_this_cell.push(Point3f { x, y, z});
//         }
//     }

// }

fn max_neighbors(ikle: &[usize], ipobo: &[usize]) -> (usize, usize) {
    let MXPTVS: usize = 0;
    let MXELVS: usize = 0;

    (MXPTVS, MXELVS)
}

fn triangle_surface(coords: &[Point3f]) -> f64 {
    let x2 = coords[1].x;
    let x3 = coords[2].x;
    let y2 = coords[1].y;
    let y3 = coords[2].y;

    0.5 * (x2 * y3 - x3 * y2)
}

fn det_inverse_triangle((cell_idx, coords): (usize, &[Point3f])) -> anyhow::Result<f64> {
    let t12 = -coords[0].x + coords[1].x;
    let t13 = -coords[0].x + coords[2].x;
    let t22 = -coords[0].y + coords[1].y;
    let t23 = -coords[0].y + coords[2].y;

    let det = t12 * t23 - t22 * t13;

    if det < 1e-20 {
        Err((DtmError::NegativeDeterminant {
            cell_idx,
            coords: coords.to_vec(),
        })
        .into())
    } else {
        Ok(1.0 / det)
    }
}

fn compute_surface(coords_per_cell: &PerCellCoords) -> anyhow::Result<Vec<f64>> {
    // TODO: check that formula in triangle_surface() is actually ok for prisms (e.g coords_per_cell.point_per_cell==6)
    if coords_per_cell.point_per_cell == 3 || coords_per_cell.point_per_cell == 6 {
        Ok(coords_per_cell.iter().map(triangle_surface).collect())
    } else {
        Err(DtmError::UnsupportedPointPerCellCnt(coords_per_cell.point_per_cell).into())
    }
}

fn compute_det_inverse(coords_per_cell: &PerCellCoords) -> anyhow::Result<Vec<f64>> {
    if coords_per_cell.point_per_cell == 3 {
        coords_per_cell
            .iter()
            .enumerate()
            .map(det_inverse_triangle)
            .collect()
    } else {
        Err(DtmError::UnsupportedPointPerCellCnt(coords_per_cell.point_per_cell).into())
    }
}

/// Create a DTM from a Selafin Geometry
pub fn init_dtm(geometry: SlfGeometry) -> anyhow::Result<DTM> {
    // Ensure we have a descent mesh
    initial_sanity_checks(&geometry)?;

    let coords_per_cell = PerCellCoords::from_selafin(&geometry);
    let surface = compute_surface(&coords_per_cell)?;
    let det_inverse = compute_det_inverse(&coords_per_cell)?;
    let neighbors = Neighbors::from_selafin(&geometry);

    Ok(DTM {
        geometry,
        neighbors,
        coords_per_cell,
        surface,
        det_inverse,
    })
}
