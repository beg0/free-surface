// # Integration tests for check-config program
mod test_helpers;

use std::str::FromStr;

use assert_cmd::Command;
use predicates::prelude::*;

use free_surface::config::casfile;
use free_surface::config::dicofile;
use serde_json_path::JsonPath;
use test_helpers::fixture;
use test_helpers::telemac_file;

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn basic_check() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");

    let input_file =
        telemac_file("examples/telemac2d/gouttedo/t2d_gouttedo.cas").expect("can't get input file");
    Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(dico_path)
        .arg(input_file)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn dump_json() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");

    let input_file =
        telemac_file("examples/telemac2d/gouttedo/t2d_gouttedo.cas").expect("can't get input file");
    let cmd_result = Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(dico_path)
        .arg("--dump")
        .arg("--format")
        .arg("json")
        .arg(input_file)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let output = cmd_result.get_output();
    let output_str = std::str::from_utf8(&output.stdout).expect("check-config did not output utf8");

    let json_content = serde_json::Value::from_str(output_str).expect("not a valid JSON");

    //dbg!(&json_content);

    let initial_condition_query =
        JsonPath::parse("$['COMPUTATION ENVIRONMENT']['INITIALIZATION']['INITIAL CONDITIONS']")
            .expect("invalid JSONPath query");
    let initial_condition_content = initial_condition_query
        .query(&json_content)
        .exactly_one()
        .expect("should have only one value");

    assert_eq!(*initial_condition_content, serde_json::json!("PARTICULAR"));
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn dump_json_compact_vs_pretty() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");

    let input_file =
        telemac_file("examples/telemac2d/gouttedo/t2d_gouttedo.cas").expect("can't get input file");
    let compact_cmd_result = Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(&dico_path)
        .arg("--dump")
        .arg("--format")
        .arg("json")
        .arg("--compact")
        .arg(&input_file)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let compact_output = std::str::from_utf8(&compact_cmd_result.get_output().stdout)
        .expect("check-config did not output utf8");

    let pretty_cmd_result = Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(&dico_path)
        .arg("--dump")
        .arg("--format")
        .arg("json")
        .arg("--pretty")
        .arg(&input_file)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let pretty_output = std::str::from_utf8(&pretty_cmd_result.get_output().stdout)
        .expect("check-config did not output utf8");

    assert!(compact_output.len() < pretty_output.len());

    let compact_json_content =
        serde_json::Value::from_str(compact_output).expect("not a valid JSON");

    let pretty_json_content = serde_json::Value::from_str(pretty_output).expect("not a valid JSON");

    assert_eq!(compact_json_content, pretty_json_content);
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn dump_damocles() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");

    let input_file =
        telemac_file("examples/telemac2d/gouttedo/t2d_gouttedo.cas").expect("can't get input file");
    let cmd_result = Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(&dico_path)
        .arg("--dump")
        .arg("--format")
        .arg("damocles")
        .arg(&input_file)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let output = cmd_result.get_output();
    let output_str = std::str::from_utf8(&output.stdout).expect("check-config did not output utf8");

    let dico = dicofile::parse_file(dico_path).expect("bad dico");
    let parser = casfile::Parser::new(&dico);

    let original = parser
        .parse_file(input_file)
        .expect("parse error for t2d_gouttedo.cas");
    let dumped = parser
        .parse(output_str)
        .expect("parse error for output of check-config");

    // Check round-trip is ok
    assert_eq!(original, dumped);
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn dump_damocles_full_dump() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");

    let input_file =
        telemac_file("examples/telemac2d/gouttedo/t2d_gouttedo.cas").expect("can't get input file");
    let cmd_result = Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(&dico_path)
        .arg("--full-dump")
        .arg("--format")
        .arg("damocles")
        .arg(&input_file)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let output = cmd_result.get_output();
    let output_str = std::str::from_utf8(&output.stdout).expect("check-config did not output utf8");

    let dico = dicofile::parse_file(dico_path).expect("bad dico");
    let parser = casfile::Parser::new(&dico);

    // Get the *full* config from the original cas file
    let original = parser
        .config_from_file(input_file)
        .expect("parse error for t2d_gouttedo.cas");

    // Get "only" the key/values pairs dumped by check-config
    // but it should be the full config as we requested so.
    let mut dumped = parser
        .parse(output_str)
        .expect("parse error for output of check-config");

    // Check round-trip is ok
    // we should have the same keys & values in both HashMap
    // except for empty lists
    for (name, original_value) in original.iter() {
        let dumped_value = dumped.remove(name);

        // There is no way to dump an empty list in damocles
        // Thus the value is not
        if original_value.is_empty() {
            assert!(dumped_value.is_none());
        } else {
            let v = dumped_value
                .unwrap_or_else(|| panic!("missing key {} in output of check-config", name));
            assert_eq!(v, *original_value);
        }
    }

    // We should have remove every keys, otherwise it means check-config added some w.r.t. the default one.
    assert!(dumped.is_empty());
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn dump_human() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");

    let input_file =
        telemac_file("examples/telemac2d/gouttedo/t2d_gouttedo.cas").expect("can't get input file");
    Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(dico_path)
        .arg("--dump")
        .arg("--format")
        .arg("human")
        .arg(input_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("[INPUT FILES]"))
        .stdout(predicate::str::contains(
            "TITLE: TELEMAC 2D: DROPLET IN A BASIN",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn bad_keyword() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");

    let fixture_name = "bad_keyword.cas";
    let input_file = fixture(fixture_name);
    Command::cargo_bin("check-config")
        .unwrap()
        .arg("--dico")
        .arg(dico_path)
        .arg(&input_file)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Unknown key: 'UNKNOWN'"))
        .stderr(predicate::str::contains(fixture_name));
}

#[test]
fn fails_on_missing_arg() {
    Command::cargo_bin("check-config")
        .unwrap()
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("--help"));
}
