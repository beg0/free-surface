//! # Sanity checks on meshes

use std::collections::HashSet;

use crate::math::Point2f;
use crate::storage::selafin::geometry::SlfGeometry;

type Coords2f = (Vec<f64>, Vec<f64>);

// TODO: refine this
/// Minimal distance between two points
const MIN_DISTANCE: f64 = 1e-3;

/// Sanity checks errors
#[derive(Debug, thiserror::Error)]
pub enum MeshSanityError {
    #[error("Empty set of points")]
    EmptyPointArray,
    #[error("Empty mesh")]
    EmptyMesh,
    #[error("Cell size is null")]
    NullCellSize,
    #[error("Connectivity table size ({connectivity_table_len}) is not a multiple of cell size ({point_per_cell})")]
    OddConnectivityTableSize {
        connectivity_table_len: usize,
        point_per_cell: usize,
    },
    #[error("Duplicated node #{point_idx} ({coords}) in the same cell #{cell_idx}")]
    DuplicatedNodeInCell {
        cell_idx: usize,
        point_idx: usize,
        coords: Point2f,
    },
    #[error("Point #{point_a_idx} ({coords_a}) and point #{point_b_idx} ({coords_b}) are too closes: {distance}")]
    PointsTooClose {
        point_a_idx: usize,
        coords_a: Point2f,
        point_b_idx: usize,
        coords_b: Point2f,
        distance: f64,
    },
    #[error("Connectivity index out-of-range: got point #{point_idx} in cell #{cell_idx} but there is only {points_cnt}")]
    ConnectivityIndexOutOfRange {
        cell_idx: usize,
        point_idx: usize,
        points_cnt: usize,
    },
}

fn get_point(points: &Coords2f, idx: usize) -> Point2f {
    Point2f {
        x: points.0[idx],
        y: points.1[idx],
    }
}

fn check_points_distance(points: &Coords2f, min_distance: f64) -> Vec<MeshSanityError> {
    let mut errors: Vec<MeshSanityError> = Vec::new();
    let points_cnt = points.0.len();

    // Let's compare versus the square distance
    // to avoid doing useless sqrt() each time
    let min_distance2 = min_distance * min_distance;

    if points_cnt == 0 {
        errors.push(MeshSanityError::EmptyPointArray);
        return errors;
    }

    for point_a_idx in 0..points_cnt - 1 {
        let coords_a = get_point(points, point_a_idx);
        for point_b_idx in point_a_idx + 1..points_cnt {
            let coords_b = get_point(points, point_b_idx);
            let d2 = coords_b.distance2(&coords_a);

            if d2 < min_distance2 {
                let distance = d2.sqrt();
                errors.push(MeshSanityError::PointsTooClose {
                    point_a_idx,
                    coords_a: coords_a.clone(),
                    point_b_idx,
                    coords_b,
                    distance,
                })
            }
        }
    }

    errors
}

fn check_duplicated_node_in_cell(
    ikle: &[u32],
    point_per_cell: usize,
    points: Coords2f,
) -> Vec<MeshSanityError> {
    let mut errors: Vec<MeshSanityError> = Vec::new();
    let points_cnt = points.0.len();

    if ikle.is_empty() {
        errors.push(MeshSanityError::EmptyMesh);
    }

    if point_per_cell == 0 {
        errors.push(MeshSanityError::NullCellSize);
        return errors; // No much we can do if point_per_cell == 0
    }

    if ikle.len().is_multiple_of(point_per_cell) {
        errors.push(MeshSanityError::OddConnectivityTableSize {
            connectivity_table_len: ikle.len(),
            point_per_cell,
        });
    }

    for (cell_idx, points_in_cell) in ikle.chunks(point_per_cell).enumerate() {
        let mut set: HashSet<usize> = HashSet::with_capacity(point_per_cell);
        for pt_idx in points_in_cell {
            let point_idx = *pt_idx as usize;
            if point_idx >= points_cnt {
                errors.push(MeshSanityError::ConnectivityIndexOutOfRange {
                    cell_idx,
                    point_idx,
                    points_cnt,
                });
                continue;
            }
            let new_value = set.insert(point_idx);
            if !new_value {
                let coords = get_point(&points, point_idx);
                errors.push(MeshSanityError::DuplicatedNodeInCell {
                    cell_idx,
                    point_idx,
                    coords,
                });
            }
        }
    }

    errors
}

/// Run some sanity checks on the mesh
pub fn initial_sanity_checks(geometry: &SlfGeometry) -> anyhow::Result<()> {
    let mut errors: Vec<MeshSanityError> = Vec::new();
    let points = geometry.points_raw().to_vec();

    let mut point_distance_errors = check_points_distance(&points, MIN_DISTANCE);

    errors.append(&mut point_distance_errors);

    let mut duplicated_node_in_cell =
        check_duplicated_node_in_cell(geometry.ikle3(), geometry.point_per_element(), points);

    errors.append(&mut duplicated_node_in_cell);

    // if !errors.is_empty() {
    //     Err(errors.into())
    // } else {
    Ok(())
    // }
}
