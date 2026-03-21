//! File-system utilities functions
//!

use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum FindFileError {
    #[error("File '{filename}' not found in any of the searched directories")]
    NotFound { filename: String },
    #[error("I/O error while searching '{dir}': {source}")]
    Io {
        dir: PathBuf,
        source: std::io::Error,
    },
}

pub fn find_file_in_dirs(
    filename: &str,
    dirs: &[impl AsRef<Path>],
) -> Result<PathBuf, FindFileError> {
    for dir in dirs {
        if let Some(found) = find_recursive(filename, dir.as_ref())? {
            return Ok(found);
        }
    }
    Err(FindFileError::NotFound {
        filename: filename.to_string(),
    })
}

fn find_recursive(filename: &str, dir: &Path) -> Result<Option<PathBuf>, FindFileError> {
    let entries = std::fs::read_dir(dir).map_err(|e| FindFileError::Io {
        dir: dir.to_path_buf(),
        source: e,
    })?;

    let mut subdirs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| FindFileError::Io {
            dir: dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();

        if path.is_dir() {
            // Collect subdirs and search them after files in the current dir
            subdirs.push(path);
        } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            return Ok(Some(path));
        }
    }

    for subdir in subdirs {
        if let Some(found) = find_recursive(filename, &subdir)? {
            return Ok(Some(found));
        }
    }

    Ok(None)
}
