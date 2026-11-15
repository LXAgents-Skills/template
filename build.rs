//! Embed `content/` into the binary: one `(set-relative path, text)` pair per `.md` file.
//!
//! Adding a markdown file to `content/` is adding a tool. Cargo reruns this script whenever
//! anything under `content/` changes, so `cargo build` and `cargo run` always serve the set as
//! it is on disk. Nothing here checks the set: the checks run at boot, in
//! `src/instructions/surface.rs`, so a malformed set fails the same way however it was built.

use std::fs;
use std::path::{Path, PathBuf};

fn markdown_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            markdown_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            found.push(path);
        }
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let content = root.join("content");
    println!("cargo:rerun-if-changed=content");

    let mut files = Vec::new();
    if content.is_dir() {
        markdown_files(&content, &mut files);
    }

    let mut entries: Vec<(String, PathBuf)> = files
        .into_iter()
        .map(|path| {
            let relative = path.strip_prefix(&content).expect("under content/");
            let key = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            (key, path)
        })
        .collect();
    entries.sort();

    let mut out = String::from(
        "/// Every `.md` under `content/`, as (set-relative path, text), sorted by path.\npub static FILES: &[(&str, &str)] = &[\n",
    );
    for (key, path) in entries {
        out.push_str(&format!(
            "    ({key:?}, include_str!({:?})),\n",
            path.display().to_string()
        ));
    }
    out.push_str("];\n");

    let target = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("content.rs");
    fs::write(target, out).expect("write content.rs");
}
