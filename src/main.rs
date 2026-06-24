use std::process::ExitCode;

use free_surface::math::bief::DTM;
use free_surface::math::Point3f;
use free_surface::mesh::neighbors::Neighbors;
use free_surface::mesh::percellcoords::PerCellCoords;
use free_surface::storage::selafin::geometry::SlfGeometry;
use free_surface::storage::selafin::parse_file;

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

fn triangle_surface(coords: &[Point3f]) -> f64 {
    let x2 = coords[1].x;
    let x3 = coords[2].x;
    let y2 = coords[1].y;
    let y3 = coords[2].y;

    0.5 * (x2 * y3 - x3 * y2)
}

fn det_inverse_triangle(coords: &[Point3f]) -> Result<f64, String> {
    let t12 = -coords[0].x + coords[1].x;
    let t13 = -coords[0].x + coords[2].x;
    let t22 = -coords[0].y + coords[1].y;
    let t23 = -coords[0].y + coords[2].y;

    let det = t12 * t23 - t22 * t13;

    if det < 1e-20 {
        Err(String::from("Negative or null determinant"))
    } else {
        Ok(1.0 / det)
    }
}

fn compute_surface(coords_per_cell: &PerCellCoords) -> Vec<f64> {
    // TODO: check coords_per_cell.point_per_cell
    // the formula in triangle_surface looks to be ok for triangle (e.g. coords_per_cell.point_per_cell==3)
    // but also for prisms (coords_per_cell.point_per_cell==6)
    coords_per_cell.iter().map(triangle_surface).collect()
}

fn compute_det_inverse(coords_per_cell: &PerCellCoords) -> Result<Vec<f64>, String> {
    if coords_per_cell.point_per_cell == 3 {
        coords_per_cell.iter().map(det_inverse_triangle).collect()
    } else {
        Ok(Vec::new())
    }
}
fn init_dtm(geometry: &SlfGeometry) -> DTM {
    //let geometry = slf.geometry();
    let coords_per_cell = PerCellCoords::from_selafin(geometry);
    let surface = compute_surface(&coords_per_cell);
    let det_inverse = compute_det_inverse(&coords_per_cell).expect("bad mesh");
    let neighbors = Neighbors::from_selafin(geometry);

    DTM {
        //geometry,
        neighbors,
        coords_per_cell,
        surface,
        det_inverse,
    }
}

fn main() -> ExitCode {
    //println!("Hello, World");
    match parse_file("/home/cca/env/telemac/v8p0r2/examples/telemac2d/gouttedo/geo_gouttedo.slf") {
        Ok(slf) => {
            let dtm = init_dtm(slf.geometry());
            for (cell_idx, neighbors) in dtm.neighbors.iter().enumerate() {
                for (edge_idx, neighbor) in neighbors.iter().enumerate() {
                    let neighbor_cell_idx = match neighbor {
                        Some(info) => (info.cell_idx + 1) as isize,
                        None => -1,
                    };
                    println!(" IFABOR({cell_idx:4},{edge_idx})={neighbor_cell_idx:4}")
                }
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    ExitCode::SUCCESS
}
