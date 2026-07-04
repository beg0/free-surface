//! # Test helpers
//!
//! Collection of helpers to be used in integration tests.
use std::path::{Path, PathBuf};

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
