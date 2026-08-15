//! # Test helpers
//!
//! Collection of helpers to be used in integration tests.
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

pub mod predicates;
mod telemac_sources;

/// Get location of the (test) cache directory
#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(".cache/test-data")
}

/// Get the location of a fixture asset
#[allow(dead_code)]
pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[allow(dead_code)]
pub use telemac_sources::telemac_file;

/// Read a file line by line
///
/// # Example
///
/// ```no_run
///
/// let lines = read_lines("my_file.txt").except("Can't read file");
///
/// for line in lines.map_while(Result::ok) {
///   println!(line);
/// }
///
/// ```
#[allow(dead_code)]
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Normalize end of lines
///
/// Input line may end with \n (unix style) or \r\n (dos style)
/// Output every line end with \n (unix style)
///
/// Also trim end of line for tailing whitespaces to ease comparison
#[allow(dead_code)]
pub fn normalize_eol(txt: &str) -> String {
    let mut output = String::with_capacity(txt.len());

    for line in txt.lines() {
        output.push_str(line.trim_end());
        output.push('\n')
    }

    output
}
