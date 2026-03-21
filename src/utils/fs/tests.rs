use super::*;
use std::fs;
use tempfile::TempDir;

// --- Helpers ---

fn make_dir_with_files(files: &[&str]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for f in files {
        fs::write(dir.path().join(f), "").unwrap();
    }
    dir
}

/// Create a nested structure under a TempDir:
/// dirs is a list of subdirectory paths to create (relative to root),
/// files is a list of (relative_dir, filename) pairs.
fn make_nested_dir(dirs: &[&str], files: &[(&str, &str)]) -> TempDir {
    let root = TempDir::new().unwrap();
    for d in dirs {
        fs::create_dir_all(root.path().join(d)).unwrap();
    }
    for (d, f) in files {
        fs::write(root.path().join(d).join(f), "").unwrap();
    }
    root
}

// =========================================================
// Flat (non-recursive) - existing behavior preserved
// =========================================================

#[test]
fn test_finds_file_in_single_dir() {
    let dir = make_dir_with_files(&["config.toml"]);
    let result = find_file_in_dirs("config.toml", &[dir.path()]).unwrap();
    assert_eq!(result, dir.path().join("config.toml"));
}

#[test]
fn test_finds_file_in_first_dir() {
    let dir1 = make_dir_with_files(&["config.toml"]);
    let dir2 = make_dir_with_files(&["other.toml"]);
    let result = find_file_in_dirs("config.toml", &[dir1.path(), dir2.path()]).unwrap();
    assert_eq!(result, dir1.path().join("config.toml"));
}

#[test]
fn test_finds_file_in_second_dir_when_not_in_first() {
    let dir1 = make_dir_with_files(&[]);
    let dir2 = make_dir_with_files(&["config.toml"]);
    let result = find_file_in_dirs("config.toml", &[dir1.path(), dir2.path()]).unwrap();
    assert_eq!(result, dir2.path().join("config.toml"));
}

#[test]
fn test_not_found_in_empty_dir_list() {
    let result = find_file_in_dirs("config.toml", &[] as &[&Path]);
    assert!(matches!(result, Err(FindFileError::NotFound { .. })));
}

#[test]
fn test_not_found_error_contains_filename() {
    let dir = make_dir_with_files(&[]);
    match find_file_in_dirs("config.toml", &[dir.path()]) {
        Err(FindFileError::NotFound { filename }) => assert_eq!(filename, "config.toml"),
        _ => panic!("expected NotFound error"),
    }
}

// =========================================================
// Recursive search
// =========================================================

#[test]
fn test_finds_file_one_level_deep() {
    let root = make_nested_dir(&["sub"], &[("sub", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]).unwrap();
    assert_eq!(result, root.path().join("sub").join("config.toml"));
}

#[test]
fn test_finds_file_two_levels_deep() {
    let root = make_nested_dir(&["a/b"], &[("a/b", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]).unwrap();
    assert_eq!(result, root.path().join("a/b/config.toml"));
}

#[test]
fn test_finds_file_deeply_nested() {
    let root = make_nested_dir(&["a/b/c/d/e"], &[("a/b/c/d/e", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]).unwrap();
    assert_eq!(result, root.path().join("a/b/c/d/e/config.toml"));
}

#[test]
fn test_prefers_shallow_over_deep() {
    // File exists at root level and also in a subdir — root wins
    let root = make_nested_dir(&["sub"], &[("", "config.toml"), ("sub", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]).unwrap();
    assert_eq!(result, root.path().join("config.toml"));
}

#[test]
fn test_finds_file_in_sibling_subdir() {
    // File is not in first subdir but is in the second
    let root = make_nested_dir(&["a", "b"], &[("b", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]).unwrap();
    assert_eq!(result, root.path().join("b").join("config.toml"));
}

#[test]
fn test_not_found_when_only_wrong_files_exist() {
    let root = make_nested_dir(&["a/b"], &[("a", "other.toml"), ("a/b", "another.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]);
    assert!(matches!(result, Err(FindFileError::NotFound { .. })));
}

#[test]
fn test_not_found_in_empty_nested_dirs() {
    let root = make_nested_dir(&["a/b/c"], &[]);
    let result = find_file_in_dirs("config.toml", &[root.path()]);
    assert!(matches!(result, Err(FindFileError::NotFound { .. })));
}

#[test]
fn test_recursive_search_across_multiple_roots() {
    // File is nested inside the second root dir
    let root1 = make_nested_dir(&["sub"], &[]);
    let root2 = make_nested_dir(&["sub"], &[("sub", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root1.path(), root2.path()]).unwrap();
    assert_eq!(result, root2.path().join("sub").join("config.toml"));
}

#[test]
fn test_does_not_confuse_filename_with_dirname() {
    // A directory named "config.toml" should not be returned
    let root = make_nested_dir(&["config.toml"], &[]);
    let result = find_file_in_dirs("config.toml", &[root.path()]);
    assert!(matches!(result, Err(FindFileError::NotFound { .. })));
}

// =========================================================
// Path correctness
// =========================================================

#[test]
fn test_returned_path_exists() {
    let root = make_nested_dir(&["sub"], &[("sub", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]).unwrap();
    assert!(result.exists());
}

#[test]
fn test_returned_path_is_absolute() {
    let root = make_nested_dir(&["sub"], &[("sub", "config.toml")]);
    let result = find_file_in_dirs("config.toml", &[root.path()]).unwrap();
    assert!(result.is_absolute());
}
