/// # Integration tests for free_surface::storage::selafin
mod test_helpers;

use binrw::Endian;
use free_surface::set_locale;
use std::assert_matches;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use free_surface::storage::selafin::container::SlfArray2D;
use free_surface::storage::selafin::{parse_file, write_file, SlfVariable};

use test_helpers::{fixture, telemac_file};

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

/// Helper to extract error from files in `tests/fixtures/slf/`
fn check_bad_slf(filename: PathBuf) -> (u64, String) {
    let Err(error) = parse_file(&filename) else {
        panic!("Expect error parsing file {}", filename.display());
    };

    let binrw::Error::AssertFail { pos, message } = error else {
        panic!("Don't get AssertFail but {}", error);
    };

    (pos, message)
}
#[test]
fn slf_with_bad_endianness() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_bad_endianness.slf"));

    assert_eq!(pos, 0);
    assert_eq!(
        error_msg,
        "Cannot determine file endianness from the first record"
    );
}

#[test]
fn slf_with_empty_entry_for_history() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_empty_entry_for_history.slf"));

    assert_eq!(pos, 348);
    assert_eq!(
        error_msg,
        "values of Variable 'NOTHING' value records for history entry #1 has 0 values, expected 3"
    );
}
#[test]
fn slf_with_inconsistent_plan_cnt() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_inconsistent_plan-cnt.slf"));

    assert_eq!(pos, 216);
    assert_eq!(
        error_msg,
        "Inconsistent number of planes, IPARAMS says 2, geometry says 1."
    );
}

#[test]
fn slf_with_too_large_ipob3() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_too_large_ipob3.slf"));

    assert_eq!(pos, 260);
    assert_eq!(
        error_msg,
        "\"boundary table\" record has 4 values, expected 3"
    );
}

#[test]
fn slf_with_truncated_var_record() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_truncated_var_record.slf"));

    assert_eq!(pos, 119);
    assert_eq!(
        error_msg,
        "\"variable\" record truncated. Got 7 bytes and expected at least 32 bytes"
    );
}

#[test]
fn slf_with_bad_float_size() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_bad_float_size.slf"));

    assert_eq!(pos, 256);
    assert_eq!(error_msg,  "Cannot determine float size: record is 16 bytes, expected 12 (f32) or 24 (f64) for npoin3=3");
}

#[test]
fn slf_with_empty_time_history_record() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_empty_time_history_record.slf"));

    assert_eq!(pos, 336);
    assert_eq!(error_msg, "Time entry for history #1 is empty");
}

#[test]
fn slf_with_short_title() {
    let slf = parse_file(fixture("slf/slf_with_short_title.slf"))
        .expect("Should be able to parse a slf with title <= 80 char");

    assert_eq!(slf.title(), "SLF WITH SMALL TITLE");
}

#[test]
fn slf_with_truncated_iparams() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_truncated_iparams.slf"));

    assert_eq!(pos, 188);
    assert_eq!(error_msg, "\"IPARAMS\" record has 9 values, expected 10");
}

// #[test]
// fn small_slf() {
//     let (pos, error_msg) = check_bad_slf(fixture("slf/small_slf.slf"));
// }

#[test]
fn slf_with_bad_record() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_bad_record.slf"));

    assert_eq!(pos, 104);
    assert_eq!(error_msg, "Record length mismatch: header=8 tailer=9");
}

#[test]
fn slf_with_inconsistent_coords_vector() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_inconsistent_coords_vector.slf"));

    assert_eq!(pos, 300);
    assert_eq!(error_msg, "Inconsistent number of points. Header says 3 points, x record says 3 points, y records says 4");
}

#[test]
fn slf_with_too_large_ikle3() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_too_large_ikle3.slf"));

    assert_eq!(pos, 240);
    assert_eq!(
        error_msg,
        "\"connectivity table\" record has 4 values, expected 3"
    );
}

#[test]
fn slf_with_truncated_var_cnt() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let (pos, error_msg) = check_bad_slf(fixture("slf/slf_with_truncated_var_cnt.slf"));

    assert_eq!(pos, 100);
    assert_eq!(error_msg, "\"variable count\" record truncated. Got 1 word of 32-bits and expected at least 2 words of 32-bits");
}

// Spell ignore some french words
// CSpell:ignore GOUTTE D'EAU DANS UN BASSIN
// CSpell:ignore MAILLAGE
