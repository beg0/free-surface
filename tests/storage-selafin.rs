/// # Integration tests for free_surface::storage::selafin
mod test_helpers;

use binrw::Endian;
use std::assert_matches;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use free_surface::storage::selafin::container::SlfArray2D;
use free_surface::storage::selafin::{parse_file, write_file, SlfVariable};

use test_helpers::telemac_file;

fn mk_slf_var(name: &str, unit: &str) -> SlfVariable {
    SlfVariable {
        name: name.to_string(),
        unit: unit.to_string(),
    }
}

fn check_geo_gouttedo(filename: PathBuf) {
    let slf = parse_file(filename).unwrap();
    assert_eq!(slf.title(), "TELEMAC 2D : GOUTTE D'EAU DANS UN BASSIN$");
    assert_eq!(slf.var_defs(), [mk_slf_var("MAILLAGE", "")]);
    assert!(slf.cld_defs().is_empty());
    assert_eq!(slf.nbvar(), 1);
    assert_eq!(slf.results().var_count(), 1);
    assert_eq!(slf.results().step_count(), 1);
    assert!(slf.results().get_var("MAILLAGE").is_some());
    assert!(slf.datetime().is_none());

    let geometry = slf.geometry();
    assert_eq!(geometry.npoin2(), 4624);
    assert_eq!(geometry.nelem2(), 8978);
    assert_eq!(geometry.planes_cnt(), 1);

    assert_matches!(geometry.points_raw(), SlfArray2D::Float { x: _, y: _ });
    let (x, y) = geometry.points_raw().to_vec();
    assert_eq!(x[0], 0.0);
    assert_eq!(y[0], 0.0);

    assert_eq!(x[2623], 14.099995613098145);
    assert_eq!(y[2623], 10.800007820129395);
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn geo_gouttedo() {
    let filename = telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf")
        .expect("Can't get telemac file");
    check_geo_gouttedo(filename);
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn slf_round_trip() {
    let filename = telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf")
        .expect("Can't get telemac file");
    let original_slf = parse_file(filename).unwrap();

    let dest_tmp_file = NamedTempFile::new().expect("Can't create temp dest file");
    write_file(dest_tmp_file.path(), &original_slf, Endian::Big).expect("Write error");
    check_geo_gouttedo(dest_tmp_file.path().to_path_buf());

    write_file(dest_tmp_file.path(), &original_slf, Endian::Little).expect("Write error");
    check_geo_gouttedo(dest_tmp_file.path().to_path_buf());
}

// Spell ignore some french words
// CSpell:ignore GOUTTE D'EAU DANS UN BASSIN
// CSpell:ignore MAILLAGE
