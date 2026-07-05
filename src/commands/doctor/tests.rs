use std::{env, fs};

use super::*;

#[cfg(unix)]
fn make_executable(dir: &Path, name: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\n").expect("write fake binary");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod fake binary");
    path
}

#[cfg(unix)]
#[test]
fn scan_path_finds_copies_in_order() {
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    make_executable(a.path(), "twc-rs");
    make_executable(b.path(), "twc-rs");

    let joined = env::join_paths([a.path(), b.path()]).expect("join paths");
    let found = scan_path(&joined, "twc-rs");
    assert_eq!(found.len(), 2);
    assert!(found[0].starts_with(a.path()));
    assert!(found[1].starts_with(b.path()));
}

#[cfg(unix)]
#[test]
fn scan_path_skips_non_executable_and_missing() {
    let a = tempfile::tempdir().expect("tempdir a");
    let b = tempfile::tempdir().expect("tempdir b");
    fs::write(a.path().join("twc-rs"), "not executable").expect("write plain file");

    let joined = env::join_paths([a.path(), b.path()]).expect("join paths");
    assert!(scan_path(&joined, "twc-rs").is_empty());
}

#[cfg(unix)]
#[test]
fn scan_path_dedupes_symlinked_directories() {
    let root = tempfile::tempdir().expect("tempdir");
    let real = root.path().join("usr-bin");
    fs::create_dir(&real).expect("mkdir real");
    make_executable(&real, "twc-rs");
    let alias = root.path().join("bin");
    std::os::unix::fs::symlink(&real, &alias).expect("symlink dir");

    let joined = env::join_paths([alias.as_path(), real.as_path()]).expect("join paths");
    let found = scan_path(&joined, "twc-rs");
    assert_eq!(found.len(), 1);
}

#[cfg(unix)]
#[test]
fn scan_path_skips_empty_entries() {
    let joined = std::ffi::OsString::from("::");
    assert!(scan_path(&joined, "twc-rs").is_empty());
}
