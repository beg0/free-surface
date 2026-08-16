// # Integration tests for free_surface::mesh::dtm
mod test_helpers;

use free_surface::mesh::{init_dtm, DTM};
use free_surface::storage::selafin::parse_file;

use test_helpers::telemac_file;

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn dtm_from_selafin() {
    let input_file =
        telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf").expect("can't get input file");

    let slf = parse_file(input_file).expect("Invalid SLF file");
    let _dtm: DTM = init_dtm(slf.into()).expect("error creating DTM");
}
