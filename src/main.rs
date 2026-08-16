use std::process::ExitCode;

use free_surface::mesh::{init_dtm, DTM};
use free_surface::storage::selafin::parse_file;

use serde_json::Value as JsonValue;

fn main() -> ExitCode {
    //println!("Hello, World");
    match parse_file("/home/cca/env/telemac/v8p0r2/examples/telemac2d/gouttedo/geo_gouttedo.slf") {
        Ok(slf) => {
            let dtm: DTM = init_dtm(slf.into()).expect("error creating DTM");

            let cells_neighbors: Vec<Vec<Option<usize>>> = dtm
                .neighbors
                .iter()
                .map(|cell_info| {
                    cell_info
                        .iter()
                        .map(|f| f.clone().map(|ni| ni.cell_idx))
                        .collect()
                })
                .collect();

            //println!("{}", serde_json::to_string_pretty(&JsonValue::from(cells_neighbors)).expect("bad json"));

            println!("{}", JsonValue::from(cells_neighbors));
            // for (cell_idx, neighbors) in dtm.neighbors.iter().enumerate() {
            //     for (edge_idx, neighbor) in neighbors.iter().enumerate() {
            //         let neighbor_cell_idx = match neighbor {
            //             Some(info) => (info.cell_idx + 1) as isize,
            //             None => -1,
            //         };
            //         println!(" IFABOR({cell_idx:4},{edge_idx})={neighbor_cell_idx:4}")
            //     }
            // }
        }
        Err(e) => eprintln!("{}", e),
    }

    ExitCode::SUCCESS
}
