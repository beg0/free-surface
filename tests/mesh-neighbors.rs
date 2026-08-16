// # Integration tests for free_surface::mesh::Neighbors

mod test_helpers;

use free_surface::mesh::celliterator::CellData;
use free_surface::mesh::neighbors::Neighbors;

use free_surface::storage::selafin::geometry::SlfGeometry;
use free_surface::storage::selafin::parse_file;

use test_helpers::fixture;
use test_helpers::telemac_file;

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn from_selafin() {
    let input_file =
        telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf").expect("can't get input file");

    let expected_file = fixture("geo_gouttedo.neighbors.json");
    let expected_raw = std::fs::read_to_string(expected_file).expect("missing expected file");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected_raw).expect("fixture is not valid json");

    let expected_cells_neighbors: &Vec<serde_json::Value> = expected_json
        .as_array()
        .expect("expected content is not an array");

    let slf = parse_file(input_file).expect("Invalid SLF file");
    let geometry: SlfGeometry = slf.into();

    let neighbors = Neighbors::from_selafin(&geometry);

    assert_eq!(
        expected_cells_neighbors.len(),
        neighbors.data_len() / neighbors.point_per_cell()
    );

    for (cell_idx, neighbors) in neighbors.iter().enumerate() {
        let expected_neighbors_this_cell_json = &expected_cells_neighbors[cell_idx];
        let expected_neighbors_this_cell = expected_neighbors_this_cell_json
            .as_array()
            .unwrap_or_else(|| panic!("entry #{} in expected fixture is not an array", cell_idx));

        assert_eq!(expected_neighbors_this_cell.len(), neighbors.len());

        for (edge_idx, neighbor) in neighbors.iter().enumerate() {
            let expected_neighbors = &expected_neighbors_this_cell[edge_idx];

            match expected_neighbors {
                serde_json::Value::Null => assert!(neighbor.is_none()),
                serde_json::Value::Number(expected_cell_idx) => {
                    let neighbor_info = neighbor.clone().unwrap_or_else(|| {
                        panic!("edge {} of cell {} shall not be NULL.", edge_idx, cell_idx)
                    });
                    assert_eq!(
                        neighbor_info.cell_idx as u64,
                        expected_cell_idx.as_u64().unwrap()
                    );
                }
                _ => panic!("Unexpected type for neighbors info"),
            }
        }
    }
}
