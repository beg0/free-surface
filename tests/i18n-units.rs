// # Integration tests for the UNITS() and WITH_UNITS() function in flt files
use free_surface::{set_locale, t};

#[test]
fn with_units() {
    set_locale(Some("C".into())).expect("LANG=C failed");

    let str_with_bytes = t!("selafin-parser-diag-truncated-record", { "record_name" => "entry1", "got" => 1, "expected" => 2, "unit" => "byte" });

    assert_eq!(
        str_with_bytes,
        "\"entry1\" record truncated. Got 1 byte and expected at least 2 bytes"
    );
}
