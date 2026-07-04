//! # Helpers to retreive telemac sources
//!
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use glob::Pattern;
use sha2::{Digest, Sha256};
use tar::Archive;

use super::cache_dir;

const TELEMAC_VERSION: &str = "v8p4r0";

/// the SHA256 of the tarball to download
const SHA256: &str = "cfa2c05e41b28ba8aaea19b4b483e6d848bb08f823c8526c0824953a1cb4d564";

/// Glob patterns (relative to the tarball root), matched against archive
/// member paths. Add more here as test data needs grow.
const KNOWN_DESIRED_FILES: &[&str] = &["examples/telemac2d/gouttedo/**", "sources/*/*.dico"];

fn tarball_name() -> String {
    format!("telemac-mascaret-{TELEMAC_VERSION}")
}

fn tarball_url() -> String {
    format!(
        "https://gitlab.pam-retd.fr/otm/telemac-mascaret/-/archive/{TELEMAC_VERSION}/{}.tar.gz",
        tarball_name()
    )
}

/// Where are data being extracted
///
/// # Note on chosen path
///
/// The path contains the sha256 of the tarball, that way if the tarball
/// get updated, the content will go in a new directory.
/// This will prevent a file  with the same path as in an old tarball
/// to be re-used.
/// Please not that today, there is no mechanism to clean previous tarball content.
///
fn extract_dir() -> PathBuf {
    cache_dir().join("extracted").join(SHA256)
}

/// Download a file from an URL and save to disk
fn download_to_disk(uri: &String, dest: &PathBuf) -> anyhow::Result<()> {
    let mut response = ureq::get(uri).call()?;
    let mut file = File::create(dest)?;
    io::copy(&mut response.body_mut().as_reader(), &mut file)?;
    file.sync_all()?;
    drop(file);
    Ok(())
}

/// Downloads the tarball only if it's missing or its checksum doesn't
/// match what we expect; otherwise reuses the cached copy untouched.
fn ensure_tarball_cached() -> anyhow::Result<(PathBuf, bool)> {
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.tar.gz", tarball_name()));

    eprintln!("check sha256 on {}", path.display());
    if path.exists() && sha256_file(&path)? == SHA256 {
        return Ok((path, true)); // already cached and verified
    }

    eprintln!("download {}", tarball_url());

    if true {
        download_to_disk(&tarball_url(), &path)?;
    } else {
        std::fs::copy(
            Path::new(env!("HOME")).join("Downloads/telemac-mascaret-v8p4r0.tar.gz"),
            path.clone(),
        )?;
    }

    eprintln!("verify integrity of {}", path.display());

    let actual = sha256_file(&path)?;
    anyhow::ensure!(
        actual == SHA256,
        "checksum mismatch for downloaded tarball: expected {SHA256}, got {actual}"
    );

    Ok((path, false))
}

fn telemac_sources_unlocked(relative_path: &str) -> anyhow::Result<PathBuf> {
    let tarball_name = tarball_name();
    let extract_dir = extract_dir();

    let fixture_path = extract_dir.join(&tarball_name).join(relative_path);

    if !fixture_path.exists() {
        let (tarball_path, existing_tarball) = ensure_tarball_cached()?;

        // If we had to re-download the tarball, maybe something wrong was
        // extracted last time. Try to clean up (but ignore errors)
        //
        // FIXME: shall I remove the content of `extract_dir.parent()`
        // so that the content of an old tarball is removed too?
        // (see [extract_dir()] for details)
        if !existing_tarball {
            let _ = std::fs::remove_dir_all(&extract_dir);
        }

        // If it's a fresh download (or a re-download due to corrupted tarball)
        // it's better to also extract known files. Thus will we open the tarball
        // only one and thus gain some time.
        let requested_files: &[&str] = if !existing_tarball {
            &[KNOWN_DESIRED_FILES, &[relative_path]].concat()
        } else {
            &[relative_path]
        };

        std::fs::create_dir_all(&extract_dir)?;

        extract_subset(&tarball_path, &extract_dir, &tarball_name, requested_files)?;

        // Be sure the requested file was extracted
        // Indeed as extract_subset() works with pattern matching, if no file path matches
        // the pattern, then nothing is extracted and no error reported
        anyhow::ensure!(
            fixture_path.exists(),
            format!(
                "File {relative_path} not found in {}",
                tarball_path.display()
            )
        );
    }

    Ok(fixture_path)
}

/// Get the location of a file in the telemac source tree
///
/// This function will download and extract the telemac source tree if needed.
pub fn telemac_file(relative_path: &str) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(cache_dir())?;

    // Create a lock-file so that multiple processes don't try to download & extract
    // the same files
    let lock_path = cache_dir().join("telemac-dl-lock");
    let lock_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&lock_path)?;

    lock_file.lock()?;
    let result = telemac_sources_unlocked(relative_path);
    fs4::FileExt::unlock(&lock_file)?;

    result
}

/// Compute sha256 on a file
fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Extracts from `tarball` the files matching `subset` into `dest_dir`.
fn extract_subset(
    tarball: &PathBuf,
    dest_dir: &PathBuf,
    prefix: &str,
    subset: &[&str],
) -> anyhow::Result<()> {
    let patterns: Vec<Pattern> = subset
        .iter()
        .map(|p| Pattern::new(p))
        .collect::<Result<_, _>>()?;

    let file = File::open(tarball)?;
    let mut archive = Archive::new(GzDecoder::new(file));

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        let candidate = if !prefix.is_empty() {
            match path.strip_prefix(prefix) {
                Ok(relative) => relative,
                Err(_) => continue,
            }
        } else {
            &path
        };

        if patterns.iter().any(|p| p.matches_path(candidate)) {
            entry.unpack_in(dest_dir)?;
        }
    }

    Ok(())
}
