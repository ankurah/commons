//! Copies of the crate's sources for the tool's tests to mint into, and to change.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use ankurah_commons::namespaces::NAMESPACES;
use tempfile::TempDir;

use crate::{minting, placement, writing};

/// The crate's own sources.
pub fn sources() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("src")
}

/// A copy of the crate's Rust sources, with the kept mint's files.
pub fn copied() -> TempDir {
    let copy = tempfile::tempdir().unwrap();
    copy_sources(&sources(), copy.path());
    copy
}

fn copy_sources(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            fs::create_dir(&target).unwrap();
            copy_sources(&path, &target);
        } else if path.extension().is_some_and(|extension| extension == "rs" || extension == placement::EXTENSION) {
            fs::copy(&path, &target).unwrap();
        }
    }
}

/// Mints into `src` as `mint` does.
pub fn mint(src: &Path) {
    writing::write(&minting::mint(src, NAMESPACES).unwrap()).unwrap();
}

/// Every file under `dir`, by its path from `dir`, with its text.
pub fn files(dir: &Path) -> BTreeMap<PathBuf, String> {
    let mut files = BTreeMap::new();
    collect(dir, dir, &mut files);
    files
}

fn collect(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            files.insert(path.strip_prefix(root).unwrap().to_owned(), fs::read_to_string(&path).unwrap());
        }
    }
}

/// Edits the first occurrence of `from` in the file at `path`.
pub fn edit(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains(from), "{} holds no `{from}`", path.display());
    fs::write(path, text.replacen(from, to, 1)).unwrap();
}
