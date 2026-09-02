//! # Sanity checks on meshes

use fluent_message::FluentMessage;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use crate::aui::diagnostic::collector::Diagnostics;
use crate::math::Point2f;
use crate::storage::selafin::geometry::SlfGeometry;

type Coords2f = (Vec<f64>, Vec<f64>);

// TODO: refine this
/// Minimal distance between two points
const MIN_DISTANCE: f64 = 1e-3;

/// Sanity checks errors
#[derive(Debug, FluentMessage)]
pub enum MeshSanityDiag {
    EmptyPointArray,
    EmptyMesh,
    NullCellSize,
    OddConnectivityTableSize {
        connectivity_table_len: usize,
        point_per_cell: usize,
    },
    DuplicatedNodeInCell {
        cell_idx: usize,
        point_idx: usize,
        #[fluent(display)]
        coords: Point2f,
    },
    PointsTooClose {
        point_a_idx: usize,
        #[fluent(display)]
        coords_a: Point2f,
        point_b_idx: usize,
        #[fluent(display)]
        coords_b: Point2f,
        distance: f64,
    },
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

fn check_points_distance(
    points: &Coords2f,
    min_distance: f64,
    diag: &mut Diagnostics,
    filename: &Option<Arc<Path>>,
) {
    let points_cnt = points.0.len();

    // Let's compare versus the square distance
    // to avoid doing useless sqrt() each time
    let min_distance2 = min_distance * min_distance;

    if points_cnt == 0 {
        diag.error(MeshSanityDiag::EmptyPointArray)
            .with_filename_option(filename);
        return;
    }

    for point_a_idx in 0..points_cnt - 1 {
        let coords_a = get_point(points, point_a_idx);
        for point_b_idx in point_a_idx + 1..points_cnt {
            let coords_b = get_point(points, point_b_idx);
            let d2 = coords_b.distance2(&coords_a);

            if d2 < min_distance2 {
                let distance = d2.sqrt();
                diag.error(MeshSanityDiag::PointsTooClose {
                    point_a_idx,
                    coords_a: coords_a.clone(),
                    point_b_idx,
                    coords_b,
                    distance,
                })
                .with_filename_option(filename);
            }
        }
    }
}

fn check_duplicated_node_in_cell(
    ikle: &[u32],
    point_per_cell: usize,
    points: Coords2f,
    diag: &mut Diagnostics,
    filename: &Option<Arc<Path>>,
) {
    let points_cnt = points.0.len();

    if ikle.is_empty() {
        diag.error(MeshSanityDiag::EmptyMesh)
            .with_filename_option(filename);
    }

    if point_per_cell == 0 {
        diag.error(MeshSanityDiag::NullCellSize)
            .with_filename_option(filename);
        return; // No much we can do if point_per_cell == 0
    }

    if !ikle.len().is_multiple_of(point_per_cell) {
        diag.error(MeshSanityDiag::OddConnectivityTableSize {
            connectivity_table_len: ikle.len(),
            point_per_cell,
        })
        .with_filename_option(filename);
    }

    for (cell_idx, points_in_cell) in ikle.chunks(point_per_cell).enumerate() {
        let mut set: HashSet<usize> = HashSet::with_capacity(point_per_cell);
        for pt_idx in points_in_cell {
            let point_idx = *pt_idx as usize;
            if point_idx >= points_cnt {
                diag.error(MeshSanityDiag::ConnectivityIndexOutOfRange {
                    cell_idx,
                    point_idx,
                    points_cnt,
                })
                .with_filename_option(filename);
                continue;
            }
            let new_value = set.insert(point_idx);
            if !new_value {
                let coords = get_point(&points, point_idx);
                diag.error(MeshSanityDiag::DuplicatedNodeInCell {
                    cell_idx,
                    point_idx,
                    coords,
                })
                .with_filename_option(filename);
            }
        }
    }
}

/// Run some sanity checks on the mesh
pub fn initial_sanity_checks(geometry: &SlfGeometry) -> Result<(), Diagnostics> {
    let mut diag = Diagnostics::default();
    let points = geometry.points_raw().to_vec();

    check_points_distance(&points, MIN_DISTANCE, &mut diag, geometry.filename());

    check_duplicated_node_in_cell(
        geometry.ikle3(),
        geometry.point_per_element(),
        points,
        &mut diag,
        geometry.filename(),
    );

    if diag.has_errors(false) {
        Err(diag)
    } else {
        Ok(())
    }
}
