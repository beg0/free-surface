// # Integration tests for slf-dump program
mod test_helpers;

use std::str::FromStr;

use assert_cmd::Command;
use predicates::prelude::*;

use test_helpers::fixture;
use test_helpers::normalize_eol;
use test_helpers::telemac_file;

const SHOW_TOKENS: [&str; 13] = [
    "title",
    "npoints",
    "nelements",
    "nlayers",
    "nplanes",
    "variables",
    "variables+units",
    "points",
    "points:layer=0",
    "elements",
    "elements:layer=0",
    "datetime",
    "results",
];

const DEFAULT_FORMAT: &str = "human";

fn show_with_format(format: &'static str, ext: &'static str) {
    let input_file =
        telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf").expect("can't get input file");

    let fixture_format = if format.is_empty() {
        DEFAULT_FORMAT.to_string()
    } else {
        format.to_owned()
    };

    for token in SHOW_TOKENS {
        let fixture_path =
            fixture(format!("bin-slf-dump/{}/{}.{}", fixture_format, token, ext).as_str());

        let expected_stdout = std::fs::read_to_string(fixture_path.clone())
            .unwrap_or_else(|_| format!("can't read {}", fixture_path.display()));

        let normalized_expected_stdout = normalize_eol(&expected_stdout);

        let mut bin = Command::cargo_bin("slf-dump").unwrap();
        let mut cmd = bin.arg("--show").arg(token);

        if !format.is_empty() {
            cmd = cmd.arg("--format").arg(format);
        }

        cmd = cmd.arg(&input_file);

        cmd.assert()
            .success()
            .stdout(test_helpers::predicates::multiline_nomalized(
                predicate::str::diff(normalized_expected_stdout),
            ))
            .stderr(predicate::str::is_empty());
    }
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn show_default() {
    show_with_format("", "txt");
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn show_human() {
    show_with_format("human", "txt");
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn show_json() {
    show_with_format("json", "json");
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn show_damocles() {
    show_with_format("damocles", "cas");
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn show_machine() {
    show_with_format("machine", "txt");
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn show_json_compact_vs_pretty() {
    let input_file =
        telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf").expect("can't get input file");

    for token in SHOW_TOKENS {
        let compact_cmd_result = Command::cargo_bin("slf-dump")
            .unwrap()
            .arg("--format")
            .arg("json")
            .arg("--compact")
            .arg("--show")
            .arg(token)
            .arg(&input_file)
            .assert()
            .success()
            .stderr(predicate::str::is_empty());

        let compact_output = std::str::from_utf8(&compact_cmd_result.get_output().stdout)
            .expect("slf-dump did not output utf8");

        let pretty_cmd_result = Command::cargo_bin("slf-dump")
            .unwrap()
            .arg("--format")
            .arg("json")
            .arg("--pretty")
            .arg("--show")
            .arg(token)
            .arg(&input_file)
            .assert()
            .success()
            .stderr(predicate::str::is_empty());

        let pretty_output = std::str::from_utf8(&pretty_cmd_result.get_output().stdout)
            .expect("slf-dump did not output utf8");

        assert!(compact_output.len() < pretty_output.len());

        let compact_json_content =
            serde_json::Value::from_str(compact_output).expect("not a valid JSON");

        let pretty_json_content =
            serde_json::Value::from_str(pretty_output).expect("not a valid JSON");

        assert_eq!(compact_json_content, pretty_json_content);
    }
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn fails_on_missing_show_args() {
    let input_file =
        telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf").expect("can't get input file");

    Command::cargo_bin("slf-dump")
        .unwrap()
        .arg(input_file)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("--help"));
}

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn fails_on_bad_show_args() {
    let input_file =
        telemac_file("examples/telemac2d/gouttedo/geo_gouttedo.slf").expect("can't get input file");

    Command::cargo_bin("slf-dump")
        .unwrap()
        .arg("--show")
        .arg("next-lottery-results")
        .arg(input_file)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("--help"));
}

#[test]
fn fails_on_missing_file_args() {
    Command::cargo_bin("slf-dump")
        .unwrap()
        .arg("--show")
        .arg("title")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("--help"));
}
