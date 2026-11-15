//! Shared by the integration tests: the repository root and the set on disk.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root.
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn walk(dir: &Path, base: &Path, found: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, base, found);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            found.push(
                path.strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

/// Every `.md` under `content/` on disk, set-relative, sorted.
pub fn content_files() -> Vec<String> {
    let base = root().join("content");
    let mut found = Vec::new();
    walk(&base, &base, &mut found);
    found.sort();
    found
}
